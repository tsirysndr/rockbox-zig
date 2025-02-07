use std::{
    io::{Read, Write},
    os::unix::net::{UnixListener, UnixStream},
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
};

use rodio::{buffer::SamplesBuffer, OutputStream, Sink};

pub const SOCKET_PATH: &str = "/tmp/rockbox_audio.socket";
lazy_static::lazy_static! {
    static ref SENDER: Arc<Mutex<Option<mpsc::Sender<Vec<u8>>>>> = Arc::new(Mutex::new(None));
}

#[no_mangle]
pub extern "C" fn process_pcm_buffer(data: *mut u8, size: usize) {
    let data = unsafe { std::slice::from_raw_parts(data, size).to_vec() };

    let sender = SENDER.lock().unwrap();
    if let Some(ref sender) = *sender {
        if sender.send(data).is_err() {
            eprintln!("Failed to send data to socket writer thread.");
        }
    }
}

#[no_mangle]
pub extern "C" fn start_audio_buffer_broker() {
    let (tx, rx) = mpsc::channel::<Vec<u8>>();
    *SENDER.lock().unwrap() = Some(tx); // Set the global sender safely

    thread::spawn(move || {
        loop {
            match UnixStream::connect(SOCKET_PATH) {
                Ok(mut socket) => {
                    println!("Connected to Unix socket at {}", SOCKET_PATH);
                    while let Ok(data) = rx.recv() {
                        println!("Received data of size: {}", data.len());
                        if socket.write_all(&data).is_err() {
                            eprintln!("Error writing to socket. Reconnecting...");
                            break; // Break to reconnect
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Failed to connect to Unix socket at {}: {}", SOCKET_PATH, e);
                    std::thread::sleep(std::time::Duration::from_secs(1)); // Retry after delay
                }
            }
        }
    });
}

pub fn read_audio_socket() -> Result<(), anyhow::Error> {
    // remove the socket if it already exists
    std::fs::remove_file(SOCKET_PATH).ok();
    let listener = UnixListener::bind(SOCKET_PATH).expect("Failed to bind socket");

    let (_stream, stream_handle) = OutputStream::try_default()?;
    let sink = Sink::try_new(&stream_handle)?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = vec![0; 4096];
                let bytes_read = stream.read(&mut buffer).expect("Failed to read data");
                println!("Received {} bytes", bytes_read);

                let samples: Vec<i16> = buffer
                    .chunks_exact(2) // Each i16 is 2 bytes
                    .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                    .collect();

                // Create an audio buffer from samples
                let source = SamplesBuffer::new(2, 44100, samples);
                sink.append(source);
            }
            Err(e) => eprintln!("Connection failed: {}", e),
        }
    }
    Ok(())
}
