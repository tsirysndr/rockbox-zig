use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::Context;

pub async fn handle_list_album(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_list_artist(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_list_title(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_update(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_search(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_rescan(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}
