use anyhow::Error;
use tokio::{
    io::{AsyncWriteExt, BufReader},
    net::TcpStream,
};

use crate::{parse_command, setup_context, Context};

use super::{
    library::{
        handle_config, handle_list_album, handle_list_artist, handle_list_title, handle_rescan,
        handle_search, handle_tagtypes, handle_tagtypes_clear,
    },
    playback::{
        handle_currentsong, handle_getvol, handle_next, handle_pause, handle_play, handle_playid,
        handle_previous, handle_random, handle_repeat, handle_seek, handle_seekcur, handle_seekid,
        handle_setvol, handle_single, handle_status, handle_toggle,
    },
    queue::{
        handle_add, handle_clear, handle_delete, handle_move, handle_playlistinfo, handle_shuffle,
    },
};

pub async fn handle_command_list_begin(
    _ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();
    for request in commands {
        let command = parse_command(&request)?;
        response.push_str(&match command.as_str() {
            "play" => handle_play(&mut ctx, &request, stream).await?,
            "pause" => handle_pause(&mut ctx, &request, stream).await?,
            "toggle" => handle_toggle(&mut ctx, &request, stream).await?,
            "next" => handle_next(&mut ctx, &request, stream).await?,
            "previous" => handle_previous(&mut ctx, &request, stream).await?,
            "playid" => handle_playid(&mut ctx, &request, stream).await?,
            "seek" => handle_seek(&mut ctx, &request, stream).await?,
            "seekid" => handle_seekid(&mut ctx, &request, stream).await?,
            "seekcur" => handle_seekcur(&mut ctx, &request, stream).await?,
            "random" => handle_random(&mut ctx, &request, stream).await?,
            "repeat" => handle_repeat(&mut ctx, &request, stream).await?,
            "getvol" => handle_getvol(&mut ctx, &request, stream).await?,
            "setvol" => handle_setvol(&mut ctx, &request, stream).await?,
            "volume" => handle_setvol(&mut ctx, &request, stream).await?,
            "single" => handle_single(&mut ctx, &request, stream).await?,
            "shuffle" => handle_shuffle(&mut ctx, &request, stream).await?,
            "add" => handle_add(&mut ctx, &request, stream).await?,
            "playlistinfo" => handle_playlistinfo(&mut ctx, &request, stream).await?,
            "delete" => handle_delete(&mut ctx, &request, stream).await?,
            "clear" => handle_clear(&mut ctx, &request, stream).await?,
            "move" => handle_move(&mut ctx, &request, stream).await?,
            "list album" => handle_list_album(&mut ctx, &request, stream).await?,
            "list artist" => handle_list_artist(&mut ctx, &request, stream).await?,
            "list title" => handle_list_title(&mut ctx, &request, stream).await?,
            "update" => handle_rescan(&mut ctx, &request, stream).await?,
            "search" => handle_search(&mut ctx, &request, stream).await?,
            "rescan" => handle_rescan(&mut ctx, &request, stream).await?,
            "status" => handle_status(&mut ctx, &request, stream).await?,
            "currentsong" => handle_currentsong(&mut ctx, &request, stream).await?,
            "config" => handle_config(&mut ctx, &request, stream).await?,
            "tagtypes " => handle_tagtypes(&mut ctx, &request, stream).await?,
            "tagtypes clear" => handle_tagtypes_clear(&mut ctx, &request, stream).await?,
            _ => {
                println!("Unhandled command: {}", request);
                if !ctx.batch {
                    stream
                        .write_all(b"ACK [5@0] {unhandled} unknown command\n")
                        .await?;
                }
                "ACK [5@0] {unhandled} unknown command\n".to_string()
            }
        });
    }

    stream.write_all(response.as_bytes()).await?;

    Ok(response)
}

pub async fn handle_command_list_ok_begin(
    _ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let mut ctx = setup_context(true).await?;

    let commands: Vec<&str> = request
        .split("\n")
        .filter(|x| !vec!["command_list_ok_begin", "command_list_end", ""].contains(x))
        .collect();

    let mut response = String::new();

    for request in commands {
        let command = parse_command(&request)?;

        response.push_str(&match command.as_str() {
            "play" => handle_play(&mut ctx, &request, stream).await?,
            "pause" => handle_pause(&mut ctx, &request, stream).await?,
            "toggle" => handle_toggle(&mut ctx, &request, stream).await?,
            "next" => handle_next(&mut ctx, &request, stream).await?,
            "previous" => handle_previous(&mut ctx, &request, stream).await?,
            "playid" => handle_playid(&mut ctx, &request, stream).await?,
            "seek" => handle_seek(&mut ctx, &request, stream).await?,
            "seekid" => handle_seekid(&mut ctx, &request, stream).await?,
            "seekcur" => handle_seekcur(&mut ctx, &request, stream).await?,
            "random" => handle_random(&mut ctx, &request, stream).await?,
            "repeat" => handle_repeat(&mut ctx, &request, stream).await?,
            "getvol" => handle_getvol(&mut ctx, &request, stream).await?,
            "setvol" => handle_setvol(&mut ctx, &request, stream).await?,
            "volume" => handle_setvol(&mut ctx, &request, stream).await?,
            "single" => handle_single(&mut ctx, &request, stream).await?,
            "shuffle" => handle_shuffle(&mut ctx, &request, stream).await?,
            "add" => handle_add(&mut ctx, &request, stream).await?,
            "playlistinfo" => handle_playlistinfo(&mut ctx, &request, stream).await?,
            "delete" => handle_delete(&mut ctx, &request, stream).await?,
            "clear" => handle_clear(&mut ctx, &request, stream).await?,
            "move" => handle_move(&mut ctx, &request, stream).await?,
            "list album" => handle_list_album(&mut ctx, &request, stream).await?,
            "list artist" => handle_list_artist(&mut ctx, &request, stream).await?,
            "list title" => handle_list_title(&mut ctx, &request, stream).await?,
            "update" => handle_rescan(&mut ctx, &request, stream).await?,
            "search" => handle_search(&mut ctx, &request, stream).await?,
            "rescan" => handle_rescan(&mut ctx, &request, stream).await?,
            "status" => handle_status(&mut ctx, &request, stream).await?,
            "currentsong" => handle_currentsong(&mut ctx, &request, stream).await?,
            "config" => handle_config(&mut ctx, &request, stream).await?,
            "tagtypes " => handle_tagtypes(&mut ctx, &request, stream).await?,
            "tagtypes clear" => handle_clear(&mut ctx, &request, stream).await?,
            _ => {
                println!("Unhandled command: {}", request);
                if !ctx.batch {
                    stream
                        .write_all(b"ACK [5@0] {unhandled} unknown command\n")
                        .await?;
                }
                "ACK [5@0] {unhandled} unknown command\n".to_string()
            }
        });
    }

    let mut response = response.replace("OK\n", "list_OK\n");
    response.push_str("OK\n");
    stream.write_all(response.as_bytes()).await?;
    Ok(response)
}
