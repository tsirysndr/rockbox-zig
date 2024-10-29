use opener::open;
use owo_colors::OwoColorize;

pub fn webui() {
    match open("http://localhost:6062") {
        Ok(_) => {}
        Err(_) => println!(
            "Open this link in your browser {}",
            "http://localhost:6062".purple()
        ),
    };
}
