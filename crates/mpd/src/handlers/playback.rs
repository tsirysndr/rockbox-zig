use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{
    AdjustVolumeRequest, HardStopRequest, NextRequest, PauseRequest, PlayRequest, PreviousRequest,
    ResumeRequest, SaveSettingsRequest, StartRequest,
};
use tokio::sync::mpsc::Sender;

use crate::Context;

use super::Subsystem;

pub async fn handle_play(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);

    if let Some(pos) = arg {
        if let Ok(pos) = pos.trim_matches('"').parse::<i32>() {
            ctx.playlist
                .start(StartRequest {
                    start_index: Some(pos),
                    ..Default::default()
                })
                .await?;
        } else {
            ctx.playback.resume(ResumeRequest {}).await?;
        }
    } else {
        ctx.playback.resume(ResumeRequest {}).await?;
    }

    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_stop(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    ctx.playback.hard_stop(HardStopRequest {}).await?;
    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_pause(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    let playback_status = ctx.playback_status.lock().await;
    let status = playback_status.as_ref().map(|x| x.status);
    drop(playback_status);

    match arg {
        Some(a) if a.trim_matches('"') == "1" => {
            if status == Some(1) {
                ctx.playback.pause(PauseRequest {}).await?;
            }
        }
        Some(a) if a.trim_matches('"') == "0" => {
            if status == Some(3) {
                ctx.playback.resume(ResumeRequest {}).await?;
            }
        }
        _ => match status {
            Some(1) => {
                ctx.playback.pause(PauseRequest {}).await?;
            }
            Some(3) => {
                ctx.playback.resume(ResumeRequest {}).await?;
            }
            _ => {
                if !ctx.batch {
                    tx.send("ACK [2@0] {pause} no song is playing\n".to_string())
                        .await?;
                }
                return Ok("ACK [2@0] {pause} no song is playing\n".to_string());
            }
        },
    }

    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_toggle(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let playback_status = {
        let guard = ctx.playback_status.lock().await;
        guard.as_ref().map(|x| x.status)
    };

    match playback_status {
        Some(1) => {
            ctx.playback.pause(PauseRequest {}).await?;
        }
        Some(3) => {
            ctx.playback.resume(ResumeRequest {}).await?;
        }
        _ => {
            if !ctx.batch {
                tx.send("ACK [2@0] {toggle} no song is playing\n".to_string())
                    .await?;
            }
            return Ok("ACK [2@0] {toggle} no song is playing\n".to_string());
        }
    }
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_status(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let playback_status = ctx.playback_status.lock().await;
    let playback_status = playback_status.as_ref().map(|x| x.status);

    let status = match playback_status {
        Some(1) => "play",
        Some(3) => "pause",
        _ => "stop",
    };

    let settings = ctx.current_settings.lock().await;
    let repeat = match settings.repeat_mode {
        0 => 0,
        1 => 1,
        2 => 1,
        _ => 0,
    };

    let random = match settings.playlist_shuffle {
        true => 1,
        false => 0,
    };
    drop(settings);

    let system_status = rockbox_sys::system::get_global_status();
    let volume = system_status.volume;
    let volume = ((volume + 80) * 100 / 80).max(0).min(100);

    let current_track = ctx.current_track.lock().await;
    let consume = ctx.consume.lock().await;
    let consume_val = if *consume { 1 } else { 0 };
    drop(consume);

    if current_track.is_none() {
        let response = format!(
            "volume: {}\nrepeat: {}\nrandom: {}\nsingle: 0\nconsume: {}\nplaylist: 0\nplaylistlength: 0\nstate: {}\nOK\n",
            volume, repeat, random, consume_val, status,
        );
        if !ctx.batch {
            tx.send(response.clone()).await?;
        }
        return Ok(response);
    }

    let current_track = current_track.as_ref().unwrap();

    let time = format!(
        "{}:{}",
        (current_track.elapsed / 1000) as i64,
        (current_track.length / 1000) as i64
    );
    let elapsed = (current_track.elapsed / 1000) as i64;
    let duration = (current_track.length / 1000) as i64;

    let single = ctx.single.lock().await;
    let single = single.as_str().replace("\"", "");
    let bitrate = current_track.bitrate;
    let audio = format!("{}:16:2", current_track.frequency);

    let current_playlist = ctx.current_playlist.lock().await;
    if current_playlist.is_none() {
        let response = format!(
            "volume: {}\nrepeat: {}\nrandom: {}\nsingle: {}\nconsume: {}\nplaylist: 0\nplaylistlength: 0\nstate: {}\nsong: 0\nelapsed: {}\ntime: {}\nduration: {}\naudio: {}\nbitrate: {}\nOK\n",
            volume, repeat, random, single, consume_val, status, elapsed, time, duration, audio, bitrate,
        );
        if !ctx.batch {
            tx.send(response.clone()).await?;
        }
        return Ok(response);
    }

    let current_playlist = current_playlist.as_ref().unwrap();
    let playlistlength = current_playlist.amount;
    let song = current_playlist.index;

    let response = format!(
        "volume: {}\nrepeat: {}\nrandom: {}\nsingle: {}\nconsume: {}\nplaylist: {}\nplaylistlength: {}\nstate: {}\nsong: {}\nsongid: {}\nnextsong: {}\nnextsongid: {}\ntime: {}\nelapsed: {}\nduration: {}\naudio: {}\nbitrate: {}\nOK\n",
        volume, repeat, random, single, consume_val, playlistlength, playlistlength, status,
        song, song + 1, song + 1, song + 2,
        time, elapsed, duration, audio, bitrate,
    );

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_next(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    ctx.playback.next(NextRequest {}).await?;
    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_previous(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    ctx.playback.previous(PreviousRequest {}).await?;
    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_playid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);

    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {playid} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {playid} incorrect arguments\n".to_string());
    }

    let arg = arg.unwrap();
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let arg = arg.parse::<i32>();

    if arg.is_err() {
        if !ctx.batch {
            tx.send("ACK [2@0] {playid} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {playid} incorrect arguments\n".to_string());
    }

    let arg = arg.unwrap();

    ctx.playlist
        .start(StartRequest {
            start_index: Some(arg - 1),
            ..Default::default()
        })
        .await?;

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_seek(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let songpos = parts.next();
    let time = parts.next();

    match (songpos, time) {
        (Some(pos), Some(t)) => {
            let pos = pos.trim_matches('"').parse::<i32>().unwrap_or(0);
            let t_secs = t.trim_matches('"').parse::<i32>().unwrap_or(0);
            ctx.playlist
                .start(StartRequest {
                    start_index: Some(pos),
                    elapsed: Some(t_secs * 1000),
                    ..Default::default()
                })
                .await?;
            match ctx.event_sender.send(Subsystem::Player) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        _ => {
            if !ctx.batch {
                tx.send("ACK [2@0] {seek} incorrect arguments\n".to_string())
                    .await?;
            }
            return Ok("ACK [2@0] {seek} incorrect arguments\n".to_string());
        }
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_seekid(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let mut parts = request.split_whitespace().skip(1);
    let songid = parts.next();
    let time = parts.next();

    match (songid, time) {
        (Some(id), Some(t)) => {
            let id = id.trim_matches('"').parse::<i32>().unwrap_or(1);
            let t_secs = t.trim_matches('"').parse::<i32>().unwrap_or(0);
            ctx.playlist
                .start(StartRequest {
                    start_index: Some(id - 1),
                    elapsed: Some(t_secs * 1000),
                    ..Default::default()
                })
                .await?;
            match ctx.event_sender.send(Subsystem::Player) {
                Ok(_) => {}
                Err(_) => {}
            }
        }
        _ => {
            if !ctx.batch {
                tx.send("ACK [2@0] {seekid} incorrect arguments\n".to_string())
                    .await?;
            }
            return Ok("ACK [2@0] {seekid} incorrect arguments\n".to_string());
        }
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_seekcur(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {seekcur} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {seekcur} incorrect arguments\n".to_string());
    }

    ctx.playback
        .play(PlayRequest {
            elapsed: arg
                .map(|x| x.trim_matches('"'))
                .map(|x| x.parse::<i64>().unwrap() * 1000)
                .unwrap_or_default(),
            offset: 0,
        })
        .await?;

    match ctx.event_sender.send(Subsystem::Player) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_random(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {random} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {random} incorrect arguments\n".to_string());
    }

    ctx.settings
        .save_settings(SaveSettingsRequest {
            playlist_shuffle: Some(arg.unwrap().trim_matches('"') == "1"),
            ..Default::default()
        })
        .await?;
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_repeat(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {repeat} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {repeat} incorrect arguments\n".to_string());
    }

    let single = ctx.single.lock().await;

    let repeat_mode = match arg.unwrap().trim_matches('"') {
        "0" => Some(0),
        "1" => match single.as_str().trim_matches('"') {
            "1" => Some(2),
            _ => Some(1),
        },
        _ => {
            if !ctx.batch {
                tx.send("ACK [2@0] {repeat} incorrect arguments\n".to_string())
                    .await?;
            }
            return Ok("ACK [2@0] {repeat} incorrect arguments\n".to_string());
        }
    };
    drop(single);
    ctx.settings
        .save_settings(SaveSettingsRequest {
            repeat_mode,
            ..Default::default()
        })
        .await?;
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_consume(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {consume} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {consume} incorrect arguments\n".to_string());
    }
    let mut consume = ctx.consume.lock().await;
    *consume = arg.unwrap().trim_matches('"') == "1";
    drop(consume);
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_getvol(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let status = rockbox_sys::system::get_global_status();
    let volume = status.volume;
    let volume = ((volume + 80) * 100 / 80).max(0).min(100);
    let response = format!("volume: {}\nOK\n", volume);

    if !ctx.batch {
        tx.send(response.clone()).await?;
    }

    Ok(response)
}

pub async fn handle_setvol(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let status = rockbox_sys::system::get_global_status();
    let volume = status.volume;
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {setvol} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {setvol} incorrect arguments\n".to_string());
    }

    let new_volume = arg.unwrap().replace("\"", "").parse::<i64>().unwrap();
    let new_volume = ((new_volume * 80 / 100) - 80) as i32;
    let steps = new_volume - volume;
    ctx.sound
        .adjust_volume(AdjustVolumeRequest { steps })
        .await?;
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_single(
    ctx: &mut Context,
    request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            tx.send("ACK [2@0] {single} incorrect arguments\n".to_string())
                .await?;
        }
        return Ok("ACK [2@0] {single} incorrect arguments\n".to_string());
    }

    let mut single = ctx.single.lock().await;
    *single = arg.unwrap().to_string();
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_currentsong(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let current = ctx.current_track.lock().await;
    if current.is_none() {
        let response = "OK\n".to_string();
        if !ctx.batch {
            tx.send(response.clone()).await?;
        }
        return Ok(response);
    }
    let current = current.as_ref().unwrap();
    let current_playlist = ctx.current_playlist.lock().await;

    if current_playlist.is_none() {
        let response = format!(
            "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nDate: {}\nTime: {}\nDuration: {}\nPos: 0\nId: 1\nOK\n",
            current.path,
            current.title,
            current.artist,
            current.album,
            current.tracknum,
            current.year,
            (current.length / 1000) as i64,
            (current.length / 1000) as i64,
        );
        if !ctx.batch {
            tx.send(response.clone()).await?;
        }
        return Ok(response);
    }

    let current_playlist = current_playlist.as_ref().unwrap();
    let pos = current_playlist.index;
    let response = format!(
        "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nDate: {}\nTime: {}\nDuration: {}\nPos: {}\nId: {}\nOK\n",
        current.path,
        current.title,
        current.artist,
        current.album,
        current.tracknum,
        current.year,
        (current.length / 1000) as i64,
        (current.length / 1000) as i64,
        pos,
        pos + 1,
    );
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_outputs(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let response =
        "outputid: 0\noutputname: default detected output\nplugin: pulse\noutputenabled: 1\nOK\n"
            .to_string();
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_enableoutput(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_disableoutput(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_toggleoutput(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}
