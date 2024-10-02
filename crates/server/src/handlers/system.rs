use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;

pub async fn get_status(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let status = rb::system::get_global_status();
    res.json(&status);
    Ok(())
}

pub async fn get_rockbox_version(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let version = rb::system::get_rockbox_version();
    res.json(&version);
    Ok(())
}
