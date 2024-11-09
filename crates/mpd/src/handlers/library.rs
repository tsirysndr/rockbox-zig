use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{GetGlobalSettingsRequest, ScanLibraryRequest};
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::Context;

pub async fn handle_list_album(
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

pub async fn handle_list_artist(
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

pub async fn handle_list_title(
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

pub async fn handle_search(
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

pub async fn handle_rescan(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let path = request
        .replace("update ", "")
        .replace("rescan ", "")
        .replace("\"", "");
    let path = Some(match path.starts_with("/") {
        true => path,
        false => format!("{}/{}", response.music_dir, path),
    });
    ctx.library
        .scan_library(ScanLibraryRequest { path })
        .await?;

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_config(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let response = format!("music_directory: {}\nOK\n", response.music_dir);

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}
