use anyhow::Error;
use lofty::{file::TaggedFileExt, probe::Probe, tag::Tag};

pub fn extract_copyright_message(track_path: &str) -> Result<Option<String>, Error> {
    let tagged_file = match Probe::open(track_path)
        .expect("ERROR: Bad path provided!")
        .read()
    {
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

    let copyright = tag
        .get_string(&lofty::tag::ItemKey::CopyrightMessage)
        .map(|v| v.to_string());

    Ok(copyright)
}
