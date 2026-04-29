use std::collections::HashSet;
use std::time::Duration;
use tokio::net::UdpSocket;

const SSDP_ADDR: &str = "239.255.255.250:1900";
const DISCOVERY_TIMEOUT: Duration = Duration::from_secs(3);

#[derive(Debug, Clone)]
pub struct DiscoveredDevice {
    pub friendly_name: String,
    pub control_url: String,
}

#[derive(Debug, Clone)]
pub struct ContentEntry {
    pub id: String,
    pub title: String,
    pub is_container: bool,
    pub uri: Option<String>,
}

pub async fn discover_media_servers() -> Vec<DiscoveredDevice> {
    let socket = match UdpSocket::bind("0.0.0.0:0").await {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("UPnP control point: bind failed: {e}");
            return vec![];
        }
    };
    if let Err(e) = socket.set_multicast_ttl_v4(4) {
        tracing::debug!("UPnP control point: set_multicast_ttl_v4 failed: {e}");
    }

    let msearch = concat!(
        "M-SEARCH * HTTP/1.1\r\n",
        "HOST: 239.255.255.250:1900\r\n",
        "MAN: \"ssdp:discover\"\r\n",
        "ST: urn:schemas-upnp-org:device:MediaServer:1\r\n",
        "MX: 2\r\n",
        "\r\n"
    );

    if let Err(e) = socket.send_to(msearch.as_bytes(), SSDP_ADDR).await {
        tracing::warn!("UPnP control point: M-SEARCH failed: {e}");
        return vec![];
    }

    let mut locations: HashSet<String> = HashSet::new();
    let mut buf = vec![0u8; 4096];
    let deadline = tokio::time::Instant::now() + DISCOVERY_TIMEOUT;

    loop {
        let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
        if remaining.is_zero() {
            break;
        }
        match tokio::time::timeout(remaining, socket.recv_from(&mut buf)).await {
            Ok(Ok((len, _))) => {
                let msg = std::str::from_utf8(&buf[..len]).unwrap_or("");
                if let Some(loc) = header_value(msg, "LOCATION") {
                    locations.insert(loc.to_string());
                }
            }
            _ => break,
        }
    }

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .unwrap_or_default();

    let mut devices = vec![];
    for location in &locations {
        if let Some(device) = fetch_device_info(&client, location).await {
            devices.push(device);
        }
    }
    devices
}

async fn fetch_device_info(client: &reqwest::Client, location: &str) -> Option<DiscoveredDevice> {
    let xml = client.get(location).send().await.ok()?.text().await.ok()?;
    let friendly_name = extract_tag(&xml, "friendlyName")?;
    let control_url_rel = extract_service_control_url(&xml, "ContentDirectory")?;
    let base = url_base(location)?;
    let control_url = if control_url_rel.starts_with("http") {
        control_url_rel
    } else {
        format!(
            "{}/{}",
            base.trim_end_matches('/'),
            control_url_rel.trim_start_matches('/')
        )
    };
    Some(DiscoveredDevice {
        friendly_name,
        control_url,
    })
}

pub async fn browse_content_directory(control_url: &str, object_id: &str) -> Vec<ContentEntry> {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
    {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    let soap_body = format!(
        r#"<?xml version="1.0"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1">
      <ObjectID>{}</ObjectID>
      <BrowseFlag>BrowseDirectChildren</BrowseFlag>
      <Filter>*</Filter>
      <StartingIndex>0</StartingIndex>
      <RequestedCount>0</RequestedCount>
      <SortCriteria></SortCriteria>
    </u:Browse>
  </s:Body>
</s:Envelope>"#,
        xml_escape(object_id)
    );

    let resp = match client
        .post(control_url)
        .header("Content-Type", r#"text/xml; charset="utf-8""#)
        .header(
            "SOAPAction",
            r#""urn:schemas-upnp-org:service:ContentDirectory:1#Browse""#,
        )
        .body(soap_body)
        .send()
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("UPnP browse {control_url}: {e}");
            return vec![];
        }
    };

    let body = match resp.text().await {
        Ok(b) => b,
        Err(_) => return vec![],
    };

    tracing::debug!("UPnP browse response body: {body}");

    let result_xml = match extract_result_tag(&body) {
        Some(r) => html_unescape(&r),
        None => {
            tracing::warn!("UPnP browse: no <Result> tag found in response");
            return vec![];
        }
    };

    tracing::debug!("UPnP DIDL-Lite XML: {result_xml}");

    let entries = parse_didl_lite(&result_xml);
    tracing::debug!("UPnP parsed {} entries", entries.len());
    entries
}

// ── DIDL-Lite parser ──────────────────────────────────────────────────────────

fn parse_didl_lite(xml: &str) -> Vec<ContentEntry> {
    let mut entries = Vec::new();
    parse_elements(xml, "container", true, &mut entries);
    parse_elements(xml, "item", false, &mut entries);
    entries
}

fn parse_elements(xml: &str, tag: &str, is_container: bool, out: &mut Vec<ContentEntry>) {
    let open_prefix = format!("<{tag} ");
    let open_empty = format!("<{tag}>");
    let close_tag = format!("</{tag}>");
    let mut pos = 0;
    loop {
        let rel = xml[pos..]
            .find(&open_prefix)
            .map(|i| (i, false))
            .into_iter()
            .chain(xml[pos..].find(&open_empty).map(|i| (i, true)))
            .min_by_key(|(i, _)| *i);
        let (start_rel, _) = match rel {
            Some(v) => v,
            None => break,
        };
        let abs_start = pos + start_rel;
        let end_abs = match xml[abs_start..].find(&close_tag) {
            Some(p) => abs_start + p + close_tag.len(),
            None => break,
        };
        let element = &xml[abs_start..end_abs];
        let id = attr_value(element, "id").unwrap_or_default();
        let title = extract_namespaced(element, "title").unwrap_or_default();
        let uri = if is_container {
            None
        } else {
            extract_res_uri(element)
        };
        out.push(ContentEntry {
            id,
            title,
            is_container,
            uri,
        });
        pos = end_abs;
    }
}

// ── XML/HTML helpers ──────────────────────────────────────────────────────────

/// Extracts the content of the SOAP <Result> element, handling optional namespace prefixes
/// like <u:Result> that some UPnP servers emit.
fn extract_result_tag(xml: &str) -> Option<String> {
    extract_tag(xml, "Result").or_else(|| extract_namespaced(xml, "Result"))
}

fn extract_tag(xml: &str, tag: &str) -> Option<String> {
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let start = xml.find(&open)? + open.len();
    let end = xml[start..].find(&close)?;
    Some(xml[start..start + end].trim().to_string())
}

fn extract_namespaced(xml: &str, local: &str) -> Option<String> {
    if let Some(v) = extract_tag(xml, local) {
        return Some(v);
    }
    // Find <prefix:local>content</prefix:local>
    let open_suffix = format!(":{local}>");
    let content_start = xml.find(&open_suffix)? + open_suffix.len();
    let remaining = &xml[content_start..];
    let mut p = 0;
    while let Some(close_rel) = remaining[p..].find("</") {
        let abs = p + close_rel;
        let after_slash = &remaining[abs + 2..];
        if let Some(colon) = after_slash.find(':') {
            let after_colon = &after_slash[colon + 1..];
            if after_colon.starts_with(local) {
                let after_local = &after_colon[local.len()..];
                if after_local.starts_with('>') {
                    return Some(remaining[..abs].trim().to_string());
                }
            }
        }
        p = abs + 2;
    }
    None
}

fn extract_res_uri(element: &str) -> Option<String> {
    let res_start = element.find("<res ")?;
    let content_start = element[res_start..].find('>')? + res_start + 1;
    let content_end = element[content_start..].find("</res>")?;
    let uri = element[content_start..content_start + content_end].trim();
    if uri.starts_with("http") {
        Some(uri.to_string())
    } else {
        None
    }
}

fn extract_service_control_url(xml: &str, service_type_fragment: &str) -> Option<String> {
    let needle = format!(":{service_type_fragment}:");
    let svc_pos = xml.find(&needle)?;
    // Walk forward to find <controlURL>
    let after = &xml[svc_pos..];
    extract_tag(after, "controlURL")
}

fn header_value<'a>(msg: &'a str, name: &str) -> Option<&'a str> {
    let prefix = format!("{name}:");
    msg.lines()
        .find(|l| {
            l.to_ascii_uppercase()
                .starts_with(&prefix.to_ascii_uppercase())
        })
        .map(|l| l[prefix.len()..].trim())
}

fn attr_value(element: &str, name: &str) -> Option<String> {
    let needle = format!(r#"{name}=""#);
    let start = element.find(&needle)? + needle.len();
    let end = element[start..].find('"')?;
    Some(element[start..start + end].to_string())
}

fn url_base(url: &str) -> Option<String> {
    let after_scheme = url.find("://")?;
    let host_start = after_scheme + 3;
    let host_end = url[host_start..]
        .find('/')
        .map(|i| host_start + i)
        .unwrap_or(url.len());
    Some(url[..host_end].to_string())
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn html_unescape(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'")
}

// ── UPnP path encoding ────────────────────────────────────────────────────────

pub fn percent_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push('%');
                out.push(char::from_digit((b >> 4) as u32, 16).unwrap());
                out.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
            }
        }
    }
    out
}

pub fn percent_decode(s: &str) -> String {
    let mut out = Vec::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(hi), Some(lo)) = (
                (bytes[i + 1] as char).to_digit(16),
                (bytes[i + 2] as char).to_digit(16),
            ) {
                out.push(((hi << 4) | lo) as u8);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}
