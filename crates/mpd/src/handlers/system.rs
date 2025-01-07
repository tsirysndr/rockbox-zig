use std::os::unix::thread;

use anyhow::Error;
use tokio::sync::mpsc::Sender;

use crate::{
    consts::{COMMANDS, DECODERS},
    Context,
};

use super::Subsystem;

pub async fn handle_idle(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    let receiver = ctx.event_receiver.clone();

    tokio::spawn(async move {
        let mut rx = receiver.lock().await;
        while let Ok(event) = rx.recv().await {
            if event == Subsystem::NoIdle {
                break;
            }
            tx.send(format!("changed: {}\n", event.to_string())).await?;
        }
        Ok::<(), Error>(())
    });

    Ok("".to_string())
}

pub async fn handle_noidle(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    ctx.event_sender.send(Subsystem::NoIdle)?;

    let response = "OK\n".to_string();
    if !ctx.batch {
        tx.send(response.clone()).await?;
    }
    Ok(response)
}

pub async fn handle_decoders(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(DECODERS.to_string()).await?;
    }
    Ok(DECODERS.to_string())
}

pub async fn handle_commands(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(COMMANDS.to_string()).await?;
    }
    Ok(COMMANDS.to_string())
}

pub async fn handle_binarylimit(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<String>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send("OK\n".to_string()).await?;
    }
    Ok("OK\n".to_string())
}
