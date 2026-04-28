use anyhow::Error;
use tokio::sync::mpsc::Sender;

use crate::{parse_command, setup_context, Context};

use super::{
    browse::{handle_listall, handle_listallinfo, handle_listfiles, handle_lsinfo},
    library::{
        handle_config, handle_count, handle_find_album, handle_find_artist, handle_find_title,
        handle_findadd, handle_list_album, handle_list_artist, handle_list_date, handle_list_genre,
        handle_list_title, handle_listplaylists, handle_load, handle_rename, handle_rescan,
        handle_rm, handle_save, handle_search, handle_searchadd, handle_stats, handle_tagtypes,
        handle_tagtypes_clear, handle_tagtypes_enable,
    },
    playback::{
        handle_consume, handle_currentsong, handle_disableoutput, handle_enableoutput,
        handle_getvol, handle_next, handle_outputs, handle_pause, handle_play, handle_playid,
        handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur, handle_seekid,
        handle_setvol, handle_single, handle_status, handle_stop, handle_toggle,
        handle_toggleoutput,
    },
    queue::{
        handle_add, handle_addid, handle_clear, handle_delete, handle_move, handle_moveid,
        handle_playlistid, handle_playlistinfo, handle_shuffle, handle_swap, handle_swapid,
    },
    system::{handle_decoders, handle_notcommands, handle_ping, handle_urlhandlers},
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
        "stop" => handle_stop(ctx, request, tx.clone()).await,
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
        "consume" => handle_consume(ctx, request, tx.clone()).await,
        "getvol" => handle_getvol(ctx, request, tx.clone()).await,
        "setvol" => handle_setvol(ctx, request, tx.clone()).await,
        "volume" => handle_setvol(ctx, request, tx.clone()).await,
        "single" => handle_single(ctx, request, tx.clone()).await,
        "shuffle" => handle_shuffle(ctx, request, tx.clone()).await,
        "add" => handle_add(ctx, request, tx.clone()).await,
        "addid" => handle_addid(ctx, request, tx.clone()).await,
        "playlistinfo" => handle_playlistinfo(ctx, request, tx.clone()).await,
        "playlistid" => handle_playlistid(ctx, request, tx.clone()).await,
        "plchanges" => handle_playlistinfo(ctx, request, tx.clone()).await,
        "delete" => handle_delete(ctx, request, tx.clone()).await,
        "clear" => handle_clear(ctx, request, tx.clone()).await,
        "move" => handle_move(ctx, request, tx.clone()).await,
        "moveid" => handle_moveid(ctx, request, tx.clone()).await,
        "swap" => handle_swap(ctx, request, tx.clone()).await,
        "swapid" => handle_swapid(ctx, request, tx.clone()).await,
        "list album" => handle_list_album(ctx, request, tx.clone()).await,
        "list albumartist" => handle_list_artist(ctx, request, tx.clone()).await,
        "list artist" => handle_list_artist(ctx, request, tx.clone()).await,
        "list title" => handle_list_title(ctx, request, tx.clone()).await,
        "list genre" => handle_list_genre(ctx, request, tx.clone()).await,
        "list date" => handle_list_date(ctx, request, tx.clone()).await,
        "update" => handle_rescan(ctx, request, tx.clone()).await,
        "search" => handle_search(ctx, request, tx.clone()).await,
        "searchadd" => handle_searchadd(ctx, request, tx.clone()).await,
        "rescan" => handle_rescan(ctx, request, tx.clone()).await,
        "count" => handle_count(ctx, request, tx.clone()).await,
        "findadd" => handle_findadd(ctx, request, tx.clone()).await,
        "status" => handle_status(ctx, request, tx.clone()).await,
        "currentsong" => handle_currentsong(ctx, request, tx.clone()).await,
        "config" => handle_config(ctx, request, tx.clone()).await,
        "tagtypes " => handle_tagtypes(ctx, request, tx.clone()).await,
        "tagtypes clear" => handle_tagtypes_clear(ctx, request, tx.clone()).await,
        "tagtypes enable" => handle_tagtypes_enable(ctx, request, tx.clone()).await,
        "stats" => handle_stats(ctx, request, tx.clone()).await,
        "outputs" => handle_outputs(ctx, request, tx.clone()).await,
        "enableoutput" => handle_enableoutput(ctx, request, tx.clone()).await,
        "disableoutput" => handle_disableoutput(ctx, request, tx.clone()).await,
        "toggleoutput" => handle_toggleoutput(ctx, request, tx.clone()).await,
        "decoders" => handle_decoders(ctx, request, tx.clone()).await,
        "lsinfo" => handle_lsinfo(ctx, request, tx.clone()).await,
        "listall" => handle_listall(ctx, request, tx.clone()).await,
        "listallinfo" => handle_listallinfo(ctx, request, tx.clone()).await,
        "listfiles" => handle_listfiles(ctx, request, tx.clone()).await,
        "listplaylists" => handle_listplaylists(ctx, request, tx.clone()).await,
        "load" => handle_load(ctx, request, tx.clone()).await,
        "save" => handle_save(ctx, request, tx.clone()).await,
        "rm" => handle_rm(ctx, request, tx.clone()).await,
        "rename" => handle_rename(ctx, request, tx.clone()).await,
        "find artist" => handle_find_artist(ctx, request, tx.clone()).await,
        "find album" => handle_find_album(ctx, request, tx.clone()).await,
        "find title" => handle_find_title(ctx, request, tx.clone()).await,
        "ping" => handle_ping(ctx, request, tx.clone()).await,
        "notcommands" => handle_notcommands(ctx, request, tx.clone()).await,
        "urlhandlers" => handle_urlhandlers(ctx, request, tx.clone()).await,
        _ => {
            if !ctx.batch {
                tx.send("ACK [5@0] {unhandled} unknown command\n".to_string())
                    .await?;
            }
            Ok("ACK [5@0] {unhandled} unknown command\n".to_string())
        }
    }
}
