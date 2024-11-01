use std::{env, thread};

use anyhow::Error;
use rockbox::{
    api::rockbox::v1alpha1::{library_service_client::LibraryServiceClient, ScanLibraryRequest},
    install_rockboxd, wait_for_rockboxd,
};

use super::start::*;

pub async fn scan(path: Option<String>) -> Result<(), Error> {
    install_rockboxd()?;
    let handle = thread::spawn(|| match start() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Failed to start Rockbox server: {}", e);
        }
    });

    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    wait_for_rockboxd(port.parse()?, None)?;

    let url = format!("tcp://{}:{}", host, port);
    let mut client = LibraryServiceClient::connect(url).await?;
    let request = tonic::Request::new(ScanLibraryRequest { path });
    client.scan_library(request).await?;
    println!("Scan request sent to Rockbox server");
    handle.join().unwrap();
    Ok(())
}
