use std::env;

use anyhow::Error;
use owo_colors::OwoColorize;
use rockbox::api::rockbox::v1alpha1::{
    bluetooth_service_client::BluetoothServiceClient, BluetoothDevice,
    ConnectBluetoothDeviceRequest, DisconnectBluetoothDeviceRequest, GetBluetoothDevicesRequest,
    ScanBluetoothRequest,
};

fn grpc_url() -> String {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());
    format!("tcp://{}:{}", host, port)
}

fn print_devices(devices: &[BluetoothDevice]) {
    if devices.is_empty() {
        println!("No devices found.");
        return;
    }
    println!(
        "{:<20} {:<32} {:<8} {:<8} {:<12} {}",
        "Address".bold(),
        "Name".bold(),
        "Paired".bold(),
        "Trusted".bold(),
        "Connected".bold(),
        "RSSI".bold(),
    );
    println!("{}", "─".repeat(88));
    for d in devices {
        let rssi = d
            .rssi
            .map(|r| r.to_string())
            .unwrap_or_else(|| "-".to_string());
        let yn = |v: bool| {
            if v {
                "yes".green().to_string()
            } else {
                "no".red().to_string()
            }
        };
        println!(
            "{:<20} {:<32} {:<8} {:<8} {:<12} {}",
            d.address.cyan(),
            d.name,
            yn(d.paired),
            yn(d.trusted),
            yn(d.connected),
            rssi,
        );
    }
}

pub async fn scan(timeout_secs: u64) -> Result<(), Error> {
    let mut client = BluetoothServiceClient::connect(grpc_url()).await?;
    let devices = client
        .scan(tonic::Request::new(ScanBluetoothRequest {
            timeout_secs: timeout_secs as u32,
        }))
        .await?
        .into_inner()
        .devices;
    print_devices(&devices);
    Ok(())
}

pub async fn devices() -> Result<(), Error> {
    let mut client = BluetoothServiceClient::connect(grpc_url()).await?;
    let devices = client
        .get_devices(tonic::Request::new(GetBluetoothDevicesRequest {}))
        .await?
        .into_inner()
        .devices;
    print_devices(&devices);
    Ok(())
}

pub async fn connect(address: &str) -> Result<(), Error> {
    let mut client = BluetoothServiceClient::connect(grpc_url()).await?;
    client
        .connect_device(tonic::Request::new(ConnectBluetoothDeviceRequest {
            address: address.to_string(),
        }))
        .await?;
    println!("Connected to {}", address.green());
    Ok(())
}

pub async fn disconnect(address: &str) -> Result<(), Error> {
    let mut client = BluetoothServiceClient::connect(grpc_url()).await?;
    client
        .disconnect(tonic::Request::new(DisconnectBluetoothDeviceRequest {
            address: address.to_string(),
        }))
        .await?;
    println!("Disconnected from {}", address.yellow());
    Ok(())
}
