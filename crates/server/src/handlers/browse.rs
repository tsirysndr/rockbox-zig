use anyhow::Error;

use crate::http::{Context, Request, Response};

pub async fn get_tree_entries(
    ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    Ok(())
}
