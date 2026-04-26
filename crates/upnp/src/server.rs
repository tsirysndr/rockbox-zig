use crate::{db, device_uuid, didl, format, get_local_ip, CONFIG};
use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use sqlx::Pool;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::fs::File;
use tokio::net::TcpListener;

type BoxBody = Full<Bytes>;

struct State {
    pool: Pool<sqlx::Sqlite>,
    server_port: u16,
    friendly_name: String,
    uuid: String,
    local_ip: std::net::Ipv4Addr,
}

pub async fn run(port: u16) -> anyhow::Result<()> {
    let pool = match db::open_pool().await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("UPnP media server: library DB not available ({e}); retrying in 10s");
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;
            db::open_pool().await?
        }
    };

    let (friendly_name, server_port) = {
        let cfg = CONFIG.lock().unwrap();
        (cfg.friendly_name.clone(), cfg.server_port)
    };

    let state = Arc::new(State {
        pool,
        server_port,
        friendly_name: if friendly_name.is_empty() {
            "Rockbox".to_string()
        } else {
            friendly_name
        },
        uuid: device_uuid().to_string(),
        local_ip: get_local_ip(),
    });

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
    tracing::info!("UPnP HTTP server listening on :{port}");

    loop {
        let (stream, _) = listener.accept().await?;
        let io = TokioIo::new(stream);
        let state = state.clone();
        tokio::spawn(async move {
            let svc = service_fn(move |req| {
                let state = state.clone();
                async move { handle(req, state).await }
            });
            if let Err(e) = Builder::new(TokioExecutor::new())
                .serve_connection(io, svc)
                .await
            {
                tracing::debug!("UPnP HTTP: connection error: {e}");
            }
        });
    }
}

async fn handle(
    req: Request<Incoming>,
    state: Arc<State>,
) -> Result<Response<BoxBody>, hyper::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();
    // Extract headers before the body is consumed.
    let header_action = soap_action_from_header(req.headers());
    let range_hdr = req
        .headers()
        .get("Range")
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string());

    let resp = match (method, path.as_str()) {
        (Method::GET, "/desc.xml") => device_description(&state),
        (Method::GET, "/ContentDirectory/desc.xml") => content_directory_scpd(),
        (Method::GET, "/ConnectionManager/desc.xml") => connection_manager_scpd(),
        (Method::POST, "/ContentDirectory/control") => {
            let body = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body).unwrap_or("").to_string();
            content_directory_control(body_str, header_action, &state).await
        }
        (Method::POST, "/ConnectionManager/control") => {
            let body = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body).unwrap_or("").to_string();
            connection_manager_control(body_str, header_action)
        }
        (Method::GET, p) if p.starts_with("/audio/") => {
            let id = &p[7..];
            serve_audio(id, range_hdr.as_deref(), &state).await
        }
        (Method::GET, p) if p.starts_with("/art/") => {
            let album_id = &p[5..];
            serve_album_art(album_id, &state).await
        }
        (m, _) if m.as_str() == "SUBSCRIBE" || m.as_str() == "UNSUBSCRIBE" => Response::builder()
            .status(200)
            .header("SID", "uuid:rockbox-event-sid")
            .header("TIMEOUT", "Second-1800")
            .body(empty_body())
            .unwrap(),
        _ => not_found(),
    };
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Device + service descriptions
// ---------------------------------------------------------------------------

fn device_description(state: &State) -> Response<BoxBody> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="urn:schemas-upnp-org:device-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <device>
    <deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>
    <friendlyName>{name}</friendlyName>
    <manufacturer>Rockbox</manufacturer>
    <manufacturerURL>https://www.rockbox.org</manufacturerURL>
    <modelDescription>Rockbox UPnP/DLNA Media Server</modelDescription>
    <modelName>Rockbox</modelName>
    <modelNumber>1.0</modelNumber>
    <UDN>uuid:{uuid}</UDN>
    <dlna:X_DLNADOC xmlns:dlna="urn:schemas-dlna-org:device-1-0">DMS-1.50</dlna:X_DLNADOC>
    <serviceList>
      <service>
        <serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId>
        <SCPDURL>/ContentDirectory/desc.xml</SCPDURL>
        <controlURL>/ContentDirectory/control</controlURL>
        <eventSubURL>/ContentDirectory/events</eventSubURL>
      </service>
      <service>
        <serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId>
        <SCPDURL>/ConnectionManager/desc.xml</SCPDURL>
        <controlURL>/ConnectionManager/control</controlURL>
        <eventSubURL>/ConnectionManager/events</eventSubURL>
      </service>
    </serviceList>
  </device>
</root>"#,
        name = xml_escape(&state.friendly_name),
        uuid = state.uuid,
    );
    xml_response(xml)
}

fn content_directory_scpd() -> Response<BoxBody> {
    xml_response(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <actionList>
    <action>
      <name>Browse</name>
      <argumentList>
        <argument><name>ObjectID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_ObjectID</relatedStateVariable></argument>
        <argument><name>BrowseFlag</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_BrowseFlag</relatedStateVariable></argument>
        <argument><name>Filter</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Filter</relatedStateVariable></argument>
        <argument><name>StartingIndex</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Index</relatedStateVariable></argument>
        <argument><name>RequestedCount</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument>
        <argument><name>SortCriteria</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_SortCriteria</relatedStateVariable></argument>
        <argument><name>Result</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Result</relatedStateVariable></argument>
        <argument><name>NumberReturned</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument>
        <argument><name>TotalMatches</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_Count</relatedStateVariable></argument>
        <argument><name>UpdateID</name><direction>out</direction><relatedStateVariable>A_ARG_TYPE_UpdateID</relatedStateVariable></argument>
      </argumentList>
    </action>
  </actionList>
  <serviceStateTable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ObjectID</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Result</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_BrowseFlag</name><dataType>string</dataType><allowedValueList><allowedValue>BrowseMetadata</allowedValue><allowedValue>BrowseDirectChildren</allowedValue></allowedValueList></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Filter</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_SortCriteria</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Index</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Count</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_UpdateID</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="yes"><name>SystemUpdateID</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="yes"><name>ContainerUpdateIDs</name><dataType>string</dataType></stateVariable>
  </serviceStateTable>
</scpd>"#
            .to_string(),
    )
}

fn connection_manager_scpd() -> Response<BoxBody> {
    xml_response(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <actionList>
    <action>
      <name>GetProtocolInfo</name>
      <argumentList>
        <argument><name>Source</name><direction>out</direction><relatedStateVariable>SourceProtocolInfo</relatedStateVariable></argument>
        <argument><name>Sink</name><direction>out</direction><relatedStateVariable>SinkProtocolInfo</relatedStateVariable></argument>
      </argumentList>
    </action>
  </actionList>
  <serviceStateTable>
    <stateVariable sendEvents="yes"><name>SourceProtocolInfo</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="yes"><name>SinkProtocolInfo</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionStatus</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_AVTransportID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_RcsID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentConnectionIDs</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Direction</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ProtocolInfo</name><dataType>string</dataType></stateVariable>
  </serviceStateTable>
</scpd>"#
            .to_string(),
    )
}

// ---------------------------------------------------------------------------
// ContentDirectory SOAP control
// ---------------------------------------------------------------------------

async fn content_directory_control(
    body: String,
    header_action: Option<String>,
    state: &State,
) -> Response<BoxBody> {
    let action = resolve_action(header_action, &body);
    match action.as_deref() {
        Some("Browse") => browse_action(body, state).await,
        Some("GetSystemUpdateID") => soap_ok_response(
            "urn:schemas-upnp-org:service:ContentDirectory:1",
            "GetSystemUpdateID",
            "<Id>1</Id>",
        ),
        Some("GetSearchCapabilities") => soap_ok_response(
            "urn:schemas-upnp-org:service:ContentDirectory:1",
            "GetSearchCapabilities",
            "<SearchCaps></SearchCaps>",
        ),
        Some("GetSortCapabilities") => soap_ok_response(
            "urn:schemas-upnp-org:service:ContentDirectory:1",
            "GetSortCapabilities",
            "<SortCaps></SortCaps>",
        ),
        _ => soap_error(401, "Invalid Action"),
    }
}

async fn browse_action(body: String, state: &State) -> Response<BoxBody> {
    let object_id = extract_tag(&body, "ObjectID").unwrap_or_else(|| "0".to_string());
    let browse_flag =
        extract_tag(&body, "BrowseFlag").unwrap_or_else(|| "BrowseDirectChildren".to_string());
    let starting_index: usize = extract_tag(&body, "StartingIndex")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let requested_count: usize = extract_tag(&body, "RequestedCount")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0); // 0 = all

    let base_url = format!("http://{}:{}", state.local_ip, state.server_port);
    let pool = &state.pool;

    let (items, total): (Vec<String>, usize) = match (browse_flag.as_str(), object_id.as_str()) {
        // ── Root container ────────────────────────────────────────────────
        (_, "0") => {
            let n_tracks = db::count_tracks(pool).await;
            let n_albums = db::count_albums(pool).await;
            let n_artists = db::count_artists(pool).await;
            let items = vec![
                didl::simple_container("1", "0", "Music", n_tracks),
                didl::simple_container("2", "0", "Albums", n_albums),
                didl::simple_container("3", "0", "Artists", n_artists),
                didl::simple_container("4", "0", "All Tracks", n_tracks),
            ];
            let total = items.len();
            (items, total)
        }

        // ── Music category ────────────────────────────────────────────────
        (_, "1") => {
            let n_tracks = db::count_tracks(pool).await;
            let n_albums = db::count_albums(pool).await;
            let n_artists = db::count_artists(pool).await;
            let items = vec![
                didl::simple_container("2", "1", "Albums", n_albums),
                didl::simple_container("3", "1", "Artists", n_artists),
                didl::simple_container("4", "1", "All Tracks", n_tracks),
            ];
            let total = items.len();
            (items, total)
        }

        // ── Albums container ──────────────────────────────────────────────
        (_, "2") => match db::all_albums(pool).await {
            Ok(albums) => {
                let total = albums.len();
                let items: Vec<String> = albums
                    .iter()
                    .map(|a| didl::album_container(a, "2"))
                    .collect();
                (items, total)
            }
            Err(e) => {
                tracing::warn!("UPnP Browse albums: {e}");
                (vec![], 0)
            }
        },

        // ── Artists container ─────────────────────────────────────────────
        (_, "3") => match db::all_artists(pool).await {
            Ok(artists) => {
                let total = artists.len();
                let items: Vec<String> = artists
                    .iter()
                    .map(|a| didl::artist_container(a, "3"))
                    .collect();
                (items, total)
            }
            Err(e) => {
                tracing::warn!("UPnP Browse artists: {e}");
                (vec![], 0)
            }
        },

        // ── All tracks ────────────────────────────────────────────────────
        (_, "4") => match db::all_tracks(pool).await {
            Ok(tracks) => {
                let total = tracks.len();
                let items: Vec<String> = tracks
                    .iter()
                    .map(|t| didl::track_item(t, "4", &base_url))
                    .collect();
                (items, total)
            }
            Err(e) => {
                tracing::warn!("UPnP Browse all tracks: {e}");
                (vec![], 0)
            }
        },

        // ── Album children ────────────────────────────────────────────────
        (_, id) if id.starts_with("album:") => {
            let album_id = &id[6..];
            match db::tracks_by_album(pool, album_id).await {
                Ok(tracks) => {
                    let total = tracks.len();
                    let items: Vec<String> = tracks
                        .iter()
                        .map(|t| didl::track_item(t, id, &base_url))
                        .collect();
                    (items, total)
                }
                Err(e) => {
                    tracing::warn!("UPnP Browse album {album_id}: {e}");
                    (vec![], 0)
                }
            }
        }

        // ── Artist children ───────────────────────────────────────────────
        (_, id) if id.starts_with("artist:") => {
            let artist_id = &id[7..];
            match db::tracks_by_artist(pool, artist_id).await {
                Ok(tracks) => {
                    let total = tracks.len();
                    let items: Vec<String> = tracks
                        .iter()
                        .map(|t| didl::track_item(t, id, &base_url))
                        .collect();
                    (items, total)
                }
                Err(e) => {
                    tracing::warn!("UPnP Browse artist {artist_id}: {e}");
                    (vec![], 0)
                }
            }
        }

        // ── Single track metadata ─────────────────────────────────────────
        ("BrowseMetadata", id) if id.starts_with("track:") => {
            let track_id = &id[6..];
            match db::track_by_id(pool, track_id).await {
                Ok(Some(track)) => {
                    let item = didl::track_item(&track, "4", &base_url);
                    (vec![item], 1)
                }
                _ => (vec![], 0),
            }
        }

        _ => (vec![], 0),
    };

    let total = total;
    let (slice, start, count) = paginate(&items, starting_index, requested_count);
    let didl = didl::wrap_didl(slice);
    let escaped = didl::escape_for_result(&didl);
    let result_body = format!(
        "<Result>{escaped}</Result>\
         <NumberReturned>{count}</NumberReturned>\
         <TotalMatches>{total}</TotalMatches>\
         <UpdateID>1</UpdateID>"
    );
    let _ = (start,); // suppress unused warning

    soap_ok_response(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "Browse",
        &result_body,
    )
}

fn connection_manager_control(body: String, header_action: Option<String>) -> Response<BoxBody> {
    let action = resolve_action(header_action, &body);
    match action.as_deref() {
        Some("GetProtocolInfo") => soap_ok_response(
            "urn:schemas-upnp-org:service:ConnectionManager:1",
            "GetProtocolInfo",
            "<Source>\
             http-get:*:audio/mpeg:*,\
             http-get:*:audio/flac:*,\
             http-get:*:audio/ogg:*,\
             http-get:*:audio/opus:*,\
             http-get:*:audio/mp4:*,\
             http-get:*:audio/aac:*,\
             http-get:*:audio/wav:*,\
             http-get:*:audio/aiff:*,\
             http-get:*:audio/x-w64:*,\
             http-get:*:audio/x-wavpack:*,\
             http-get:*:audio/x-ape:*,\
             http-get:*:audio/x-musepack:*,\
             http-get:*:audio/ac3:*,\
             http-get:*:audio/x-ms-wma:*,\
             http-get:*:audio/x-pn-realaudio:*,\
             http-get:*:audio/x-tta:*,\
             http-get:*:audio/x-shorten:*,\
             http-get:*:audio/basic:*,\
             http-get:*:audio/x-sony-oma:*,\
             http-get:*:audio/vox:*,\
             http-get:*:audio/x-adx:*,\
             http-get:*:audio/mod:*,\
             http-get:*:audio/prs.sid:*,\
             http-get:*:audio/x-nsf:*,\
             http-get:*:audio/x-spc:*,\
             http-get:*:audio/x-asap:*,\
             http-get:*:audio/x-ay:*,\
             http-get:*:audio/x-vtx:*,\
             http-get:*:audio/x-gbs:*,\
             http-get:*:audio/x-hes:*,\
             http-get:*:audio/x-sgc:*,\
             http-get:*:audio/x-vgm:*,\
             http-get:*:audio/x-kss:*\
             </Source>\
             <Sink></Sink>",
        ),
        _ => soap_error(401, "Invalid Action"),
    }
}

// ---------------------------------------------------------------------------
// Audio file serving
// ---------------------------------------------------------------------------

async fn serve_audio(id: &str, range: Option<&str>, state: &State) -> Response<BoxBody> {
    let track = match db::track_by_id(&state.pool, id).await {
        Ok(Some(t)) => t,
        Ok(None) => return not_found(),
        Err(e) => {
            tracing::warn!("UPnP audio: db error for {id}: {e}");
            return server_error();
        }
    };

    let mut file = match File::open(&track.path).await {
        Ok(f) => f,
        Err(e) => {
            tracing::warn!("UPnP audio: open {:?}: {e}", track.path);
            return not_found();
        }
    };

    let file_size = file
        .metadata()
        .await
        .map(|m| m.len())
        .unwrap_or(track.filesize as u64);

    let ct = format::content_type_for_path(&track.path);

    // Parse a "bytes=start-[end]" Range header.
    let parsed_range = range.and_then(|r| {
        let r = r.strip_prefix("bytes=")?;
        let (s, e) = r.split_once('-')?;
        let start: u64 = s.parse().ok()?;
        let end: Option<u64> = if e.is_empty() { None } else { e.parse().ok() };
        Some((start, end))
    });

    let (start, end) = match parsed_range {
        Some((s, e)) => (s, e.unwrap_or(file_size - 1).min(file_size - 1)),
        None => (0, file_size - 1),
    };

    let length = end - start + 1;

    // Seek and read only the requested slice to avoid buffering the whole file.
    use tokio::io::{AsyncReadExt as _, AsyncSeekExt as _};
    if start > 0 {
        if file.seek(std::io::SeekFrom::Start(start)).await.is_err() {
            return server_error();
        }
    }
    let mut data = vec![0u8; length as usize];
    if file.read_exact(&mut data).await.is_err() {
        return server_error();
    }

    let (status, content_range) = if parsed_range.is_some() {
        (206u16, Some(format!("bytes {start}-{end}/{file_size}")))
    } else {
        (200, None)
    };

    let mut builder = Response::builder()
        .status(status)
        .header("Content-Type", ct)
        .header("Content-Length", data.len().to_string())
        .header("Accept-Ranges", "bytes")
        .header("TransferMode.DLNA.ORG", "Interactive");

    if let Some(cr) = content_range {
        builder = builder.header("Content-Range", cr);
    }

    builder.body(Full::from(Bytes::from(data))).unwrap()
}

async fn serve_album_art(album_id: &str, state: &State) -> Response<BoxBody> {
    // Find the album_art path from any track with this album_id.
    let art_path: Option<String> = sqlx::query_scalar(
        "SELECT album_art FROM track WHERE album_id = ? AND album_art IS NOT NULL LIMIT 1",
    )
    .bind(album_id)
    .fetch_optional(&state.pool)
    .await
    .ok()
    .flatten();

    let Some(path) = art_path else {
        return not_found();
    };

    let data = match tokio::fs::read(&path).await {
        Ok(d) => d,
        Err(_) => return not_found(),
    };

    let ct = if path.ends_with(".png") {
        "image/png"
    } else {
        "image/jpeg"
    };

    Response::builder()
        .status(200)
        .header("Content-Type", ct)
        .header("Content-Length", data.len().to_string())
        .body(Full::from(Bytes::from(data)))
        .unwrap()
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the SOAP action from the `SOAPAction` HTTP header.
/// Value looks like `"urn:schemas-upnp-org:service:ContentDirectory:1#Browse"`.
fn soap_action_from_header(headers: &http::HeaderMap) -> Option<String> {
    headers
        .get("SOAPAction")
        .or_else(|| headers.get("soapaction"))
        .and_then(|v| v.to_str().ok())
        .and_then(|s| {
            let s = s.trim().trim_matches('"');
            s.rfind('#').map(|i| s[i + 1..].to_string())
        })
}

/// Extract the SOAP action by scanning the XML body character by character.
/// Works for both compact (single-line) and pretty-printed envelopes.
fn soap_action_from_body(body: &str) -> Option<String> {
    let mut pos = 0;
    let b = body.as_bytes();
    while pos < b.len() {
        if b[pos] != b'<' {
            pos += 1;
            continue;
        }
        pos += 1;
        if pos >= b.len() {
            break;
        }
        match b[pos] {
            b'/' | b'?' | b'!' => {
                while pos < b.len() && b[pos] != b'>' {
                    pos += 1;
                }
                continue;
            }
            _ => {}
        }
        let name_start = pos;
        while pos < b.len() {
            match b[pos] {
                b'>' | b' ' | b'\t' | b'\n' | b'\r' | b'/' => break,
                _ => pos += 1,
            }
        }
        if let Ok(full) = std::str::from_utf8(&b[name_start..pos]) {
            let bare = full.split(':').last().unwrap_or(full);
            if !bare.is_empty() && !matches!(bare, "Envelope" | "Body") {
                return Some(bare.to_string());
            }
        }
    }
    None
}

fn resolve_action(header_action: Option<String>, body: &str) -> Option<String> {
    header_action.or_else(|| soap_action_from_body(body))
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

fn paginate<'a>(items: &'a [String], start: usize, count: usize) -> (&'a [String], usize, usize) {
    let total = items.len();
    let from = start.min(total);
    let slice = if count == 0 {
        &items[from..]
    } else {
        let to = (from + count).min(total);
        &items[from..to]
    };
    (slice, from, slice.len())
}

fn soap_ok_response(service_type: &str, action: &str, inner: &str) -> Response<BoxBody> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:{action}Response xmlns:u="{service_type}">
      {inner}
    </u:{action}Response>
  </s:Body>
</s:Envelope>"#
    );
    xml_response(xml)
}

fn soap_error(code: u32, description: &str) -> Response<BoxBody> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <s:Fault>
      <faultcode>s:Client</faultcode>
      <faultstring>UPnPError</faultstring>
      <detail>
        <UPnPError xmlns="urn:schemas-upnp-org:control-1-0">
          <errorCode>{code}</errorCode>
          <errorDescription>{description}</errorDescription>
        </UPnPError>
      </detail>
    </s:Fault>
  </s:Body>
</s:Envelope>"#
    );
    Response::builder()
        .status(500)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .body(Full::from(Bytes::from(xml)))
        .unwrap()
}

fn xml_response(xml: String) -> Response<BoxBody> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .body(Full::from(Bytes::from(xml)))
        .unwrap()
}

fn not_found() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(empty_body())
        .unwrap()
}

fn server_error() -> Response<BoxBody> {
    Response::builder()
        .status(StatusCode::INTERNAL_SERVER_ERROR)
        .body(empty_body())
        .unwrap()
}

fn empty_body() -> BoxBody {
    Full::from(Bytes::new())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
