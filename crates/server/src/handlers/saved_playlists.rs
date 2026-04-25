use crate::http::{Context, Request, Response};
use crate::PLAYER_MUTEX;
use anyhow::Error;
use rockbox_library::repo;
use rockbox_sys::{self as rb};
use serde::Deserialize;

#[derive(Deserialize)]
struct CreatePlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
    track_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct UpdatePlaylistBody {
    name: String,
    description: Option<String>,
    image: Option<String>,
    folder_id: Option<String>,
}

#[derive(Deserialize)]
struct AddTracksBody {
    track_ids: Vec<String>,
}

#[derive(Deserialize)]
struct CreateFolderBody {
    name: String,
}

pub async fn list_saved_playlists(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let folder_id = req
        .query_params
        .get("folder_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let playlists = match folder_id.as_deref() {
        Some(fid) if !fid.is_empty() => ctx.playlist_store.list_by_folder(fid).await?,
        _ => ctx.playlist_store.list().await?,
    };
    res.json(&playlists);
    Ok(())
}

pub async fn get_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    match ctx.playlist_store.get(id).await? {
        Some(p) => res.json(&p),
        None => res.set_status(404),
    }
    Ok(())
}

pub async fn create_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: CreatePlaylistBody = serde_json::from_str(body)?;
    if payload.name.is_empty() {
        res.set_status(400);
        return Ok(());
    }

    let playlist = ctx
        .playlist_store
        .create(
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
        )
        .await?;
    if let Some(ids) = payload.track_ids {
        if !ids.is_empty() {
            ctx.playlist_store.add_tracks(&playlist.id, &ids).await?;
        }
    }
    let playlist = ctx
        .playlist_store
        .get(&playlist.id)
        .await?
        .unwrap_or(playlist);
    res.set_status(201);
    res.json(&playlist);
    Ok(())
}

pub async fn update_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: UpdatePlaylistBody = serde_json::from_str(body)?;
    ctx.playlist_store
        .update(
            id,
            &payload.name,
            payload.description.as_deref(),
            payload.image.as_deref(),
            payload.folder_id.as_deref(),
        )
        .await?;
    res.set_status(204);
    Ok(())
}

pub async fn delete_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    ctx.playlist_store.delete(id).await?;
    res.set_status(204);
    Ok(())
}

pub async fn get_saved_playlist_tracks(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let track_ids = ctx.playlist_store.get_track_ids(playlist_id).await?;
    let mut tracks = Vec::with_capacity(track_ids.len());
    for id in &track_ids {
        if let Some(track) = repo::track::find(ctx.pool.clone(), id).await? {
            tracks.push(track);
        }
    }
    res.json(&tracks);
    Ok(())
}

pub async fn add_tracks_to_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: AddTracksBody = serde_json::from_str(body)?;
    ctx.playlist_store
        .add_tracks(playlist_id, &payload.track_ids)
        .await?;
    res.set_status(204);
    Ok(())
}

pub async fn get_saved_playlist_track_ids(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let track_ids = ctx.playlist_store.get_track_ids(playlist_id).await?;
    res.json(&track_ids);
    Ok(())
}

pub async fn remove_track_from_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let track_id = req.params.get(1).map(|s| s.as_str()).unwrap_or("");
    ctx.playlist_store
        .remove_track(playlist_id, track_id)
        .await?;
    res.set_status(204);
    Ok(())
}

pub async fn play_saved_playlist(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let playlist_id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    let track_ids = ctx.playlist_store.get_track_ids(playlist_id).await?;

    if track_ids.is_empty() {
        res.set_status(422);
        return Ok(());
    }

    let mut paths = Vec::with_capacity(track_ids.len());
    for id in &track_ids {
        if let Some(track) = repo::track::find(ctx.pool.clone(), id).await? {
            paths.push(track.path);
        }
    }

    if paths.is_empty() {
        res.set_status(422);
        return Ok(());
    }

    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let first = &paths[0];
    let dir = {
        let parts: Vec<_> = first.split('/').collect();
        parts[..parts.len().saturating_sub(1)].join("/")
    };
    rb::playlist::create(&dir, None);
    rb::playlist::build_playlist(
        paths.iter().map(|p| p.as_str()).collect(),
        0,
        paths.len() as i32,
    );
    rb::playlist::start(0, 0, 0);
    drop(player_mutex);

    res.set_status(204);
    Ok(())
}

// ── Folders ────────────────────────────────────────────────────────────────

pub async fn list_playlist_folders(
    ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let folders = ctx.playlist_store.list_folders().await?;
    res.json(&folders);
    Ok(())
}

pub async fn create_playlist_folder(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let body = match req.body.as_ref() {
        Some(b) => b,
        None => {
            res.set_status(400);
            return Ok(());
        }
    };
    let payload: CreateFolderBody = serde_json::from_str(body)?;
    if payload.name.is_empty() {
        res.set_status(400);
        return Ok(());
    }
    let folder = ctx.playlist_store.create_folder(&payload.name).await?;
    res.set_status(201);
    res.json(&folder);
    Ok(())
}

pub async fn delete_playlist_folder(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let id = req.params.first().map(|s| s.as_str()).unwrap_or("");
    ctx.playlist_store.delete_folder(id).await?;
    res.set_status(204);
    Ok(())
}
