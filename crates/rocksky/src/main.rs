use anyhow::Error;

#[tokio::main]
async fn main() -> Result<(), Error> {
    rockbox_rocksky::register_rockbox()?;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}
