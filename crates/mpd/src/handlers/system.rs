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
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let receiver = ctx.event_receiver.clone();

    tokio::spawn(async move {
        let mut rx = receiver.lock().await;
        loop {
            match rx.recv().await {
                Ok(Subsystem::NoIdle) => break, // handle_noidle already sent OK
                Ok(event) => {
                    let mut changed = vec![event.to_string()];
                    // Drain any other pending events without blocking
                    while let Ok(next) = rx.try_recv() {
                        if next != Subsystem::NoIdle {
                            let name = next.to_string();
                            if !changed.contains(&name) {
                                changed.push(name);
                            }
                        }
                    }
                    let mut response = changed
                        .iter()
                        .map(|s| format!("changed: {}\n", s))
                        .collect::<String>();
                    response.push_str("OK\n");
                    tx.send(response.into_bytes()).await.ok();
                    break;
                }
                Err(_) => break,
            }
        }
        Ok::<(), Error>(())
    });

    Ok("".to_string())
}

pub async fn handle_noidle(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    ctx.event_sender.send(Subsystem::NoIdle)?;

    let response = "OK\n".to_string();
    if !ctx.batch {
        tx.send(response.clone().into_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_decoders(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(DECODERS.as_bytes().to_vec()).await?;
    }
    Ok(DECODERS.to_string())
}

pub async fn handle_commands(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(COMMANDS.as_bytes().to_vec()).await?;
    }
    Ok(COMMANDS.to_string())
}

pub async fn handle_binarylimit(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_ping(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    if !ctx.batch {
        tx.send(b"OK\n".to_vec()).await?;
    }
    Ok("OK\n".to_string())
}

pub async fn handle_notcommands(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let response = "OK\n".to_string();
    if !ctx.batch {
        tx.send(response.clone().into_bytes()).await?;
    }
    Ok(response)
}

pub async fn handle_urlhandlers(
    ctx: &mut Context,
    _request: &str,
    tx: Sender<Vec<u8>>,
) -> Result<String, Error> {
    let response = "handler: file://\nOK\n".to_string();
    if !ctx.batch {
        tx.send(response.clone().into_bytes()).await?;
    }
    Ok(response)
}
