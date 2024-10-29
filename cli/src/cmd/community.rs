use opener::open;
use owo_colors::OwoColorize;

pub fn community() {
    match open("https://discord.gg/tXPrgcPKSt") {
        Ok(_) => {}
        Err(_) => println!(
            "Open this link in your browser {}",
            "https://discord.gg/tXPrgcPKSt".purple()
        ),
    };
}
