use std::env;

use anyhow::Error;
use handlers::playback::handle_play;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
};

pub mod handlers;

pub struct MpdServer {}

impl MpdServer {
    pub async fn start() -> Result<(), Error> {
        let port = env::var("ROCKBOX_MPD_PORT").unwrap_or_else(|_| "6600".to_string());
        let addr = format!("0.0.0.0:{}", port);

        let listener = TcpListener::bind(&addr).await?;

        loop {
            let (stream, _) = listener.accept().await?;
            tokio::spawn(async move {
                match handle_client(stream).await {
                    Ok(_) => {}
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }
    }
}

pub async fn handle_client(stream: TcpStream) -> Result<(), Error> {
    let mut buf = [0; 1024];
    let mut stream = tokio::io::BufReader::new(stream);
    stream.write_all(b"OK MPD 0.23.15\n").await?;

    while let Ok(n) = stream.read(&mut buf).await {
        if n == 0 {
            break;
        }
        let request = String::from_utf8_lossy(&buf[..n]);
        let command = parse_command(&request)?;

        match command {
            "play" => handle_play(&request, &mut stream).await?,
            _ => {}
        }
    }
    Ok(())
}

fn parse_command(request: &str) -> Result<&str, Error> {
    let command = request.split_whitespace().next().unwrap_or_default();
    Ok(command)
}
