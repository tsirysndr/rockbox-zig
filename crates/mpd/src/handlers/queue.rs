use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::ShufflePlaylistRequest;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::Context;

pub async fn handle_shuffle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    ctx.playlist
        .shuffle_playlist(ShufflePlaylistRequest { start_index: 0 })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_add(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_playlistinfo(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_delete(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_clear(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_move(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    println!("{}", request);
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}
