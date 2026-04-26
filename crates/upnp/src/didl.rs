use crate::db::{Album, Artist, Track};
use crate::format::protocol_info_for_path;

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Format track duration (milliseconds) as `h:mm:ss.mmm`.
fn fmt_duration(ms: i64) -> String {
    let ms = ms.max(0) as u64;
    let secs = ms / 1000;
    let millis = ms % 1000;
    let minutes = secs / 60;
    let secs = secs % 60;
    let hours = minutes / 60;
    let minutes = minutes % 60;
    format!("{hours}:{minutes:02}:{secs:02}.{millis:03}")
}

pub fn track_item(track: &Track, parent_id: &str, base_url: &str) -> String {
    let protocol_info = protocol_info_for_path(&track.path);
    let duration = fmt_duration(track.length);
    let res_url = format!("{base_url}/audio/{}", track.id);
    let art_tag = track.album_art.as_deref().map(|_| {
        format!(
            "<upnp:albumArtURI>{base_url}/art/{}</upnp:albumArtURI>",
            xml_escape(&track.album_id)
        )
    });
    let track_num_tag = track
        .track_number
        .map(|n| format!("<upnp:originalTrackNumber>{n}</upnp:originalTrackNumber>"));
    let genre_tag = track
        .genre
        .as_deref()
        .map(|g| format!("<upnp:genre>{}</upnp:genre>", xml_escape(g)));

    format!(
        r#"<item id="track:{tid}" parentID="{parent_id}" restricted="1">
  <dc:title>{title}</dc:title>
  <dc:creator>{artist}</dc:creator>
  <upnp:class>object.item.audioItem.musicTrack</upnp:class>
  <upnp:artist>{artist}</upnp:artist>
  <upnp:album>{album}</upnp:album>
  {art}{track_num}{genre}<res protocolInfo="{protocol_info}" size="{size}" duration="{duration}">{res_url}</res>
</item>"#,
        tid = xml_escape(&track.id),
        parent_id = xml_escape(parent_id),
        title = xml_escape(&track.title),
        artist = xml_escape(&track.artist),
        album = xml_escape(&track.album),
        art = art_tag.unwrap_or_default(),
        track_num = track_num_tag.unwrap_or_default(),
        genre = genre_tag.unwrap_or_default(),
        size = track.filesize,
    )
}

pub fn album_container(album: &Album, parent_id: &str) -> String {
    format!(
        r#"<container id="album:{id}" parentID="{parent_id}" restricted="1" childCount="{count}" searchable="0">
  <dc:title>{title}</dc:title>
  <dc:creator>{artist}</dc:creator>
  <upnp:class>object.container.album.musicAlbum</upnp:class>
</container>"#,
        id = xml_escape(&album.id),
        parent_id = xml_escape(parent_id),
        title = xml_escape(&album.title),
        artist = xml_escape(&album.artist),
        count = album.track_count,
    )
}

pub fn artist_container(artist: &Artist, parent_id: &str) -> String {
    format!(
        r#"<container id="artist:{id}" parentID="{parent_id}" restricted="1" childCount="{count}" searchable="0">
  <dc:title>{name}</dc:title>
  <upnp:class>object.container.person.musicArtist</upnp:class>
</container>"#,
        id = xml_escape(&artist.id),
        parent_id = xml_escape(parent_id),
        name = xml_escape(&artist.name),
        count = artist.track_count,
    )
}

pub fn simple_container(id: &str, parent_id: &str, title: &str, child_count: i64) -> String {
    format!(
        r#"<container id="{id}" parentID="{parent_id}" restricted="1" childCount="{child_count}" searchable="1">
  <dc:title>{title}</dc:title>
  <upnp:class>object.container</upnp:class>
</container>"#,
        id = xml_escape(id),
        parent_id = xml_escape(parent_id),
        title = xml_escape(title),
    )
}

pub fn wrap_didl(items: &[String]) -> String {
    let body = items.join("\n");
    format!(
        r#"<DIDL-Lite xmlns="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/" xmlns:dc="http://purl.org/dc/elements/1.1/" xmlns:upnp="urn:schemas-upnp-org:metadata-1-0/upnp/">{body}</DIDL-Lite>"#
    )
}

/// XML-escape the DIDL-Lite string for embedding inside a SOAP <Result> element.
pub fn escape_for_result(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}
