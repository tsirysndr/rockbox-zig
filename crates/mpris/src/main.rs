use anyhow::Error;
use rockbox_mpris::MprisServer;

#[async_std::main]
async fn main() -> Result<(), Error> {
    MprisServer::start().await?;
    Ok(())
}
