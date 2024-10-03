use crate::http::{Context, Request, Response};
use anyhow::Error;
use rockbox_sys as rb;

pub async fn get_tree_entries(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let path = match req.query_params.get("q") {
        Some(path) => path.as_str().unwrap_or("/"),
        None => "/",
    };
    if let Err(e) = rb::browse::rockbox_browse_at(path) {
        if e.to_string().starts_with("No such file or directory") {
            res.set_status(404);
            return Ok(());
        }
        res.set_status(500);
        return Ok(());
    }

    let mut entries = vec![];
    let context = rb::browse::tree_get_context();

    for i in 0..context.filesindir {
        let entry = rb::browse::tree_get_entry_at(i);
        entries.push(entry);
    }

    res.json(&entries);
    Ok(())
}
