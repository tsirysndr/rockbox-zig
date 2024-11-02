use std::env;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::sync::mpsc;
use std::thread;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

#[derive(Debug)]
struct PcmFormat {
    sample_rate: u32,
    channels: u16,
    bits_per_sample: u16,
    is_float: bool,
    is_signed: bool,
    is_le: bool, // true for little-endian, false for big-endian
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
        // 16-bit signed integer
        (16, false, true, true) => {
            let value = i16::from_le_bytes([buffer[0], buffer[1]]);
            value as f32 / 32768.0
        }
        (16, false, true, false) => {
            let value = i16::from_be_bytes([buffer[0], buffer[1]]);
            value as f32 / 32768.0
        }
        // 32-bit float
        (32, true, _, true) => f32::from_le_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        (32, true, _, false) => f32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]),
        // 8-bit unsigned
        (8, false, false, _) => (buffer[0] as f32 - 128.0) / 128.0,
        // 8-bit signed
        (8, false, true, _) => buffer[0] as i8 as f32 / 128.0,

        // Add more format conversions as needed
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
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: {} <raw_pcm_file> [sample_rate] [channels] [bits] [float/int] [signed/unsigned] [le/be]", args[0]);
        println!("Default format: 44100Hz, 2ch, 16-bit signed integer, little-endian");
        return Ok(());
    }

    // Parse format arguments or use defaults
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

    println!("Playing PCM file with format: {:?}", format);

    // Set up audio output
    let host = cpal::default_host();
    let device = host
        .default_output_device()
        .expect("Failed to get default output device");

    let config = cpal::StreamConfig {
        channels: format.channels,
        sample_rate: cpal::SampleRate(format.sample_rate),
        buffer_size: cpal::BufferSize::Default,
    };

    // Create a channel to send audio samples
    let (tx, rx) = mpsc::sync_channel(4096); // Increased buffer size

    // Spawn audio playback thread
    let play_thread = thread::spawn(move || {
        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    for sample in data.iter_mut() {
                        *sample = rx.try_recv().unwrap_or(0.0);
                    }
                },
                |err| eprintln!("Error in audio playback: {}", err),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play stream");

        loop {
            thread::sleep(std::time::Duration::from_millis(100));
        }
    });

    // Calculate bytes per sample
    let bytes_per_sample = (format.bits_per_sample as usize + 7) / 8;
    let mut file = File::open(Path::new(&args[1]))?;
    let mut samples_played = 0;

    // Read file in larger chunks for efficiency
    const CHUNK_SIZE: usize = 1024;
    let mut chunk = vec![0u8; CHUNK_SIZE * bytes_per_sample];

    while let Ok(bytes_read) = file.read(&mut chunk) {
        if bytes_read == 0 {
            break;
        }

        // Process each sample in the chunk
        for chunk in chunk[..bytes_read].chunks_exact(bytes_per_sample) {
            let sample = convert_sample(chunk, &format);

            // Basic DC offset removal and normalization
            let normalized_sample = if sample.abs() > 1.0 {
                sample.signum() // Clip to [-1.0, 1.0]
            } else {
                sample
            };

            if tx.send(normalized_sample).is_err() {
                println!("\nPlayback stopped");
                return Ok(());
            }

            samples_played += 1;
            if samples_played % format.sample_rate == 0 {
                print!(
                    "\rPlayed {} seconds",
                    samples_played / format.sample_rate / format.channels as u32
                );
            }
        }
    }

    println!("\nFinished playback");
    play_thread.join().unwrap();

    Ok(())
}
