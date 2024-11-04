use cpal::traits::{HostTrait, StreamTrait};
use rodio::DeviceTrait;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

pub const SOCKET_PATH: &str = "/tmp/rockbox_audio.socket";

extern "C" {
    fn pcm_callback(data: *mut u8, size: *mut usize);
}

#[no_mangle]
pub extern "C" fn start_audio_buffer_broker() {
    thread::spawn(move || {
        let _ = std::fs::remove_file(SOCKET_PATH);
        // start unix domain socket server
        let listener = UnixListener::bind(SOCKET_PATH).expect("Could not bind Unix socket");

        for stream in listener.incoming() {
            match stream {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        // Keep handling requests on this connection
                        loop {
                            let mut buffer = [0; 1024];
                            let bytes_read = match stream.read(&mut buffer) {
                                Ok(bytes) => bytes,
                                Err(e) => {
                                    eprintln!("Error reading from socket: {}", e);
                                    break;
                                }
                            };

                            if bytes_read == 0 {
                                // Client closed the connection
                                println!("Client disconnected");
                                break;
                            }

                            let request = String::from_utf8_lossy(&buffer[..bytes_read]);
                            println!(">> Received request: {}", request);

                            let mut data: Vec<u8> = vec![0; 4096];

                            let mut size = data.len();
                            // Get mutable pointers for `data` and `size`
                            let data_ptr = data.as_mut_ptr();
                            let size_ptr = &mut size;

                            unsafe {
                                pcm_callback(data_ptr, size_ptr);
                            }

                            println!("<< data: {:?}", &data[..10]);
                            println!("<< Response size: {}", size);

                            let response = unsafe { std::slice::from_raw_parts(data_ptr, size) };

                            if let Err(e) = stream.write_all(&response) {
                                eprintln!("Failed to send response: {}", e);
                                break;
                            }
                            if let Err(e) = stream.flush() {
                                eprintln!("Failed to flush response: {}", e);
                                break;
                            }
                            println!("<< Response sent: {:?}", &response[..10]);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Connection failed: {}", e);
                }
            }
        }
        Ok::<(), io::Error>(())
    });
}

#[derive(Debug, Clone)]
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

const BUFFER_SIZE: usize = 4096; // Increased buffer size
const PREBUFFER_THRESHOLD: usize = BUFFER_SIZE * 32; // Amount to prebuffer before starting playback

struct AudioBuffer {
    queue: VecDeque<f32>,
    format: PcmFormat,
}

impl AudioBuffer {
    fn new(format: PcmFormat) -> Self {
        Self {
            queue: VecDeque::with_capacity(PREBUFFER_THRESHOLD),
            format,
        }
    }

    fn push_samples(&mut self, samples: Vec<f32>) {
        self.queue.extend(samples);
    }

    fn get_samples(&mut self, count: usize) -> Vec<f32> {
        (0..count)
            .map(|_| self.queue.pop_front().unwrap_or(0.0))
            .collect()
    }

    fn available_samples(&self) -> usize {
        self.queue.len()
    }
}

pub fn read_audio_socket() -> Result<(), anyhow::Error> {
    let socket = UnixStream::connect(SOCKET_PATH)?;
    let shared_socket = Arc::new(Mutex::new(socket));

    // Create audio buffer
    let audio_buffer = Arc::new(Mutex::new(AudioBuffer::new(PcmFormat::default())));
    let audio_buffer_clone = audio_buffer.clone();

    // Buffer filling thread
    let fill_thread = {
        let shared_socket = shared_socket.clone();
        thread::spawn(move || {
            loop {
                let mut buffer = audio_buffer.lock().unwrap();
                if buffer.available_samples() < PREBUFFER_THRESHOLD {
                    // Request more audio data
                    if let Ok(mut socket) = shared_socket.lock() {
                        let request = b"request_pcm_buffer";
                        if socket.write_all(request).is_ok() {
                            let mut response = vec![0u8; BUFFER_SIZE];
                            if let Ok(_) = socket.read_exact(&mut response) {
                                if response.iter().all(|x| *x == 0) {
                                    continue;
                                }
                                let samples = response
                                    .chunks_exact(2)
                                    .map(|chunk| convert_sample(chunk, &buffer.format))
                                    .collect::<Vec<f32>>();
                                buffer.push_samples(samples);
                            }
                        }
                    }
                }
                drop(buffer); // Release the lock
                thread::sleep(Duration::from_millis(40));
            }
        })
    };

    // Playback thread
    let play_thread = thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to get default output device");

        let config = cpal::StreamConfig {
            channels: 2,
            sample_rate: cpal::SampleRate(44100),
            buffer_size: cpal::BufferSize::Fixed(4096),
        };

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let mut buffer = audio_buffer_clone.lock().unwrap();

                    // Wait for initial buffering
                    if buffer.available_samples() < PREBUFFER_THRESHOLD {
                        for sample in data.iter_mut() {
                            *sample = 0.0;
                        }
                        return;
                    }

                    // Get samples from buffer
                    let samples = buffer.get_samples(data.len());
                    for (i, &sample) in samples.iter().enumerate() {
                        if i < data.len() {
                            data[i] = sample;
                        }
                    }
                },
                |err| eprintln!("Error in audio playback: {}", err),
                None,
            )
            .expect("Failed to build output stream");

        stream.play().expect("Failed to play stream");

        loop {
            thread::sleep(Duration::from_secs(1));
        }
    });

    fill_thread.join().unwrap();
    play_thread.join().unwrap();

    Ok(())
}
