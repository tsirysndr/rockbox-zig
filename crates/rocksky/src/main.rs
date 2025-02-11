use std::{thread, time::Duration};

use anyhow::Error;
use rockbox_library::repo;
use rockbox_rocksky::save_track;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let pool = rockbox_library::create_connection_pool().await?;
    let tracks = repo::track::all(pool.clone()).await?;
    println!("Tracks {}", tracks.len());

    for (i, track) in tracks.iter().enumerate() {
        print!("{}/{} ", i + 1, tracks.len());
        let album = repo::album::find(pool.clone(), &track.album_id).await?;
        let track_clone = track.clone();
        let album = album.unwrap();
        let album_clone = album.clone();
        thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(save_track(track_clone, album_clone)).unwrap();
        });
        thread::sleep(Duration::from_millis(500));
    }

    Ok(())
}
