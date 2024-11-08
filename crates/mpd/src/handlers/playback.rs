use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

pub async fn handle_play(request: &str, stream: &mut BufReader<TcpStream>) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}
