use anyhow::Error;
use std::path::Path;
use std::process::Command;

const SERVICE_TEMPLATE: &str = include_str!("../../systemd/rockbox.service");

pub fn install() -> Result<(), Error> {
    let home = std::env::var("HOME")?;
    let service_path: &str = &format!("{}/.config/systemd/user/rockbox.service", home);
    std::fs::create_dir_all(format!("{}/.config/systemd/user", home))
        .expect("Failed to create systemd user directory");

    if Path::new(service_path).exists() {
        println!("Service file already exists. Nothing to install.");
        return Ok(());
    }

    let rockbox_path = std::env::current_exe()?;
    let service_template: &str = &SERVICE_TEMPLATE.replace(
        "ExecStart=/usr/local/bin/rockboxd",
        &format!("ExecStart={}d", rockbox_path.display()),
    );

    std::fs::write(service_path, service_template).expect("Failed to write service file");

    Command::new("systemctl")
        .arg("--user")
        .arg("daemon-reload")
        .status()?;

    Command::new("systemctl")
        .arg("--user")
        .arg("enable")
        .arg("rockbox")
        .status()?;

    Command::new("systemctl")
        .arg("--user")
        .arg("start")
        .arg("rockbox")
        .status()?;

    println!("✅ Rockbox service installed successfully!");

    Ok(())
}

pub fn uninstall() -> Result<(), Error> {
    let home = std::env::var("HOME")?;
    let service_path: &str = &format!("{}/.config/systemd/user/rockbox.service", home);

    if Path::new(service_path).exists() {
        Command::new("systemctl")
            .arg("--user")
            .arg("stop")
            .arg("rockbox")
            .status()?;

        Command::new("systemctl")
            .arg("--user")
            .arg("disable")
            .arg("rockbox")
            .status()?;

        std::fs::remove_file(service_path).expect("Failed to remove service file");

        Command::new("systemctl")
            .arg("--user")
            .arg("daemon-reload")
            .status()?;

        println!("✅ Rockbox service uninstalled successfully!");
    } else {
        println!("Service file does not exist. Nothing to uninstall.");
    }

    Ok(())
}

pub fn status() -> Result<(), Error> {
    let home = std::env::var("HOME")?;
    let service_path: &str = &format!("{}/.config/systemd/user/rockbox.service", home);

    if Path::new(service_path).exists() {
        Command::new("systemctl")
            .arg("--user")
            .arg("status")
            .arg("rockbox")
            .status()?;
    } else {
        println!("Service file does not exist. Rockbox service is not installed.");
    }

    Ok(())
}
