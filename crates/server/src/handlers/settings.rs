use crate::http::{Context, Request, Response};
use crate::PLAYER_MUTEX;
use anyhow::Error;
use rockbox_sys as rb;
use rockbox_sys::types::user_settings::NewGlobalSettings;

pub async fn get_global_settings(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let settings = rb::settings::get_global_settings();
    res.json(&settings);
    drop(player_mutex);
    Ok(())
}

pub async fn update_global_settings(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let player_mutex = PLAYER_MUTEX.lock().unwrap();
    let body = req.body.as_ref().unwrap();
    let settings: NewGlobalSettings = serde_json::from_str(body)?;
    rockbox_settings::load_settings(Some(settings))?;
    rockbox_settings::write_settings()?;
    res.set_status(204);
    drop(player_mutex);
    Ok(())
}
