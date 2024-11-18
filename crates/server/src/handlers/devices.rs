use anyhow::Error;
use rockbox_chromecast::Chromecast;

use crate::{
    http::{Context, Request, Response},
    GLOBAL_MUTEX,
};

pub async fn connect(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let id = &req.params[0];
    let mut player = ctx.player.lock().unwrap();
    let mut current_device = ctx.current_device.lock().unwrap();
    let devices = ctx.devices.lock().unwrap();
    let device = devices.iter().find(|d| d.id == *id);
    if let Some(device) = device {
        let mut mutex = GLOBAL_MUTEX.lock().unwrap();
        *mutex = 1;
        *player = Chromecast::connect(device.clone())?;
        *current_device = Some(device.clone());
        res.set_status(200);
        return Ok(());
    }
    res.set_status(404);
    Ok(())
}

pub async fn disconnect(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let _id = &req.params[0];
    let mut player = ctx.player.lock().unwrap();
    let mut current_device = ctx.current_device.lock().unwrap();
    if let Some(player) = player.as_mut() {
        player.stop().await?;
        player.disconnect().await?;
    }
    let mut mutex = GLOBAL_MUTEX.lock().unwrap();
    *mutex = 0;
    *player = None;
    *current_device = None;
    res.set_status(200);
    Ok(())
}

pub async fn get_devices(ctx: &Context, _req: &Request, res: &mut Response) -> Result<(), Error> {
    let devices = ctx.devices.lock().unwrap();
    res.json(&devices.clone());
    Ok(())
}

pub async fn get_device(ctx: &Context, req: &Request, res: &mut Response) -> Result<(), Error> {
    let id = &req.params[0];
    if id == "current" {
        let current_device = ctx.current_device.lock().unwrap();
        if let Some(device) = current_device.as_ref() {
            res.json(&device.clone());
            return Ok(());
        }
        res.set_status(404);
        return Ok(());
    }

    let devices = ctx.devices.lock().unwrap();
    let device = devices.iter().find(|d| d.id == *id);

    if let Some(device) = device {
        res.json(&device.clone());
        return Ok(());
    }

    res.json(&device);
    res.set_status(404);
    Ok(())
}
