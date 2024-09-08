use rockbox_sys::playback;

#[no_mangle]
pub extern "C" fn start_server() {
    // Start the server
    println!("Starting server...");
    let status = playback::status();
    playback::current_track();
    println!("Status: {}", status);
    playback::pause();
}
