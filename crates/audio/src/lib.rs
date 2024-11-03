use anyhow::Error;
use sdl2::audio::{AudioCallback, AudioSpecDesired};
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const SOCKET_PATH: &str = "/tmp/rockbox_audio.socket";
// Constants for buffer management
const SAMPLE_RATE: u32 = 44100;
const CHANNELS: u16 = 2;
const BUFFER_SIZE_SAMPLES: usize = 128; // Smaller chunks for SDL callback
const MIN_BUFFER_SIZE: usize = SAMPLE_RATE as usize * CHANNELS as usize * 2; // 2 seconds of audio data
const MAX_BUFFER_SIZE: usize = SAMPLE_RATE as usize * CHANNELS as usize * 10; // 6 seconds

const BUFFER_CAPACITY: usize = 65536; // Define an appropriate capacity for the ring buffer

struct RingBuffer {
    buffer: VecDeque<u8>,
    capacity: usize,
    condvar: Condvar,
}

impl RingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(capacity),
            capacity,
            condvar: Condvar::new(),
        }
    }

    fn write(&mut self, data: &[u8]) {
        for &byte in data {
            if self.buffer.len() < self.capacity {
                self.buffer.push_back(byte);
            }
        }
        self.condvar.notify_one(); // Notify reader thread that new data is available
    }

    fn read_chunk(&mut self, chunk_size: usize) -> Vec<u8> {
        let mut chunk = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            if let Some(byte) = self.buffer.pop_front() {
                chunk.push(byte);
            } else {
                break; // Stop if there’s no data left
            }
        }
        chunk
    }
}

struct SharedRingBuffer {
    buffer: Mutex<RingBuffer>,
    condvar: Condvar,
}

impl SharedRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(RingBuffer::new(capacity)),
            condvar: Condvar::new(),
        }
    }
}

// Global shared buffer to be used in `process_pcm_buffer` and writer thread
static mut SHARED_RING_BUFFER: Option<Arc<SharedRingBuffer>> = None;

#[no_mangle]
pub extern "C" fn process_pcm_buffer(data: *mut u8, size: usize) {
    let data = unsafe { std::slice::from_raw_parts(data, size) };

    // Write data to the ring buffer
    if let Some(ref shared_ring_buffer) = unsafe { &SHARED_RING_BUFFER } {
        let mut buffer = shared_ring_buffer.buffer.lock().unwrap();
        buffer.write(data);
    }
}

#[no_mangle]
pub extern "C" fn start_audio_buffer_broker() {
    let shared_ring_buffer = Arc::new(SharedRingBuffer::new(BUFFER_CAPACITY));
    unsafe {
        SHARED_RING_BUFFER = Some(shared_ring_buffer.clone());
    }

    // Background thread to send data to the Unix socket
    thread::spawn(move || {
        if let Ok(mut socket) = UnixStream::connect("/tmp/rockbox_audio.socket") {
            let chunk_size = 4096; // Define a reasonable chunk size
            loop {
                let buffer = shared_ring_buffer.buffer.lock().unwrap();

                // Wait for data to be available
                let chunk = shared_ring_buffer
                    .condvar
                    .wait_while(buffer, |x| x.buffer.len() < chunk_size)
                    .unwrap()
                    .read_chunk(chunk_size);

                // drop(buffer); // Release lock before writing to the socket

                // Write chunk to socket
                if let Err(e) = socket.write_all(&chunk) {
                    eprintln!("Error writing to socket: {}", e);
                    break;
                }
            }
        } else {
            eprintln!("Failed to connect to Unix socket");
        }
    });
}

struct AudioState {
    data: VecDeque<i16>,
}

struct SharedState {
    state: Mutex<AudioState>,
    condvar: Condvar,
}

impl SharedState {
    fn new() -> Self {
        Self {
            state: Mutex::new(AudioState {
                data: VecDeque::with_capacity(MAX_BUFFER_SIZE),
            }),
            condvar: Condvar::new(),
        }
    }
}

struct AudioPlayer {
    shared: Arc<SharedState>,
}

impl AudioCallback for AudioPlayer {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        let mut state = self.shared.state.lock().unwrap();

        println!("AudioCallback buffer length: {}", state.data.len());

        // If buffer is too small, output silence
        if state.data.len() < MIN_BUFFER_SIZE {
            self.shared.condvar.notify_one();
            for sample in out.iter_mut() {
                *sample = 0;
            }
            return;
        }

        // Copy samples to output buffer
        for sample in out.iter_mut() {
            *sample = state.data.pop_front().unwrap_or(0);
        }

        // Notify if buffer needs more data
        if state.data.len() < MIN_BUFFER_SIZE {
            self.shared.condvar.notify_one();
        }
    }
}

fn convert_samples(input: &[u8]) -> Vec<i16> {
    input
        .chunks_exact(2)
        .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
        .collect()
}
pub fn read_audio_socket() -> Result<(), Error> {
    let sdl_context = sdl2::init().map_err(|e| Error::msg(e.to_string()))?;
    let audio_subsystem = sdl_context.audio().map_err(|e| Error::msg(e.to_string()))?;

    // Create shared state
    let shared = Arc::new(SharedState::new());

    // Set up audio device
    let desired_spec = AudioSpecDesired {
        freq: Some(SAMPLE_RATE as i32),
        channels: Some(CHANNELS as u8),
        samples: Some(BUFFER_SIZE_SAMPLES as u16),
    };

    let audio_player = AudioPlayer {
        shared: Arc::clone(&shared),
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |_spec| audio_player)
        .map_err(|e| Error::msg(e.to_string()))?;

    // Set up socket listener
    let _ = std::fs::remove_file(SOCKET_PATH);
    let listener = UnixListener::bind(SOCKET_PATH)?;
    println!("Listening on {}", SOCKET_PATH);

    // Start playback
    device.resume();

    // Spawn a background thread for continuously reading the audio data
    let shared_clone = Arc::clone(&shared);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let mut stream = match stream {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error accepting connection: {}", e);
                    continue;
                }
            };
            stream.set_nonblocking(true).unwrap();

            let mut read_buffer = vec![0u8; BUFFER_SIZE_SAMPLES * 32];
            let mut initial_fill_done = false;

            loop {
                let mut state = shared_clone.state.lock().unwrap();

                println!("Background thread buffer length: {}", state.data.len());

                if !initial_fill_done && state.data.len() >= MIN_BUFFER_SIZE {
                    println!("Initial buffer fill complete, resuming playback.");
                    initial_fill_done = true;
                }

                // Wait if buffer is full
                if state.data.len() >= MAX_BUFFER_SIZE {
                    let _unused = shared_clone
                        .condvar
                        .wait_while(state, |state| state.data.len() >= MAX_BUFFER_SIZE)
                        .unwrap();
                    continue;
                }

                match stream.read(&mut read_buffer) {
                    Ok(n) if n == 0 => break, // EOF
                    Ok(n) => {
                        // Convert and add samples to the buffer
                        let samples = convert_samples(&read_buffer[..n]);
                        state.data.extend(samples);
                        shared_clone.condvar.notify_one();
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        println!("No data available yet, sleeping.");
                        // Sleep briefly to avoid busy waiting
                        drop(state);
                        thread::sleep(Duration::from_millis(1));
                        continue;
                    }
                    Err(e) => {
                        eprintln!("Error reading from socket: {}", e);
                        break;
                    }
                }
            }
        }
    });

    loop {
        thread::sleep(Duration::from_millis(100));
    }
}
