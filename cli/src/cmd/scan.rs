use std::env;

use anyhow::Error;
use rockbox::api::rockbox::v1alpha1::{
    library_service_client::LibraryServiceClient, ScanLibraryRequest,
};

pub async fn scan(path: Option<String>) -> Result<(), Error> {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    let url = format!("tcp://{}:{}", host, port);
    let mut client = LibraryServiceClient::connect(url).await?;
    let request = tonic::Request::new(ScanLibraryRequest { path });
    client.scan_library(request).await?;
    println!("Scan request sent to Rockbox server");
    Ok(())
}
