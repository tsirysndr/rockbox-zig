use std::{env, ffi::OsString};

use anyhow::Error;
use clap::{arg, Command};
use owo_colors::OwoColorize;

use cmd::{
    clear::*, community::*, login::*, open::*, repl::*, run::*, scan::*, service, start::*,
    webui::*, whoami::*,
};

use crate::cmd::setup;

pub mod cmd;

fn cli() -> Command {
    let banner = format!(
        "{}\nA fork of the original Rockbox project, with a focus on modernization and more features.",
        r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#
        .yellow()
    );
    const VERSION: &str = match option_env!("TAG") {
        Some(tag) => tag,
        None => env!("CARGO_PKG_VERSION"),
    };
    let cli = Command::new("rockbox")
        .version(VERSION)
        .about(&banner)
        .arg(arg!(--rebuild -r "Rebuild index after scan"))
        .subcommand(
            Command::new("scan")
                .arg(arg!(--directory -d [PATH] "path to your music library").required(false))
                .arg(arg!(--rebuild -r "Rebuild index after scan"))
                .about("Scan your music library for new media files"),
        )
        .subcommand(
            Command::new("community").about("Join our community on Discord to chat with us!"),
        )
        .subcommand(
            Command::new("start")
                .about("Start Rockbox server")
                .arg(arg!(--rebuild -r "Rebuild index after scan")),
        )
        .subcommand(Command::new("tui").about("Start Rockbox TUI"))
        .subcommand(
            Command::new("webui")
                .about("Open the Rockbox web UI in your browser")
                .visible_alias("web"),
        )
        .subcommand(
            Command::new("repl")
                .about("Start the Rockbox REPL")
                .visible_alias("shell"),
        )
        .subcommand(
            Command::new("run")
                .arg(arg!(<FILE> "JavaScript or TypeScript file to run"))
                .about("Run a JavaScript or TypeScript program")
                .visible_alias("x"),
        )
        .subcommand(
            Command::new("open")
                .arg(arg!(<PATH_OR_URL> "Local file path or HTTP URL to play"))
                .about("Play a local track or remote HTTP URL directly"),
        )
        .subcommand(
            Command::new("service")
                .about("Manage systemd service for Rockbox")
                .subcommand(Command::new("install").about("Install systemd service for Rockbox"))
                .subcommand(
                    Command::new("uninstall").about("Uninstall systemd service for Rockbox"),
                )
                .subcommand(
                    Command::new("status").about("Check status of systemd service for Rockbox"),
                ),
        )
        .subcommand(
            Command::new("login")
                .arg(arg!(<handle> "Your BlueSky handle"))
                .about("Login to your Rocksky account")
                .visible_alias("auth"),
        )
        .subcommand(
            Command::new("whoami")
                .about("Display information about the currently logged in user")
                .visible_alias("me"),
        )
        .subcommand(Command::new("setup").about("Setup Rockbox and its dependencies"))
        .subcommand(Command::new("clear").about("Clear current playlist"));
    #[cfg(target_os = "linux")]
    let cli = cli.subcommand(
        Command::new("bluetooth")
            .about("Manage Bluetooth audio devices")
            .subcommand_required(true)
            .arg_required_else_help(true)
            .subcommand(
                Command::new("scan")
                    .about("Scan for nearby Bluetooth devices")
                    .arg(
                        clap::Arg::new("timeout")
                            .long("timeout")
                            .short('t')
                            .value_name("SECS")
                            .default_value("10")
                            .value_parser(clap::value_parser!(u64))
                            .help("Scan duration in seconds"),
                    ),
            )
            .subcommand(Command::new("devices").about("List known Bluetooth devices"))
            .subcommand(
                Command::new("connect")
                    .about("Connect (pair) a Bluetooth audio device")
                    .arg(
                        clap::Arg::new("address")
                            .required(true)
                            .help("Bluetooth device address (e.g. AA:BB:CC:DD:EE:FF)"),
                    ),
            )
            .subcommand(
                Command::new("disconnect")
                    .about("Disconnect a Bluetooth device")
                    .arg(
                        clap::Arg::new("address")
                            .required(true)
                            .help("Bluetooth device address"),
                    ),
            ),
    );
    cli
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let subscriber = tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);

    let args = std::env::args().collect::<Vec<String>>();
    if args.len() > 1 && args[1] == "run" {
        let _args = args
            .into_iter()
            .map(|s| match s.as_str() {
                "rockbox" => "deno".into(),
                _ => s.into(),
            })
            .collect::<Vec<OsString>>();

        run(_args);
        return Ok(());
    }

    let matches = cli().get_matches();

    match matches.subcommand() {
        Some(("scan", args)) => {
            let directory = args.get_one::<String>("directory").map(|d| d.to_string());
            let rebuild_index = match args.get_flag("rebuild") {
                true => Some(true),
                false => None,
            };
            scan(directory, rebuild_index).await?;
        }
        Some(("community", _)) => {
            community();
        }
        Some(("start", _)) => {
            start(true)?;
        }
        Some(("webui", _)) => {
            webui()?;
        }
        Some(("repl", _)) => {
            repl();
        }
        Some(("open", args)) => {
            let path_or_url = args.get_one::<String>("PATH_OR_URL").unwrap();
            open(path_or_url).await?;
        }
        Some(("tui", _)) => {
            rmpc::main_tui()?;
        }
        Some(("service", sub_m)) => match sub_m.subcommand() {
            Some(("install", _)) => {
                service::install()?;
            }
            Some(("uninstall", _)) => {
                service::uninstall()?;
            }
            Some(("status", _)) => {
                service::status()?;
            }
            _ => {
                println!("Invalid subcommand. Use `rockbox service --help` for more information.");
            }
        },
        Some(("login", args)) => {
            let handle = args.get_one::<String>("handle").unwrap();
            login(handle).await?;
        }
        Some(("whoami", _)) => {
            whoami().await?;
        }
        Some(("clear", _)) => {
            match clear() {
                Ok(_) => {}
                Err(e) => {}
            };
            println!("✅ Rockbox Playlist Cleared");
        }
        Some(("setup", _)) => {
            setup::install_dependencies()?;
        }
        #[cfg(target_os = "linux")]
        Some(("bluetooth", sub_m)) => match sub_m.subcommand() {
            Some(("scan", m)) => {
                let timeout = *m.get_one::<u64>("timeout").unwrap_or(&10);
                cmd::bluetooth::scan(timeout).await?;
            }
            Some(("devices", _)) => {
                cmd::bluetooth::devices().await?;
            }
            Some(("connect", m)) => {
                let address = m.get_one::<String>("address").unwrap();
                cmd::bluetooth::connect(address).await?;
            }
            Some(("disconnect", m)) => {
                let address = m.get_one::<String>("address").unwrap();
                cmd::bluetooth::disconnect(address).await?;
            }
            _ => {}
        },
        Some((_, args)) => {
            if args.get_flag("rebuild") {
                env::set_var("ROCKBOX_UPDATE_LIBRARY", "1");
            }
            start(true)?;
        }
        None => start(true)?,
    }
    Ok(())
}
