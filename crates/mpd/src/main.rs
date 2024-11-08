use anyhow::Error;
use rockbox_mpd::MpdServer;

#[tokio::main]
async fn main() -> Result<(), Error> {
    MpdServer::start().await?;
    Ok(())
}
