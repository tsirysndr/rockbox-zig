fn main() {
    match rockbox_controls::run_media_controls() {
        Ok(_) => {}
        Err(e) => {
            eprintln!("Error starting rockbox controls: {}", e);
        }
    }
}
