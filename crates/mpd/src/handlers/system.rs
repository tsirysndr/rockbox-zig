use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{
    consts::{COMMANDS, DECODERS},
    Context,
};

pub async fn handle_idle(
    _ctx: &mut Context,
    _request: &str,
    _stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    // TODO: Implement idle
    /*
        let idle = ctx.idle.lock().await;

        if *idle {
            stream
                .write_all(b"changed: player\nchanged: playlist\nOK\n")
                .await?;
            return Ok("changed: player\nchanged: playlist\nOK\n".to_string());
        }
    */
    Ok("".to_string())
}

pub async fn handle_noidle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    ctx.idle_state.send(false)?;
    let mut idle = ctx.idle.lock().await;
    *idle = false;

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

pub async fn handle_commands(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    if !ctx.batch {
        stream.write_all(COMMANDS.as_bytes()).await?;
    }
    Ok(COMMANDS.to_string())
}
