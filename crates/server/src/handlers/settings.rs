use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;
use rockbox_sys::types::user_settings::NewGlobalSettings;

pub async fn get_global_settings(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let settings = rb::settings::get_global_settings();
    res.json(&settings);
    Ok(())
}

pub async fn update_global_settings(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let body = req.body.as_ref().unwrap();
    let settings: NewGlobalSettings = serde_json::from_str(&body)?;
    rb::settings::save_settings(settings);
    rb::settings::apply_settings(false);
    res.set_status(204);
    Ok(())
}
