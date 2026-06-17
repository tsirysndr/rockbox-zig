use anyhow::Error;
use lofty::{file::TaggedFileExt, probe::Probe, tag::Tag};

pub fn extract_label(track_path: &str) -> Result<Option<String>, Error> {
    let probe = match Probe::open(track_path) {
        Ok(p) => p,
        Err(e) => {
            println!("label: cannot open {}: {}", track_path, e);
            return Ok(None);
        }
    };
    let tagged_file = match probe.read() {
        Ok(tagged_file) => tagged_file,
        Err(e) => {
            println!("Error opening file: {}", e);
            return Ok(None);
        }
    };

    let primary_tag = tagged_file.primary_tag();
    let tag: &Tag = match primary_tag {
        Some(tag) => tag,
        None => {
            println!("No tag found in file: {}", track_path);
            return Ok(None);
        }
    };

    let label = tag
        .get_string(&lofty::tag::ItemKey::Label)
        .map(|label| label.to_string());

    Ok(label)
}
