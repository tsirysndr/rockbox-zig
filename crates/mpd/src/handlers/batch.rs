use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{parse_command, setup_context, Context};

use super::{
    browse::{handle_listall, handle_listallinfo, handle_listfiles, handle_lsinfo},
    library::{
        handle_config, handle_list_album, handle_list_artist, handle_list_title, handle_rescan,
        handle_search, handle_stats, handle_tagtypes, handle_tagtypes_clear,
        handle_tagtypes_enable,
    },
    playback::{
        handle_currentsong, handle_getvol, handle_next, handle_outputs, handle_pause, handle_play,
        handle_playid, handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur,
        handle_seekid, handle_setvol, handle_single, handle_status, handle_toggle,
    },
    queue::{
        handle_add, handle_clear, handle_delete, handle_move, handle_playlistinfo, handle_shuffle,
    },
    system::handle_decoders,
};

pub async fn handle_command_list_begin(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true, Some(ctx.clone())).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();
    for request in commands {
        let command = parse_command(&request)?;
        response.push_str(&match_command(&command, &mut ctx, request, stream).await?);
    }

    stream.write_all(response.as_bytes()).await?;

    Ok(response)
}

pub async fn handle_command_list_ok_begin(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true, Some(ctx.clone())).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_ok_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();

    for request in commands {
        let command = parse_command(&request)?;
        response.push_str(&match_command(&command, &mut ctx, request, stream).await?);
    }

    let mut response = response.replace("OK\n", "list_OK\n");
    response.push_str("OK\n");
    stream.write_all(response.as_bytes()).await?;
    Ok(response)
}

pub async fn match_command(
    command: &str,
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    match command {
        "play" => handle_play(ctx, request, stream).await,
        "pause" => handle_pause(ctx, request, stream).await,
        "toggle" => handle_toggle(ctx, request, stream).await,
        "next" => handle_next(ctx, request, stream).await,
        "previous" => handle_previous(ctx, request, stream).await,
        "playid" => handle_playid(ctx, request, stream).await,
        "seek" => handle_seek(ctx, request, stream).await,
        "seekid" => handle_seekid(ctx, request, stream).await,
        "seekcur" => handle_seekcur(ctx, request, stream).await,
        "random" => handle_random(ctx, request, stream).await,
        "repeat" => handle_repeat(ctx, request, stream).await,
        "getvol" => handle_getvol(ctx, request, stream).await,
        "setvol" => handle_setvol(ctx, request, stream).await,
        "volume" => handle_setvol(ctx, request, stream).await,
        "single" => handle_single(ctx, request, stream).await,
        "shuffle" => handle_shuffle(ctx, request, stream).await,
        "add" => handle_add(ctx, request, stream).await,
        "playlistinfo" => handle_playlistinfo(ctx, request, stream).await,
        "delete" => handle_delete(ctx, request, stream).await,
        "clear" => handle_clear(ctx, request, stream).await,
        "move" => handle_move(ctx, request, stream).await,
        "list album" => handle_list_album(ctx, request, stream).await,
        "list artist" => handle_list_artist(ctx, request, stream).await,
        "list title" => handle_list_title(ctx, request, stream).await,
        "update" => handle_rescan(ctx, request, stream).await,
        "search" => handle_search(ctx, request, stream).await,
        "rescan" => handle_rescan(ctx, request, stream).await,
        "status" => handle_status(ctx, request, stream).await,
        "currentsong" => handle_currentsong(ctx, request, stream).await,
        "config" => handle_config(ctx, request, stream).await,
        "tagtypes " => handle_tagtypes(ctx, request, stream).await,
        "tagtypes clear" => handle_tagtypes_clear(ctx, request, stream).await,
        "tagtypes enable" => handle_tagtypes_enable(ctx, request, stream).await,
        "stats" => handle_stats(ctx, request, stream).await,
        "plchanges" => handle_playlistinfo(ctx, request, stream).await,
        "outputs" => handle_outputs(ctx, request, stream).await,
        "decoders" => handle_decoders(ctx, request, stream).await,
        "lsinfo" => handle_lsinfo(ctx, request, stream).await,
        "listall" => handle_listall(ctx, request, stream).await,
        "listallinfo" => handle_listallinfo(ctx, request, stream).await,
        "listfiles" => handle_listfiles(ctx, request, stream).await,
        _ => {
            println!("Unhandled command: {}", request);
            if !ctx.batch {
                stream
                    .write_all(b"ACK [5@0] {unhandled} unknown command\n")
                    .await?;
            }
            Ok("ACK [5@0] {unhandled} unknown command\n".to_string())
        }
    }
}
