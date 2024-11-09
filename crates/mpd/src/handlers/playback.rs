use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{
    NextRequest, PauseRequest, PlayRequest, PreviousRequest, ResumeRequest, SaveSettingsRequest,
    StatusRequest,
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
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {seekcur} incorrect arguments\n")
            .await?;
        return Ok(());
    }

    ctx.playback
        .play(PlayRequest {
            elapsed: arg
                .map(|x| x.replace("\"", ""))
                .map(|x| x.parse::<i64>().unwrap() * 1000)
                .unwrap_or_default(),
            offset: 0,
        })
        .await?;
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_random(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {random} incorrect arguments\n")
            .await?;
        return Ok(());
    }

    ctx.settings
        .save_settings(SaveSettingsRequest {
            playlist_shuffle: Some(arg.unwrap() == r#""1""#),
            ..Default::default()
        })
        .await?;
    stream.write_all(b"OK\n").await?;
    Ok(())
}

pub async fn handle_repeat(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<(), Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {repeat} incorrect arguments\n")
            .await?;
        return Ok(());
    }

    let single = ctx.single.lock().await;

    let repeat_mode = match arg.unwrap() {
        r#""0""# => Some(0),
        r#""1""# => match single.as_str() {
            r#""1""# => Some(2),
            _ => Some(1),
        },
        _ => {
            stream
                .write_all(b"ACK [2@0] {repeat} incorrect arguments\n")
                .await?;
            return Ok(());
        }
    };
    ctx.settings
        .save_settings(SaveSettingsRequest {
            repeat_mode,
            ..Default::default()
        })
        .await?;
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
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {single} incorrect arguments\n")
            .await?;
        return Ok(());
    }

    let mut single = ctx.single.lock().await;
    *single = arg.unwrap().to_string();
    stream.write_all(b"OK\n").await?;
    Ok(())
}
