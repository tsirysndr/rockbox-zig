use anyhow::Error;
use tokio::sync::mpsc::Sender;

use crate::{parse_command, setup_context, Context};

use super::{
    browse::{handle_listall, handle_listallinfo, handle_listfiles, handle_lsinfo},
    library::{
        handle_config, handle_find_album, handle_find_artist, handle_find_title, handle_list_album,
        handle_list_artist, handle_list_title, handle_rescan, handle_search, handle_stats,
        handle_tagtypes, handle_tagtypes_clear, handle_tagtypes_enable,
    },
    playback::{
        handle_currentsong, handle_getvol, handle_next, handle_outputs, handle_pause, handle_play,
        handle_playid, handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur,
        handle_seekid, handle_setvol, handle_single, handle_status, handle_toggle,
    },
    queue::{
        handle_add, handle_addid, handle_clear, handle_delete, handle_move, handle_playlistinfo,
        handle_shuffle,
    },
    system::handle_decoders,
};

pub async fn handle_command_list_begin(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true, Some(ctx.clone())).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();
    for request in commands {
        let command = parse_command(&request)?;
        response.push_str(&match_command(&command, &mut ctx, request, tx.clone()).await?);
    }

    tx.send(response.clone()).await?;

    Ok(response)
}

pub async fn handle_command_list_ok_begin(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true, Some(ctx.clone())).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_ok_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();

    for request in commands {
        let command = parse_command(&request)?;
        response.push_str(&match_command(&command, &mut ctx, request, tx.clone()).await?);
    }

    let mut response = response.replace("OK\n", "list_OK\n");
    response.push_str("OK\n");
    tx.send(response.clone()).await?;
    Ok(response)
}

pub async fn match_command(
    command: &str,
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    match command {
        "play" => handle_play(ctx, request, tx.clone()).await,
        "pause" => handle_pause(ctx, request, tx.clone()).await,
        "toggle" => handle_toggle(ctx, request, tx.clone()).await,
        "next" => handle_next(ctx, request, tx.clone()).await,
        "previous" => handle_previous(ctx, request, tx.clone()).await,
        "playid" => handle_playid(ctx, request, tx.clone()).await,
        "seek" => handle_seek(ctx, request, tx.clone()).await,
        "seekid" => handle_seekid(ctx, request, tx.clone()).await,
        "seekcur" => handle_seekcur(ctx, request, tx.clone()).await,
        "random" => handle_random(ctx, request, tx.clone()).await,
        "repeat" => handle_repeat(ctx, request, tx.clone()).await,
        "getvol" => handle_getvol(ctx, request, tx.clone()).await,
        "setvol" => handle_setvol(ctx, request, tx.clone()).await,
        "volume" => handle_setvol(ctx, request, tx.clone()).await,
        "single" => handle_single(ctx, request, tx.clone()).await,
        "shuffle" => handle_shuffle(ctx, request, tx.clone()).await,
        "add" => handle_add(ctx, request, tx.clone()).await,
        "addid" => handle_addid(ctx, request, tx.clone()).await,
        "playlistinfo" => handle_playlistinfo(ctx, request, tx.clone()).await,
        "delete" => handle_delete(ctx, request, tx.clone()).await,
        "clear" => handle_clear(ctx, request, tx.clone()).await,
        "move" => handle_move(ctx, request, tx.clone()).await,
        "list album" => handle_list_album(ctx, request, tx.clone()).await,
        "list artist" => handle_list_artist(ctx, request, tx.clone()).await,
        "list title" => handle_list_title(ctx, request, tx.clone()).await,
        "update" => handle_rescan(ctx, request, tx.clone()).await,
        "search" => handle_search(ctx, request, tx.clone()).await,
        "rescan" => handle_rescan(ctx, request, tx.clone()).await,
        "status" => handle_status(ctx, request, tx.clone()).await,
        "currentsong" => handle_currentsong(ctx, request, tx.clone()).await,
        "config" => handle_config(ctx, request, tx.clone()).await,
        "tagtypes " => handle_tagtypes(ctx, request, tx.clone()).await,
        "tagtypes clear" => handle_tagtypes_clear(ctx, request, tx.clone()).await,
        "tagtypes enable" => handle_tagtypes_enable(ctx, request, tx.clone()).await,
        "stats" => handle_stats(ctx, request, tx.clone()).await,
        "plchanges" => handle_playlistinfo(ctx, request, tx.clone()).await,
        "outputs" => handle_outputs(ctx, request, tx.clone()).await,
        "decoders" => handle_decoders(ctx, request, tx.clone()).await,
        "lsinfo" => handle_lsinfo(ctx, request, tx.clone()).await,
        "listall" => handle_listall(ctx, request, tx.clone()).await,
        "listallinfo" => handle_listallinfo(ctx, request, tx.clone()).await,
        "listfiles" => handle_listfiles(ctx, request, tx.clone()).await,
        "find artist" => handle_find_artist(ctx, request, tx.clone()).await,
        "find album" => handle_find_album(ctx, request, tx.clone()).await,
        "find title" => handle_find_title(ctx, request, tx.clone()).await,
        _ => {
            println!("Unhandled command: {}", request);
            if !ctx.batch {
                tx.send("ACK [5@0] {unhandled} unknown command\n".to_string())
                    .await?;
            }
            Ok("ACK [5@0] {unhandled} unknown command\n".to_string())
        }
    }
}
