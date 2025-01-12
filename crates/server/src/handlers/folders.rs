use crate::http::{Context, Request, Response};
use anyhow::Error;
use cuid::cuid1;
use rockbox_library::{entity, repo};
use rockbox_types::{Folder, FolderUpdate};
use serde_json::json;

pub async fn create_folder(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    if req.body.is_none() {
        res.set_status(400);
        return Ok(());
    }
    let body = req.body.as_ref().unwrap();
    let folder: Folder = serde_json::from_str(body)?;
    let id = repo::folder::save(
        ctx.pool.clone(),
        entity::folder::Folder {
            id: cuid1()?,
            name: folder.name,
            parent_id: folder.parent_id,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        },
    )
    .await?;
    res.json(&json!({ "id": id }));
    Ok(())
}

pub async fn get_folder(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let folder = repo::folder::find(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&folder);
    Ok(())
}

pub async fn get_folders(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let parent_id = req.query_params.get("parent_id");
    let parent_id = parent_id.map(|s| s.as_str().unwrap().to_string());
    let folders = repo::folder::find_by_parent(ctx.pool.clone(), parent_id).await?;
    res.json(&folders);
    Ok(())
}

pub async fn update_folder(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    if req.body.is_none() {
        res.set_status(400);
        return Ok(());
    }
    let body = req.body.as_ref().unwrap();
    let folder: FolderUpdate = serde_json::from_str(body)?;
    repo::folder::update(
        ctx.pool.clone(),
        entity::folder::Folder {
            id: req.params[0].clone(),
            name: folder.name.unwrap_or_default(),
            parent_id: folder.parent_id,
            ..Default::default()
        },
    )
    .await?;
    res.json(&json!({ "id": req.params[0] }));
    Ok(())
}

pub async fn delete_folder(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    repo::folder::delete(ctx.pool.clone(), &req.params[0]).await?;
    res.json(&json!({ "id": req.params[0] }));
    Ok(())
}
