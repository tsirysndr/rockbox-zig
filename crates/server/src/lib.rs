use owo_colors::OwoColorize;
use rockbox_sys::playback;

#[no_mangle]
pub extern "C" fn start_server() {
    const BANNER: &str = r#"
              __________               __   ___.
    Open      \______   \ ____   ____ |  | _\_ |__   _______  ___
    Source     |       _//  _ \_/ ___\|  |/ /| __ \ /  _ \  \/  /
    Jukebox    |    |   (  <_> )  \___|    < | \_\ (  <_> > <  <
    Firmware   |____|_  /\____/ \___  >__|_ \|___  /\____/__/\_ \
                      \/            \/     \/    \/            \/
    "#;

    // Start the server
    println!("{}", BANNER.yellow());
    let status = playback::status();
    playback::current_track();
    println!("Status: {}", status);
    playback::pause();
}
