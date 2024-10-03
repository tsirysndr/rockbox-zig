use anyhow::Error;

use crate::http::{Context, Request, Response};

pub async fn get_openapi(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let spec = include_str!("../../openapi.json");
    res.add_header("Content-Type", "application/json");
    res.set_body(spec);
    Ok(())
}

pub async fn index(_ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let index = include_str!("../../docs/index.html");
    res.add_header("Content-Type", "text/html");
    res.set_body(index);
    Ok(())
}
