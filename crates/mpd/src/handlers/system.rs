use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{consts::DECODERS, Context};

pub async fn handle_idle(
    _ctx: &mut Context,
    _request: &str,
    _stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    Ok("".to_string())
}

pub async fn handle_noidle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    ctx.idle_state.send(false)?;

    let response = "OK\n".to_string();
    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_decoders(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    if !ctx.batch {
        stream.write_all(DECODERS.as_bytes()).await?;
    }
    Ok(DECODERS.to_string())
}
