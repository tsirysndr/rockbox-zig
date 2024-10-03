use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;

pub async fn get_global_settings(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let settings = rb::settings::get_global_settings();
    res.json(&settings);
    Ok(())
}
