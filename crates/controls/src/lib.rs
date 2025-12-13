use anyhow::Error;
use souvlaki::{
    MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition, PlatformConfig,
};
use std::env;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc};
use std::time::Duration;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::WindowId;

use crate::api::rockbox::v1alpha1::playback_service_client::PlaybackServiceClient;
use crate::api::rockbox::v1alpha1::{StreamCurrentTrackRequest, StreamStatusRequest};

pub mod api {
    #[path = ""]
    pub mod rockbox {

        #[path = "rockbox.v1alpha1.rs"]
        pub mod v1alpha1;
    }
}

// Commands to send to the media controls from async tasks
#[derive(Debug)]
enum MediaCommand {
    SetMetadata {
        title: String,
        artist: String,
        album: String,
        duration: Duration,
        cover_url: Option<String>,
    },
    Play,
    Pause,
    Next,
    Previous,
    SetMediaPosition((MediaPosition, bool)),
}

struct App {
    controls: MediaControls,
    command_receiver: mpsc::Receiver<MediaCommand>,
    media_event_sender: tokio::sync::mpsc::UnboundedSender<MediaControlEvent>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {
        println!("App resumed, media controls ready!");
    }

    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Process all pending commands from async tasks
        while let Ok(cmd) = self.command_receiver.try_recv() {
            match cmd {
                MediaCommand::SetMetadata {
                    title,
                    artist,
                    album,
                    duration,
                    cover_url,
                } => {
                    // Set playback state first
                    if let Err(e) = self
                        .controls
                        .set_playback(MediaPlayback::Playing { progress: None })
                    {
                        eprintln!("Failed to set playback state: {}", e);
                    }

                    // Then set metadata
                    if let Err(e) = self.controls.set_metadata(MediaMetadata {
                        title: Some(&title),
                        artist: Some(&artist),
                        album: Some(&album),
                        duration: Some(duration),
                        cover_url: cover_url.as_deref(),
                    }) {
                        eprintln!("Failed to set metadata: {}", e);
                    }
                }
                MediaCommand::Play => {
                    if let Err(e) = self
                        .controls
                        .set_playback(MediaPlayback::Playing { progress: None })
                    {
                        eprintln!("Failed to set playback state: {}", e);
                    }
                }
                MediaCommand::Pause => {
                    if let Err(e) = self
                        .controls
                        .set_playback(MediaPlayback::Paused { progress: None })
                    {
                        eprintln!("Failed to set playback state: {}", e);
                    }
                }
                MediaCommand::SetMediaPosition((position, playing)) => {
                    if let Err(e) = self.controls.set_playback(match playing {
                        true => MediaPlayback::Playing {
                            progress: Some(position),
                        },
                        false => MediaPlayback::Paused {
                            progress: Some(position),
                        },
                    }) {
                        eprintln!("Failed to set playback state: {}", e);
                    }
                }
                _ => {}
            }
        }
    }
}

/// Start the media controls system.
/// This function blocks and runs the event loop on the main thread.
pub fn run_media_controls() -> Result<(), Box<dyn std::error::Error>> {
    // Channel for sending commands TO the media controls (from async tasks)
    let (command_sender, command_receiver) = mpsc::channel::<MediaCommand>();

    // Channel for receiving events FROM the media controls (to async tasks)
    let (media_event_sender, media_event_receiver) =
        tokio::sync::mpsc::unbounded_channel::<MediaControlEvent>();

    // Shared playing state between status and metadata tasks
    let is_playing = Arc::new(AtomicBool::new(false));

    let sender = command_sender.clone();
    let playing_state = Arc::clone(&is_playing);
    std::thread::spawn(move || {
        println!(">> spawn metadata update task");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(spawn_metadata_update_task(sender.clone(), playing_state)) {
            Ok(_) => println!("Metadata update task completed"),
            Err(e) => eprintln!("Metadata update task failed: {}", e),
        }
    });

    let sender = command_sender.clone();
    std::thread::spawn(move || {
        println!(">> spawn event handler task");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(spawn_event_handler_task(
            sender.clone(),
            media_event_receiver,
        )) {
            Ok(()) => println!("Event handler task completed"),
            Err(e) => eprintln!("Event handler task failed: {}", e),
        }
    });

    let sender = command_sender.clone();
    let playing_state = Arc::clone(&is_playing);
    std::thread::spawn(move || {
        println!(">> spawn status update task");
        let runtime = tokio::runtime::Runtime::new().unwrap();
        match runtime.block_on(spawn_status_update_task(sender.clone(), playing_state)) {
            Ok(_) => println!("Status update task completed"),
            Err(e) => eprintln!("Status update task failed: {}", e),
        }
    });

    // Run the event loop on the main thread (required for macOS)
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut controls = MediaControls::new(PlatformConfig {
        display_name: "Rockbox",
        dbus_name: "tsirysndr.rockbox",
        hwnd: None,
    })?;

    // Attach event handler - forward events to async task
    let event_sender = media_event_sender.clone();
    controls.attach(move |event| {
        let _ = event_sender.send(event);
    })?;

    let mut app = App {
        controls,
        command_receiver,
        media_event_sender,
    };

    event_loop.run_app(&mut app)?;

    Ok(())
}

async fn spawn_metadata_update_task(
    command_sender: mpsc::Sender<MediaCommand>,
    is_playing: Arc<AtomicBool>,
) -> Result<(), Error> {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    let url = format!("tcp://{}:{}", host, port);

    let mut client = PlaybackServiceClient::connect(url).await?;

    let mut stream = client
        .stream_current_track(StreamCurrentTrackRequest {})
        .await?
        .into_inner();

    let asset_host = env::var("ROCKBOX_GRAPHQL_HOST").unwrap_or_else(|_| "localhost".to_string());
    let asset_port = env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or_else(|_| "6062".to_string());

    let mut previous_album_art: Option<String> = None;
    let mut current_cover_url: Option<String> = None;

    while let Some(track) = stream.message().await? {
        // Only update cover_url if album_art has changed
        if track.album_art != previous_album_art {
            previous_album_art.clone_from(&track.album_art);
            current_cover_url = match &track.album_art {
                Some(album_art) => match album_art.starts_with("http") {
                    true => Some(album_art.clone()),
                    false => Some(format!(
                        "http://{}:{}/covers/{}",
                        asset_host, asset_port, album_art
                    )),
                },
                None => None,
            };
        }

        let cmd = MediaCommand::SetMetadata {
            title: track.title,
            artist: track.artist,
            album: track.album,
            duration: Duration::from_millis(track.length),
            cover_url: current_cover_url.clone(),
        };

        command_sender.send(cmd)?;

        // Get current playing state and send position update
        let playing = is_playing.load(Ordering::Relaxed);
        command_sender.send(MediaCommand::SetMediaPosition((
            MediaPosition(Duration::from_millis(track.elapsed)),
            playing,
        )))?;
    }

    Ok(())
}

async fn spawn_event_handler_task(
    command_sender: std::sync::mpsc::Sender<MediaCommand>,
    mut receiver: tokio::sync::mpsc::UnboundedReceiver<MediaControlEvent>,
) -> Result<(), Error> {
    while let Some(event) = receiver.recv().await {
        match event {
            MediaControlEvent::Play => {
                println!("[MediaControl] Play");
                // TODO: Add your play logic here
                command_sender.send(MediaCommand::Play)?;
            }
            MediaControlEvent::Pause => {
                println!("[MediaControl] Pause");
                // TODO: Add your pause logic here
                command_sender.send(MediaCommand::Pause)?;
            }
            MediaControlEvent::Next => {
                println!("[MediaControl] Next");
                // TODO: Add your next track logic here
                command_sender.send(MediaCommand::Next)?;
            }
            MediaControlEvent::Previous => {
                println!("[MediaControl] Previous");
                // TODO: Add your previous track logic here
                command_sender.send(MediaCommand::Previous)?;
            }
            MediaControlEvent::Seek(_) => {
                println!("[MediaControl] Seek");
            }
            MediaControlEvent::Toggle => {
                println!("[MediaControl] Toggle");
            }
            MediaControlEvent::Stop => {
                println!("[MediaControl] Stop");
            }
            MediaControlEvent::SeekBy(seek_direction, duration) => {
                println!("[MediaControl] SeekBy {:?} {:?}", seek_direction, duration);
            }
            MediaControlEvent::SetPosition(media_position) => {
                println!("[MediaControl] SetPosition {:?}", media_position);
            }
            MediaControlEvent::SetVolume(volume) => {
                println!("[MediaControl] SetVolume {}", volume);
            }
            MediaControlEvent::OpenUri(uri) => {
                println!("[MediaControl] OpenUri {}", uri);
            }
            MediaControlEvent::Raise => {
                println!("[MediaControl] Raise");
            }
            MediaControlEvent::Quit => {
                println!("[MediaControl] Quit");
            }
        }
    }

    println!(">> Event receiver closed");

    Ok(())
}

async fn spawn_status_update_task(
    command_sender: mpsc::Sender<MediaCommand>,
    is_playing: Arc<AtomicBool>,
) -> Result<(), Error> {
    let host = env::var("ROCKBOX_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("ROCKBOX_PORT").unwrap_or_else(|_| "6061".to_string());

    let url = format!("tcp://{}:{}", host, port);

    let mut client = PlaybackServiceClient::connect(url).await?;

    let mut stream = client
        .stream_status(StreamStatusRequest {})
        .await?
        .into_inner();

    while let Some(response) = stream.message().await? {
        match response.status {
            1 => {
                is_playing.store(true, Ordering::Relaxed);
                command_sender.send(MediaCommand::Play)?;
            }
            3 => {
                is_playing.store(false, Ordering::Relaxed);
                command_sender.send(MediaCommand::Pause)?;
            }
            _ => {
                is_playing.store(false, Ordering::Relaxed);
                command_sender.send(MediaCommand::Pause)?;
            }
        };
    }

    Ok(())
}
