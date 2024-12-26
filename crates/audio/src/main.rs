use std::collections::VecDeque;
use std::env;
use std::fs::File;
use std::io::{ErrorKind, Read};
use std::path::Path;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

const BUFFER_DURATION_SECS: f32 = 60.0; // Increased buffer size
const MIN_BUFFER_FILL_PERCENT: f32 = 0.5; // Start playback when buffer is 80% full

struct RingBuffer {
    buffer: VecDeque<f32>,
    capacity: usize,
    ready: bool,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            ready: false,
        }
    }

    fn write_chunk(&mut self, samples: &[f32]) -> usize {
        let space = self.capacity - self.buffer.len();
        let write_count = samples.len().min(space);
        self.buffer.extend(&samples[..write_count]);
        write_count
    }

    fn read_chunk(&mut self, out: &mut [f32]) -> usize {
        let available = self.buffer.len().min(out.len());
        for i in 0..available {
            out[i] = self.buffer.pop_front().unwrap_or(0.0);
        }
        available
    }

    fn available(&self) -> usize {
        self.buffer.len()
    }

    fn capacity(&self) -> usize {
        self.capacity
    }

    fn is_ready(&self) -> bool {
        self.ready
    }

    fn set_ready(&mut self, ready: bool) {
        self.ready = ready;
    }

    fn current_size(&self) -> usize {
        self.buffer.len()
    }
}

#[derive(Debug)]
struct PcmFormat {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    is_float: bool,
    is_signed: bool,
    is_le: bool,
}

impl Default for PcmFormat {
    fn default() -> Self {
        Self {
            sample_rate: 44100,
            channels: 2,
            bits_per_sample: 16,
            is_float: false,
            is_signed: true,
            is_le: true,
        }
    }
}

fn convert_sample(buffer: &[u8], format: &PcmFormat) -> f32 {
    match (
        format.bits_per_sample,
        format.is_float,
        format.is_signed,
        format.is_le,
    ) {
        (16, false, true, true) => {
            let value = i16::from_le_bytes([buffer[0], buffer[1]]);
            value as f32 / 32768.0
        }
        (16, false, true, false) => {
            let value = i16::from_be_bytes([buffer[0], buffer[1]]);
            value as f32 / 32768.0
        }
        (32, true, _, true) => f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        (32, true, _, false) => f32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        (8, false, false, _) => (buffer[0] as f32 - 128.0) / 128.0,
        (8, false, true, _) => buffer[0] as i8 as f32 / 128.0,
        _ => {
            eprintln!(
                "Unsupported format: {}bit, float={}, signed={}, le={}",
                format.bits_per_sample, format.is_float, format.is_signed, format.is_le
            );
            0.0
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <raw_pcm_file/fifo> [sample_rate] [channels] [bits] [float/int] [signed/unsigned] [le/be]", args[0]);
        println!("Default format: 44100Hz, 2ch, 16-bit signed integer, little-endian");
        return Ok(());
    }

    let mut format = PcmFormat::default();
    if args.len() > 2 {
        format.sample_rate = args[2].parse().unwrap_or(44100);
    }
    if args.len() > 3 {
        format.channels = args[3].parse().unwrap_or(2);
    }
    if args.len() > 4 {
        format.bits_per_sample = args[4].parse().unwrap_or(16);
    }
    if args.len() > 5 {
        format.is_float = args[5].to_lowercase() == "float";
    }
    if args.len() > 6 {
        format.is_signed = args[6].to_lowercase() == "signed";
    }
    if args.len() > 7 {
        format.is_le = args[7].to_lowercase() == "le";
    }

    println!("Playing PCM from {} with format: {:?}", args[1], format);

    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");

    // Calculate buffer sizes
    let total_buffer_samples =
        (format.sample_rate as f32 * BUFFER_DURATION_SECS * format.channels as f32) as usize;
    let chunk_samples = format.sample_rate as usize / 10 * format.channels as usize; // 100ms chunks

    let config = cpal::StreamConfig {
        channels: format.channels,
        sample_rate: cpal::SampleRate(format.sample_rate),
        buffer_size: cpal::BufferSize::Fixed(chunk_samples as u32),
    };

    let ring_buffer = Arc::new(Mutex::new(RingBuffer::new(total_buffer_samples)));
    let ring_buffer_player = ring_buffer.clone();

    let (stop_tx, stop_rx) = mpsc::channel();
    let (ready_tx, ready_rx) = mpsc::channel();

    let ring_buffer_clone = ring_buffer.clone();
    // Spawn audio playback thread

    let play_thread = thread::spawn(move || {
        let mut output_buffer = vec![0.0f32; chunk_samples];

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut ring_buffer = ring_buffer_player.lock().unwrap();
                    if !ring_buffer.is_ready() {
                        return;
                    }
                    let read = ring_buffer.read_chunk(data);

                    // Fill remaining with silence if buffer underrun
                    if read < data.len() {
                        data[read..].fill(0.0);
                    }
                },
                |err| eprintln!("Error in audio playback: {}", err),
                None,
            )
            .expect("Failed to build output stream");

        // Wait for buffer to fill initially/
        // ready_rx.recv().unwrap();

        println!("Waiting for buffer fill");
        std::thread::sleep(Duration::from_secs(20));

        let mut mutex = ring_buffer_clone.lock().unwrap();
        mutex.set_ready(true);
        drop(mutex);

        println!("Starting playback");
        stream.play().expect("Failed to play stream");

        // Wait for stop signal
        let _ = stop_rx.recv();
    });

    let bytes_per_sample = (format.bits_per_sample as usize + 7) / 8;
    let mut file = File::open(Path::new(&args[1]))?;
    let mut samples_played = 0;
    let mut is_playing = false;

    // Pre-allocate buffers
    let chunk_bytes = chunk_samples * bytes_per_sample;
    let mut input_chunk = vec![0u8; chunk_bytes];
    let mut conversion_buffer = Vec::with_capacity(chunk_samples);

    loop {
        match file.read(&mut input_chunk) {
            Ok(0) => {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            Ok(bytes_read) => {
                conversion_buffer.clear();
                println!(">> bytes read {}", bytes_read);
                println!(">> input chunk {}", chunk_bytes);

                // Convert bytes to samples in chunks
                for chunk in input_chunk[..bytes_read].chunks_exact(bytes_per_sample) {
                    let sample = convert_sample(chunk, &format);
                    let normalized_sample = if sample.abs() > 1.0 {
                        sample.signum()
                    } else {
                        sample
                    };
                    conversion_buffer.push(normalized_sample);
                }

                // Write to ring buffer
                loop {
                    let mut ring_buffer = ring_buffer.lock().unwrap();
                    let written = ring_buffer.write_chunk(&conversion_buffer);
                    println!(
                        "Writing to ring buffer {} - {}",
                        ring_buffer.current_size(),
                        conversion_buffer.len()
                    );

                    if written == conversion_buffer.len() {
                        // Check if we should start playback
                        if !is_playing
                            && (ring_buffer.available() as f32 / ring_buffer.capacity() as f32)
                                >= MIN_BUFFER_FILL_PERCENT
                        {
                            ready_tx.send(()).unwrap();
                            is_playing = true;
                        }
                        break;
                    }

                    drop(ring_buffer);
                }

                samples_played += conversion_buffer.len();
                if samples_played % (format.sample_rate as usize * format.channels as usize) == 0 {
                    print!(
                        "\rPlayed {} seconds",
                        samples_played / format.sample_rate as usize / format.channels as usize
                    );
                }
            }
            Err(e) if e.kind() == ErrorKind::Interrupted => continue,
            Err(e) => {
                eprintln!("\nError reading from file: {}", e);
                break;
            }
        }
    }

    let _ = stop_tx.send(());
    play_thread.join().unwrap();

    println!("\nPlayback finished");
    Ok(())
}
