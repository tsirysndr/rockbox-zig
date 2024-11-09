use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{
    NextRequest, PauseRequest, PreviousRequest, ResumeRequest, StatusRequest,
};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::Context;

pub async fn handle_play(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    ctx.playback.resume(ResumeRequest {}).await?;
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_pause(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    let arg = request.split_whitespace().nth(1);
    match arg {
        Some(r#""0""#) => {
            ctx.playback.resume(ResumeRequest {}).await?;
            stream.write_all(b"OK\n").await?;
        }
        Some(r#""1""#) => {
            ctx.playback.pause(PauseRequest {}).await?;
            stream.write_all(b"OK\n").await?;
        }
        _ => {
            stream
                .write_all(b"ACK [2@0] {pause} incorrect arguments\n")
                .await?;
        }
    }
    Ok(())
}

pub async fn handle_toggle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    let response = ctx.playback.status(StatusRequest {}).await?;
    let response = response.into_inner();
    match response.status {
        1 => {
            ctx.playback.pause(PauseRequest {}).await?;
        }
        3 => {
            ctx.playback.resume(ResumeRequest {}).await?;
        }
        _ => {
            stream
                .write_all(b"ACK [2@0] {toggle} no song is playing\n")
                .await?;
        }
    }
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_status(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    let response = ctx.playback.status(StatusRequest {}).await?;
    let response = response.into_inner();
    let status = match response.status {
        1 => "play",
        3 => "pause",
        _ => "stop",
    };
    let response = format!("state: {}\nOK\n", status);
    stream.write_all(response.as_bytes()).await?;
    Ok(())
}

pub async fn handle_next(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    ctx.playback.next(NextRequest {}).await?;
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_previous(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    ctx.playback.previous(PreviousRequest {}).await?;
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_playid(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_seek(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_seekid(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_seekcur(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_stop(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_random(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_repeat(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_getvol(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_setvol(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_single(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    println!("{}", request);
    stream.write_all(b"OK\n").await?;
    Ok(())
}
