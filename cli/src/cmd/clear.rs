use std::fs;

use anyhow::Error;

pub fn clear() -> Result<(), Error> {
    let mut playlist_control = dirs::home_dir().unwrap();
    playlist_control.push(".config/rockbox.org/.playlist_control");
    fs::remove_file(playlist_control)?;

    let mut playlist_control = dirs::home_dir().unwrap();
    playlist_control.push(".config/rockbox.org/.playlist_control.old");
    fs::remove_file(playlist_control)?;

    let mut resume = dirs::home_dir().unwrap();
    resume.push(".config/rockbox.org/.resume.cfg");
    fs::remove_file(resume)?;

    let mut resume = dirs::home_dir().unwrap();
    resume.push(".config/rockbox.org/.resume.cfg.new");
    fs::remove_file(resume)?;
    Ok(())
}
