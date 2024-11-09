use anyhow::Error;
use rockbox_rpc::api::rockbox::v1alpha1::{
    AdjustVolumeRequest, CurrentTrackRequest, GetCurrentRequest, GetGlobalSettingsRequest,
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
) -> Result<String, Error> {
    ctx.playback.resume(ResumeRequest {}).await?;

    if !ctx.batch {
        stream.write_all(b"OK\n").await?;
    }

    Ok("OK\n".to_string())
}

pub async fn handle_pause(
    ctx: &mut Context,
    request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
    let arg = request.split_whitespace().nth(1);
    match arg {
        Some(r#""0""#) => {
            ctx.playback.resume(ResumeRequest {}).await?;
            if !ctx.batch {
                stream.write_all(b"OK\n").await?;
            }
        }
        Some(r#""1""#) => {
            ctx.playback.pause(PauseRequest {}).await?;
            if !ctx.batch {
                stream.write_all(b"OK\n").await?;
            }
        }
        _ => {
            stream
                .write_all(b"ACK [2@0] {pause} incorrect arguments\n")
                .await?;
        }
    }
    Ok("OK\n".to_string())
}

pub async fn handle_toggle(
    ctx: &mut Context,
    _request: &str,
    stream: &mut BufReader<TcpStream>,
) -> Result<String, Error> {
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
    let response = ctx.playback.status(StatusRequest {}).await?;
    let response = response.into_inner();
    let status = match response.status {
        1 => "play",
        3 => "pause",
        _ => "stop",
    };

    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let repeat = match response.repeat_mode {
        0 => 0,
        1 => 1,
        2 => 1,
        _ => 0,
    };

    let random = match response.playlist_shuffle {
        true => 1,
        false => 0,
    };

    let volume = response.volume;
    // volume is between -80 db and 0 db
    // we need to convert it to 0-100
    // -80 db is 0
    // 0 db is 100
    let volume = ((volume + 80) * 100 / 80).max(0).min(100);

    let response = ctx.playback.current_track(CurrentTrackRequest {}).await?;
    let response = response.into_inner();

    let time = format!(
        "{}:{}",
        (response.elapsed / 1000) as i64,
        (response.length / 1000) as i64
    );
    let elapsed = (response.elapsed / 1000) as i64;

    let single = ctx.single.lock().await;
    let single = single.as_str().replace("\"", "");

    let response = ctx.playlist.get_current(GetCurrentRequest {}).await?;
    let response = response.into_inner();
    let playlistlength = response.amount;
    let song = response.index;

    let response = format!(
        "state: {}\nrepeat: {}\nsingle: {}\nrandom: {}\ntime: {}\nelapsed: {}\nplaylistlength: {}\nsong: {}\nvolume: {}\nOK\n",
        status, repeat, single, random, time, elapsed, playlistlength, song, volume
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
    println!("{}", request);

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
                .map(|x| x.replace("\"", ""))
                .map(|x| x.parse::<i64>().unwrap() * 1000)
                .unwrap_or_default(),
            offset: 0,
        })
        .await?;
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
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let volume = response.volume;
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
    let response = ctx
        .settings
        .get_global_settings(GetGlobalSettingsRequest {})
        .await?;
    let response = response.into_inner();
    let volume = response.volume as i32;
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
    let response = ctx.playback.current_track(CurrentTrackRequest {}).await?;
    let response = response.into_inner();
    let response = format!(
        "file: {}\nTitle: {}\nArtist: {}\nAlbum: {}\nTrack: {}\nDate: {}\nTime: {}\nOK\n",
        response.path,
        response.title,
        response.artist,
        response.album,
        response.tracknum,
        response.year,
        (response.elapsed / 1000) as i64
    );
    if !ctx.batch {
        stream.write_all(response.as_bytes()).await?;
    }
    Ok(response)
}
