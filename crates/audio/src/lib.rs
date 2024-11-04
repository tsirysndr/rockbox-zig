use cpal::traits::{HostTrait, StreamTrait};
use rodio::DeviceTrait;
use std::collections::VecDeque;
use std::io::{self, Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Duration;

pub const SOCKET_PATH: &str = "/tmp/rockbox_audio.socket";
const BUFFER_TARGET_DURATION_MS: u32 = 16000; // Reduced buffer target for lower latency
const MIN_BUFFER_DURATION_MS: u32 = 15000; // Minimum buffer size to prevent underruns

const BUFFER_CAPACITY: usize = 44100 * 2 * 32000; // 2 channels, 16-bit samples

struct SimpleRingBuffer {
    buffer: VecDeque<u8>,
    capacity: usize,
    condvar: Condvar,
}

impl SimpleRingBuffer {
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

    fn len(&self) -> usize {
        self.buffer.len()
    }
}

struct SharedRingBuffer {
    buffer: Mutex<SimpleRingBuffer>,
    condvar: Condvar,
}

impl SharedRingBuffer {
    fn new(capacity: usize) -> Self {
        Self {
            buffer: Mutex::new(SimpleRingBuffer::new(capacity)),
            condvar: Condvar::new(),
        }
    }
}

static mut SHARED_RING_BUFFER: Option<Arc<SharedRingBuffer>> = None;

extern "C" {
    fn pcm_callback();
}

#[no_mangle]
pub extern "C" fn process_pcm_buffer(data: *mut u8, size: usize) {
    let data = unsafe { std::slice::from_raw_parts(data, size) };

    println!("Received PCM buffer of size: {}", size);

    // Write data to the ring buffer
    if let Some(ref shared_ring_buffer) = unsafe { &SHARED_RING_BUFFER } {
        let mut buffer = shared_ring_buffer.buffer.lock().unwrap();
        buffer.write(data);
        shared_ring_buffer.condvar.notify_one(); // Notify reader thread that new data is available
        thread::sleep(Duration::from_millis(16));
    }
}

#[no_mangle]
pub extern "C" fn start_audio_buffer_broker() {
    let shared_ring_buffer = Arc::new(SharedRingBuffer::new(BUFFER_CAPACITY));
    unsafe {
        SHARED_RING_BUFFER = Some(shared_ring_buffer.clone());
    }

    thread::spawn(move || {
        loop {
            // Call the C callback function to process the PCM buffer
            unsafe {
                pcm_callback();
            }
        }
    });

    // Background thread to send data to the Unix socket
    thread::spawn(move || {
        if let Ok(mut socket) = UnixStream::connect(SOCKET_PATH) {
            let chunk_size = 4096; // Define a reasonable chunk size
            loop {
                println!("Filling chunk ...");
                let chunk = {
                    let buffer = shared_ring_buffer.buffer.lock().unwrap();
                    // Use a reference to buffer in wait_while closure
                    let mut guard = shared_ring_buffer
                        .condvar
                        .wait_while(buffer, |b| b.len() < chunk_size)
                        .unwrap();

                    // Read the chunk using the guard
                    guard.read_chunk(chunk_size)
                };

                println!("Writing chunk of size: {}", chunk.len());

                // Write chunk to socket outside the lock
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

struct RingBuffer {
    buffer: Vec<f32>,
    head: usize,
    tail: usize,
    size: usize,
    full: bool,
}

impl RingBuffer {
    pub fn new(size: usize) -> Self {
        RingBuffer {
            buffer: vec![0.0; size],
            head: 0,
            tail: 0,
            size,
            full: false,
        }
    }

    pub fn push_chunk(&mut self, chunk: &[f32]) -> bool {
        if self.available_space() < chunk.len() {
            return false;
        }

        for &sample in chunk {
            self.buffer[self.head] = sample;
            self.head = (self.head + 1) % self.size;
        }

        self.full = self.head == self.tail;
        true
    }

    pub fn pop_chunk(&mut self, chunk_size: usize) -> Vec<f32> {
        let mut result = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            if let Some(value) = self.pop() {
                result.push(value);
            } else {
                break;
            }
        }
        result
    }

    pub fn pop(&mut self) -> Option<f32> {
        if self.is_empty() {
            return None;
        }
        let value = self.buffer[self.tail];
        self.tail = (self.tail + 1) % self.size;
        self.full = false;
        Some(value)
    }

    pub fn is_empty(&self) -> bool {
        !self.full && (self.head == self.tail)
    }

    pub fn len(&self) -> usize {
        if self.full {
            self.size
        } else if self.head >= self.tail {
            self.head - self.tail
        } else {
            self.size + self.head - self.tail
        }
    }

    pub fn available_space(&self) -> usize {
        self.size - self.len()
    }
}

pub fn read_audio_socket() -> Result<(), anyhow::Error> {
    let format = PcmFormat::default();

    // Calculate buffer sizes based on target latency
    let samples_per_ms = (format.sample_rate as f32 / 1000.0) as usize * format.channels as usize;
    let ring_buffer_size = samples_per_ms * BUFFER_TARGET_DURATION_MS as usize;
    let min_buffer_samples = samples_per_ms * MIN_BUFFER_DURATION_MS as usize;

    let ring_buffer = Arc::new((
        Mutex::new(RingBuffer::new(ring_buffer_size)),
        Condvar::new(),
    ));

    // Socket thread
    let ring_buffer_clone = Arc::clone(&ring_buffer);
    let socket_thread = thread::spawn(move || {
        let _ = std::fs::remove_file(SOCKET_PATH);
        let listener = UnixListener::bind(SOCKET_PATH).expect("Could not bind Unix socket");

        for stream in listener.incoming() {
            let mut stream = stream.expect("Failed to accept connection");
            let mut buffer = vec![0u8; 4096];

            stream
                .set_nonblocking(true)
                .expect("Failed to set non-blocking");

            loop {
                match stream.read(&mut buffer) {
                    Ok(n) => {
                        if n == 0 {
                            // EOF reached
                            println!("EOF reached, closing socket.");
                            break;
                        }

                        println!("{:?}", &buffer[..5]);

                        let (lock, cvar) = &*ring_buffer_clone;
                        let mut ring_buffer = lock.lock().unwrap();

                        let mut samples = Vec::with_capacity(n / 2);
                        for chunk in buffer[..n].chunks_exact(2) {
                            samples.push(convert_sample(chunk, &format));
                        }

                        if !ring_buffer.push_chunk(&samples) {
                            // Buffer is full, skip some samples
                            ring_buffer.pop_chunk(samples.len());
                            ring_buffer.push_chunk(&samples);
                        }

                        cvar.notify_one();
                    }
                    Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // thread::sleep(Duration::from_micros(10));
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

    // Playback thread
    let play_thread = thread::spawn(move || {
        let format = PcmFormat::default();
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to get default output device");

        let config = cpal::StreamConfig {
            channels: format.channels,
            sample_rate: cpal::SampleRate(format.sample_rate),
            buffer_size: cpal::BufferSize::Fixed(4096),
        };

        let stream = device
            .build_output_stream(
                &config,
                move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                    let (lock, cvar) = &*ring_buffer;
                    let mut ring_buffer = lock.lock().unwrap();

                    if ring_buffer.len() < min_buffer_samples {
                        println!("Buffer underrun, waiting for data...");
                        ring_buffer = cvar
                            .wait_timeout(ring_buffer, Duration::from_millis(10))
                            .unwrap()
                            .0;
                    }

                    // Fill output buffer
                    let samples = ring_buffer.pop_chunk(data.len());
                    let samples_len = samples.len();
                    for (i, sample) in samples.into_iter().enumerate() {
                        data[i] = sample;
                    }

                    // Fill remaining space with silence if we run out of data
                    for sample in data.iter_mut().skip(samples_len) {
                        *sample = 0.0;
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

    socket_thread.join().expect("Socket thread panicked");
    play_thread.join().expect("Play thread panicked");

    Ok(())
}
