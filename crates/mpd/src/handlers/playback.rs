use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{
    AdjustVolumeRequest, NextRequest, PauseRequest, PlayRequest, PreviousRequest, ResumeRequest,
    SaveSettingsRequest, StartRequest,
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
) -> Result<String, Error> {
    ctx.playback.resume(ResumeRequest {}).await?;
    match ctx.event_sender.send("player".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_pause(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let playback_status = ctx.playback_status.lock().await;
    let status = playback_status.as_ref().map(|x| x.status);

    match status {
        Some(1) => {
            ctx.playback.pause(PauseRequest {}).await?;
        }
        Some(3) => {
            ctx.playback.resume(ResumeRequest {}).await?;
        }
        _ => {
            stream
                .write_all(b"ACK [2@0] {pause} no song is playing\n")
                .await?;
        }
    }

    match ctx.event_sender.send("player".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    Ok("OK\n".to_string())
}

pub async fn handle_toggle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let playback_status = ctx.playback_status.lock().await;
    let playback_status = playback_status.as_ref().map(|x| x.status);

    match playback_status {
        Some(1) => {
            ctx.playback.pause(PauseRequest {}).await?;
        }
        Some(3) => {
            ctx.playback.resume(ResumeRequest {}).await?;
        }
        _ => {
            stream
                .write_all(b"ACK [2@0] {toggle} no song is playing\n")
                .await?;
        }
    }
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_status(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
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

    let volume = settings.volume;
    // volume is between -80 db and 0 db
    // we need to convert it to 0-100
    // -80 db is 0
    // 0 db is 100
    let volume = ((volume + 80) * 100 / 80).max(0).min(100);

    let current_track = ctx.current_track.lock().await;

    if current_track.is_none() {
        let response = format!(
            "state: {}\nrepeat: {}\nsingle: 0\nrandom: {}\ntime: 0:0\nelapsed: 0\nplaylistlength: 0\nvolume: {}\naudio: 0:16:2\nbitrate: 0\nOK\n",
            status, repeat, random, volume,
        );
        if !ctx.batch {
            stream.write_all(response.as_bytes()).await?;
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

    let single = ctx.single.lock().await;
    let single = single.as_str().replace("\"", "");
    let bitrate = current_track.bitrate;
    let audio = format!("{}:16:2", current_track.frequency);

    let current_playlist = ctx.current_playlist.lock().await;
    if current_playlist.is_none() {
        let response = format!(
            "state: {}\nrepeat: {}\nsingle: {}\nrandom: {}\ntime: {}\nelapsed: {}\nplaylistlength: 0\nsong: 0\nvolume: {}\naudio: {}\nbitrate: {}\nOK\n",
            status, repeat, single, random, time, elapsed, volume, audio, bitrate,
        );
        if !ctx.batch {
            stream.write_all(response.as_bytes()).await?;
        }
        return Ok(response);
    }

    let current_playlist = current_playlist.as_ref().unwrap();
    let playlistlength = current_playlist.amount;
    let song = current_playlist.index;

    let response = format!(
        "state: {}\nrepeat: {}\nsingle: {}\nrandom: {}\ntime: {}\nelapsed: {}\nplaylist: {}\nplaylistlength: {}\nsong: {}\nsongid: {}\nvolume: {}\naudio: {}\nbitrate: {}\nnextsong: {}\nnextsongid: {}\nOK\n",
        status, repeat, single, random, time, elapsed, playlistlength + 1, playlistlength, song, song + 1, volume, audio, bitrate,
        song + 1, song + 2,
    );

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_next(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    ctx.playback.next(NextRequest {}).await?;
    match ctx.event_sender.send("player".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_previous(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    ctx.playback.previous(PreviousRequest {}).await?;
    match ctx.event_sender.send("player".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_playid(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);

    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {playid} incorrect arguments\n")
            .await?;
        return Ok("ACK [2@0] {playid} incorrect arguments\n".to_string());
    }

    let arg = arg.unwrap();
    let arg = arg.trim();
    let arg = arg.trim_matches('"');
    let arg = arg.parse::<i32>();

    if arg.is_err() {
        stream
            .write_all(b"ACK [2@0] {playid} incorrect arguments\n")
            .await?;
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
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_seek(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    // TODO: Implement seek
    println!("{}", request);

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_seekid(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    // TODO: Implement seekid
    println!("{}", request);

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_seekcur(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        stream
            .write_all(b"ACK [2@0] {seekcur} incorrect arguments\n")
            .await?;
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

    match ctx.event_sender.send("player".to_string()) {
        Ok(_) => {}
        Err(_) => {}
    }

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_random(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {random} incorrect arguments\n")
                .await?;
        }
        return Ok("ACK [2@0] {random} incorrect arguments\n".to_string());
    }

    ctx.settings
        .save_settings(SaveSettingsRequest {
            playlist_shuffle: Some(arg.unwrap() == r#""1""#),
            ..Default::default()
        })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_repeat(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {repeat} incorrect arguments\n")
                .await?;
        }
        return Ok("ACK [2@0] {repeat} incorrect arguments\n".to_string());
    }

    let single = ctx.single.lock().await;

    let repeat_mode = match arg.unwrap() {
        r#""0""# => Some(0),
        r#""1""# => match single.as_str() {
            r#""1""# => Some(2),
            _ => Some(1),
        },
        _ => {
            if !ctx.batch {
                stream
                    .write_all(b"ACK [2@0] {repeat} incorrect arguments\n")
                    .await?;
            }
            return Ok("ACK [2@0] {repeat} incorrect arguments\n".to_string());
        }
    };
    ctx.settings
        .save_settings(SaveSettingsRequest {
            repeat_mode,
            ..Default::default()
        })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_getvol(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let settings = rockbox_sys::settings::get_global_settings();
    let volume = settings.volume;
    // volume is between -80 db and 0 db
    // we need to convert it to 0-100
    // -80 db is 0
    // 0 db is 100
    let volume = ((volume + 80) * 100 / 80).max(0).min(100);
    let response = format!("volume: {}\nOK\n", volume);

    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }

    Ok(response)
}

pub async fn handle_setvol(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let settings = rockbox_sys::settings::get_global_settings();
    let volume = settings.volume as i32;
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {setvol} incorrect arguments\n")
                .await?;
        }
        return Ok("ACK [2@0] {setvol} incorrect arguments\n".to_string());
    }

    let new_volume = arg.unwrap().replace("\"", "").parse::<i64>().unwrap();
    // volume is between 0 and 100
    // we need to convert it to -80 db to 0 db
    // 0 is -80 db
    // 100 is 0 db
    let new_volume = ((new_volume * 80 / 100) - 80) as i32;
    let steps = new_volume - volume;
    ctx.sound
        .adjust_volume(AdjustVolumeRequest { steps })
        .await?;
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_single(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    if arg.is_none() {
        if !ctx.batch {
            stream
                .write_all(b"ACK [2@0] {single} incorrect arguments\n")
                .await?;
        }
        return Ok("ACK [2@0] {single} incorrect arguments\n".to_string());
    }

    let mut single = ctx.single.lock().await;
    *single = arg.unwrap().to_string();
    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_currentsong(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let current = ctx.current_track.lock().await;
    if current.is_none() {
        let response = "OK\n".to_string();
        if !ctx.batch {
            stream.write_all(response.as_bytes()).await?;
        }
        return Ok(response);
    }
    let current = current.as_ref().unwrap();
    let current_playlist = ctx.current_playlist.lock().await;

    if current_playlist.is_none() {
        let response = format!(
            "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nDate: {}\nTime: {}\nPos: 0\nOK\n",
            current.path,
            current.title,
            current.artist,
            current.album,
            current.tracknum,
            current.year,
            (current.elapsed / 1000) as i64,
        );
        if !ctx.batch {
            stream.write_all(response.as_bytes()).await?;
        }
        return Ok(response);
    }

    let current_playlist = current_playlist.as_ref().unwrap();
    let response = format!(
        "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nDate: {}\nTime: {}\nPos: {}\nOK\n",
        current.path,
        current.title,
        current.artist,
        current.album,
        current.tracknum,
        current.year,
        (current.elapsed / 1000) as i64,
        current_playlist.index,
    );
    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_outputs(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let response =
        "outputid: 0\noutputname: default detected output\nplugin: pulse\noutputenabled: 1\nOK\n"
            .to_string();
    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}
