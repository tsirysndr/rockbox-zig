use anyhow::Error;
use rockbox_bluetooth::{connect, disconnect, get_devices, scan};

use crate::http::{Context, Request, Response};

pub async fn scan_bluetooth(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let timeout_secs = req.query_params["timeout_secs"]
        .as_str()
        .and_then(|s| s.parse::<u64>().ok())
        .or_else(|| req.query_params["timeout_secs"].as_u64())
        .unwrap_or(10);

    let devices = scan(timeout_secs).await?;
    res.json(&devices);
    Ok(())
}

pub async fn get_bluetooth_devices(
    _ctx: &Context,
    _req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let devices = get_devices().await?;
    res.json(&devices);
    Ok(())
}

pub async fn connect_bluetooth_device(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let address = &req.params[0];
    match connect(address).await {
        Ok(_) => res.set_status(200),
        Err(e) => {
            tracing::error!("bluetooth: connect {}: {}", address, e);
            res.set_status(500);
        }
    }
    Ok(())
}

pub async fn disconnect_bluetooth_device(
    _ctx: &Context,
    req: &Request,
    res: &mut Response,
) -> Result<(), Error> {
    let address = &req.params[0];
    match disconnect(address).await {
        Ok(_) => res.set_status(200),
        Err(e) => {
            tracing::error!("bluetooth: disconnect {}: {}", address, e);
            res.set_status(500);
        }
    }
    Ok(())
}
