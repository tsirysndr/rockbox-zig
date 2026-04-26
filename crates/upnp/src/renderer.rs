use crate::{get_local_ip, get_runtime, CONFIG};
use bytes::Bytes;
use http::{Method, Request, Response, StatusCode};
use http_body_util::{BodyExt, Full};
use hyper::body::Incoming;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto::Builder;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::{Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex, OnceLock};
use tokio::net::TcpListener;
use tokio::time::Duration;

// ---------------------------------------------------------------------------
// Renderer state
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
enum TransportState {
    NoMediaPresent,
    Stopped,
    Playing,
    PausedPlayback,
}

impl TransportState {
    fn as_str(&self) -> &'static str {
        match self {
            Self::NoMediaPresent => "NO_MEDIA_PRESENT",
            Self::Stopped => "STOPPED",
            Self::Playing => "PLAYING",
            Self::PausedPlayback => "PAUSED_PLAYBACK",
        }
    }
}

struct RendererState {
    current_uri: Option<String>,
    transport_state: TransportState,
    mute: bool,
}

static RENDERER_STATE: Mutex<RendererState> = Mutex::new(RendererState {
    current_uri: None,
    transport_state: TransportState::NoMediaPresent,
    mute: false,
});

static RENDERER_UUID: OnceLock<String> = OnceLock::new();

fn renderer_uuid() -> &'static str {
    RENDERER_UUID.get_or_init(|| uuid::Uuid::new_v4().to_string())
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

pub fn start(port: u16, friendly_name: &str) {
    let name = if friendly_name.is_empty() {
        "Rockbox".to_string()
    } else {
        friendly_name.to_string()
    };
    let _ = renderer_uuid();
    let rt = get_runtime();
    rt.spawn(async move {
        if let Err(e) = run_http(port, name).await {
            tracing::error!("UPnP renderer HTTP server error: {e}");
        }
    });
    rt.spawn(async move {
        run_ssdp(port).await;
    });
    tracing::info!("UPnP media renderer started on :{port}");
}

// ---------------------------------------------------------------------------
// HTTP server
// ---------------------------------------------------------------------------

struct State {
    friendly_name: String,
    uuid: String,
}

async fn run_http(port: u16, friendly_name: String) -> anyhow::Result<()> {
    let state = Arc::new(State {
        friendly_name,
        uuid: renderer_uuid().to_string(),
    });

    let listener = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], port))).await?;
    tracing::info!("UPnP renderer listening on :{port}");

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
                tracing::debug!("UPnP renderer: connection error: {e}");
            }
        });
    }
}

async fn handle(
    req: Request<Incoming>,
    state: Arc<State>,
) -> Result<Response<Full<Bytes>>, hyper::Error> {
    let path = req.uri().path().to_string();
    let method = req.method().clone();

    // Extract SOAPAction header before consuming the body.
    let header_action = soap_action_from_header(req.headers());

    let resp = match (method, path.as_str()) {
        (Method::GET, "/renderer/desc.xml") => renderer_description(&state),
        (Method::GET, "/AVTransport/desc.xml") => avtransport_scpd(),
        (Method::GET, "/RenderingControl/desc.xml") => rendering_control_scpd(),
        (Method::GET, "/ConnectionManager/desc.xml") => renderer_connection_manager_scpd(),
        (Method::POST, "/AVTransport/control") => {
            let body = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body).unwrap_or("").to_string();
            avtransport_control(body_str, header_action).await
        }
        (Method::POST, "/RenderingControl/control") => {
            let body = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body).unwrap_or("").to_string();
            rendering_control(body_str, header_action)
        }
        (Method::POST, "/ConnectionManager/control") => {
            let body = req.collect().await?.to_bytes();
            let body_str = std::str::from_utf8(&body).unwrap_or("").to_string();
            renderer_connection_manager(body_str, header_action)
        }
        (m, _) if m.as_str() == "SUBSCRIBE" || m.as_str() == "UNSUBSCRIBE" => Response::builder()
            .status(200)
            .header("SID", "uuid:rockbox-renderer-event-sid")
            .header("TIMEOUT", "Second-1800")
            .body(Full::from(Bytes::new()))
            .unwrap(),
        _ => not_found(),
    };
    Ok(resp)
}

// ---------------------------------------------------------------------------
// Device description
// ---------------------------------------------------------------------------

fn renderer_description(state: &State) -> Response<Full<Bytes>> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<root xmlns="urn:schemas-upnp-org:device-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <device>
    <deviceType>urn:schemas-upnp-org:device:MediaRenderer:1</deviceType>
    <friendlyName>{name} (Renderer)</friendlyName>
    <manufacturer>Rockbox</manufacturer>
    <manufacturerURL>https://www.rockbox.org</manufacturerURL>
    <modelDescription>Rockbox UPnP/DLNA Media Renderer</modelDescription>
    <modelName>Rockbox</modelName>
    <modelNumber>1.0</modelNumber>
    <UDN>uuid:{uuid}</UDN>
    <dlna:X_DLNADOC xmlns:dlna="urn:schemas-dlna-org:device-1-0">DMR-1.50</dlna:X_DLNADOC>
    <serviceList>
      <service>
        <serviceType>urn:schemas-upnp-org:service:AVTransport:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:AVTransport</serviceId>
        <SCPDURL>/AVTransport/desc.xml</SCPDURL>
        <controlURL>/AVTransport/control</controlURL>
        <eventSubURL>/AVTransport/events</eventSubURL>
      </service>
      <service>
        <serviceType>urn:schemas-upnp-org:service:RenderingControl:1</serviceType>
        <serviceId>urn:upnp-org:serviceId:RenderingControl</serviceId>
        <SCPDURL>/RenderingControl/desc.xml</SCPDURL>
        <controlURL>/RenderingControl/control</controlURL>
        <eventSubURL>/RenderingControl/events</eventSubURL>
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

fn avtransport_scpd() -> Response<Full<Bytes>> {
    xml_response(r#"<?xml version="1.0" encoding="UTF-8"?>
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <actionList>
    <action><name>SetAVTransportURI</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>CurrentURI</name><direction>in</direction><relatedStateVariable>AVTransportURI</relatedStateVariable></argument>
      <argument><name>CurrentURIMetaData</name><direction>in</direction><relatedStateVariable>AVTransportURIMetaData</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>Play</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Speed</name><direction>in</direction><relatedStateVariable>TransportPlaySpeed</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>Stop</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>Pause</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>Seek</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Unit</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_SeekMode</relatedStateVariable></argument>
      <argument><name>Target</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_SeekTarget</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>GetTransportInfo</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>CurrentTransportState</name><direction>out</direction><relatedStateVariable>TransportState</relatedStateVariable></argument>
      <argument><name>CurrentTransportStatus</name><direction>out</direction><relatedStateVariable>TransportStatus</relatedStateVariable></argument>
      <argument><name>CurrentSpeed</name><direction>out</direction><relatedStateVariable>TransportPlaySpeed</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>GetPositionInfo</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Track</name><direction>out</direction><relatedStateVariable>CurrentTrack</relatedStateVariable></argument>
      <argument><name>TrackDuration</name><direction>out</direction><relatedStateVariable>CurrentTrackDuration</relatedStateVariable></argument>
      <argument><name>TrackMetaData</name><direction>out</direction><relatedStateVariable>CurrentTrackMetaData</relatedStateVariable></argument>
      <argument><name>TrackURI</name><direction>out</direction><relatedStateVariable>CurrentTrackURI</relatedStateVariable></argument>
      <argument><name>RelTime</name><direction>out</direction><relatedStateVariable>RelativeTimePosition</relatedStateVariable></argument>
      <argument><name>AbsTime</name><direction>out</direction><relatedStateVariable>AbsoluteTimePosition</relatedStateVariable></argument>
      <argument><name>RelCount</name><direction>out</direction><relatedStateVariable>RelativeCounterPosition</relatedStateVariable></argument>
      <argument><name>AbsCount</name><direction>out</direction><relatedStateVariable>AbsoluteCounterPosition</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>GetMediaInfo</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>NrTracks</name><direction>out</direction><relatedStateVariable>NumberOfTracks</relatedStateVariable></argument>
      <argument><name>MediaDuration</name><direction>out</direction><relatedStateVariable>CurrentMediaDuration</relatedStateVariable></argument>
      <argument><name>CurrentURI</name><direction>out</direction><relatedStateVariable>AVTransportURI</relatedStateVariable></argument>
      <argument><name>CurrentURIMetaData</name><direction>out</direction><relatedStateVariable>AVTransportURIMetaData</relatedStateVariable></argument>
      <argument><name>NextURI</name><direction>out</direction><relatedStateVariable>NextAVTransportURI</relatedStateVariable></argument>
      <argument><name>NextURIMetaData</name><direction>out</direction><relatedStateVariable>NextAVTransportURIMetaData</relatedStateVariable></argument>
      <argument><name>PlayMedium</name><direction>out</direction><relatedStateVariable>PlaybackStorageMedium</relatedStateVariable></argument>
      <argument><name>RecordMedium</name><direction>out</direction><relatedStateVariable>RecordStorageMedium</relatedStateVariable></argument>
      <argument><name>WriteStatus</name><direction>out</direction><relatedStateVariable>RecordMediumWriteStatus</relatedStateVariable></argument>
    </argumentList></action>
  </actionList>
  <serviceStateTable>
    <stateVariable sendEvents="yes"><name>TransportState</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>TransportStatus</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>TransportPlaySpeed</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>AVTransportURI</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>AVTransportURIMetaData</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>NextAVTransportURI</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>NextAVTransportURIMetaData</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentTrack</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentTrackDuration</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentTrackMetaData</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentTrackURI</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>RelativeTimePosition</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>AbsoluteTimePosition</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>RelativeCounterPosition</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>AbsoluteCounterPosition</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>NumberOfTracks</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentMediaDuration</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>PlaybackStorageMedium</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>RecordStorageMedium</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>RecordMediumWriteStatus</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_InstanceID</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_SeekMode</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_SeekTarget</name><dataType>string</dataType></stateVariable>
  </serviceStateTable>
</scpd>"#.to_string())
}

fn rendering_control_scpd() -> Response<Full<Bytes>> {
    xml_response(r#"<?xml version="1.0" encoding="UTF-8"?>
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <actionList>
    <action><name>GetVolume</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Channel</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Channel</relatedStateVariable></argument>
      <argument><name>CurrentVolume</name><direction>out</direction><relatedStateVariable>Volume</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>SetVolume</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Channel</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Channel</relatedStateVariable></argument>
      <argument><name>DesiredVolume</name><direction>in</direction><relatedStateVariable>Volume</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>GetMute</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Channel</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Channel</relatedStateVariable></argument>
      <argument><name>CurrentMute</name><direction>out</direction><relatedStateVariable>Mute</relatedStateVariable></argument>
    </argumentList></action>
    <action><name>SetMute</name><argumentList>
      <argument><name>InstanceID</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_InstanceID</relatedStateVariable></argument>
      <argument><name>Channel</name><direction>in</direction><relatedStateVariable>A_ARG_TYPE_Channel</relatedStateVariable></argument>
      <argument><name>DesiredMute</name><direction>in</direction><relatedStateVariable>Mute</relatedStateVariable></argument>
    </argumentList></action>
  </actionList>
  <serviceStateTable>
    <stateVariable sendEvents="yes"><name>Volume</name><dataType>ui2</dataType><allowedValueRange><minimum>0</minimum><maximum>100</maximum><step>1</step></allowedValueRange></stateVariable>
    <stateVariable sendEvents="yes"><name>Mute</name><dataType>boolean</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_InstanceID</name><dataType>ui4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Channel</name><dataType>string</dataType></stateVariable>
  </serviceStateTable>
</scpd>"#.to_string())
}

fn renderer_connection_manager_scpd() -> Response<Full<Bytes>> {
    xml_response(r#"<?xml version="1.0" encoding="UTF-8"?>
<scpd xmlns="urn:schemas-upnp-org:service-1-0">
  <specVersion><major>1</major><minor>0</minor></specVersion>
  <actionList>
    <action><name>GetProtocolInfo</name><argumentList>
      <argument><name>Source</name><direction>out</direction><relatedStateVariable>SourceProtocolInfo</relatedStateVariable></argument>
      <argument><name>Sink</name><direction>out</direction><relatedStateVariable>SinkProtocolInfo</relatedStateVariable></argument>
    </argumentList></action>
  </actionList>
  <serviceStateTable>
    <stateVariable sendEvents="yes"><name>SourceProtocolInfo</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="yes"><name>SinkProtocolInfo</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>CurrentConnectionIDs</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionStatus</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ConnectionID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_AVTransportID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_RcsID</name><dataType>i4</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_Direction</name><dataType>string</dataType></stateVariable>
    <stateVariable sendEvents="no"><name>A_ARG_TYPE_ProtocolInfo</name><dataType>string</dataType></stateVariable>
  </serviceStateTable>
</scpd>"#.to_string())
}

// ---------------------------------------------------------------------------
// AVTransport SOAP
// ---------------------------------------------------------------------------

async fn avtransport_control(body: String, header_action: Option<String>) -> Response<Full<Bytes>> {
    let action = resolve_action(header_action, &body);
    match action.as_deref() {
        Some("SetAVTransportURI") => {
            let uri = extract_tag(&body, "CurrentURI").unwrap_or_default();
            if uri.is_empty() {
                return soap_error(402, "Invalid Args");
            }
            tracing::info!("UPnP renderer: SetAVTransportURI = {uri}");
            {
                let mut st = RENDERER_STATE.lock().unwrap();
                st.current_uri = Some(uri);
                st.transport_state = TransportState::Stopped;
            }
            soap_ok(
                "urn:schemas-upnp-org:service:AVTransport:1",
                "SetAVTransportURI",
                "",
            )
        }

        Some("Play") => {
            let uri = {
                let st = RENDERER_STATE.lock().unwrap();
                st.current_uri.clone()
            };
            match uri {
                None => soap_error(701, "Transition not available"),
                Some(uri) => {
                    tracing::info!("UPnP renderer: Play {uri}");
                    tokio::task::spawn_blocking(move || {
                        play_url(&uri);
                    })
                    .await
                    .ok();
                    RENDERER_STATE.lock().unwrap().transport_state = TransportState::Playing;
                    soap_ok("urn:schemas-upnp-org:service:AVTransport:1", "Play", "")
                }
            }
        }

        Some("Stop") => {
            tracing::info!("UPnP renderer: Stop");
            tokio::task::spawn_blocking(|| {
                stop_playback();
            })
            .await
            .ok();
            RENDERER_STATE.lock().unwrap().transport_state = TransportState::Stopped;
            soap_ok("urn:schemas-upnp-org:service:AVTransport:1", "Stop", "")
        }

        Some("Pause") => {
            let state = RENDERER_STATE.lock().unwrap().transport_state.clone();
            match state {
                TransportState::Playing => {
                    tokio::task::spawn_blocking(|| {
                        pause_playback();
                    })
                    .await
                    .ok();
                    RENDERER_STATE.lock().unwrap().transport_state = TransportState::PausedPlayback;
                    soap_ok("urn:schemas-upnp-org:service:AVTransport:1", "Pause", "")
                }
                TransportState::PausedPlayback => {
                    tokio::task::spawn_blocking(|| {
                        resume_playback();
                    })
                    .await
                    .ok();
                    RENDERER_STATE.lock().unwrap().transport_state = TransportState::Playing;
                    soap_ok("urn:schemas-upnp-org:service:AVTransport:1", "Pause", "")
                }
                _ => soap_error(701, "Transition not available"),
            }
        }

        Some("Seek") => {
            let unit = extract_tag(&body, "Unit").unwrap_or_default();
            let target = extract_tag(&body, "Target").unwrap_or_default();
            if unit == "REL_TIME" || unit == "ABS_TIME" {
                if let Some(ms) = parse_time_to_ms(&target) {
                    tokio::task::spawn_blocking(move || {
                        seek_to(ms);
                    })
                    .await
                    .ok();
                    return soap_ok("urn:schemas-upnp-org:service:AVTransport:1", "Seek", "");
                }
            }
            soap_error(711, "Illegal seek target")
        }

        Some("GetTransportInfo") => {
            let state = RENDERER_STATE.lock().unwrap().transport_state.clone();
            let inner = format!(
                "<CurrentTransportState>{}</CurrentTransportState>\
                 <CurrentTransportStatus>OK</CurrentTransportStatus>\
                 <CurrentSpeed>1</CurrentSpeed>",
                state.as_str()
            );
            soap_ok(
                "urn:schemas-upnp-org:service:AVTransport:1",
                "GetTransportInfo",
                &inner,
            )
        }

        Some("GetPositionInfo") => {
            let (uri, elapsed_ms, duration_ms) = get_position_info();
            let rel_time = ms_to_time(elapsed_ms);
            let track_duration = ms_to_time(duration_ms);
            let inner = format!(
                "<Track>1</Track>\
                 <TrackDuration>{track_duration}</TrackDuration>\
                 <TrackMetaData></TrackMetaData>\
                 <TrackURI>{uri}</TrackURI>\
                 <RelTime>{rel_time}</RelTime>\
                 <AbsTime>{rel_time}</AbsTime>\
                 <RelCount>0</RelCount>\
                 <AbsCount>0</AbsCount>",
                uri = xml_escape(&uri),
            );
            soap_ok(
                "urn:schemas-upnp-org:service:AVTransport:1",
                "GetPositionInfo",
                &inner,
            )
        }

        Some("GetMediaInfo") => {
            let (uri, _, duration_ms) = get_position_info();
            let duration = ms_to_time(duration_ms);
            let inner = format!(
                "<NrTracks>1</NrTracks>\
                 <MediaDuration>{duration}</MediaDuration>\
                 <CurrentURI>{uri}</CurrentURI>\
                 <CurrentURIMetaData></CurrentURIMetaData>\
                 <NextURI></NextURI>\
                 <NextURIMetaData></NextURIMetaData>\
                 <PlayMedium>NETWORK</PlayMedium>\
                 <RecordMedium>NOT_IMPLEMENTED</RecordMedium>\
                 <WriteStatus>NOT_IMPLEMENTED</WriteStatus>",
                uri = xml_escape(&uri),
            );
            soap_ok(
                "urn:schemas-upnp-org:service:AVTransport:1",
                "GetMediaInfo",
                &inner,
            )
        }

        _ => soap_error(401, "Invalid Action"),
    }
}

// ---------------------------------------------------------------------------
// RenderingControl SOAP
// ---------------------------------------------------------------------------

fn rendering_control(body: String, header_action: Option<String>) -> Response<Full<Bytes>> {
    let action = resolve_action(header_action, &body);
    match action.as_deref() {
        Some("GetVolume") => {
            let vol = current_volume_pct();
            let inner = format!("<CurrentVolume>{vol}</CurrentVolume>");
            soap_ok(
                "urn:schemas-upnp-org:service:RenderingControl:1",
                "GetVolume",
                &inner,
            )
        }

        Some("SetVolume") => {
            let desired: i32 = extract_tag(&body, "DesiredVolume")
                .and_then(|v| v.parse().ok())
                .unwrap_or(50)
                .clamp(0, 100);
            set_volume_pct(desired);
            soap_ok(
                "urn:schemas-upnp-org:service:RenderingControl:1",
                "SetVolume",
                "",
            )
        }

        Some("GetMute") => {
            let mute = RENDERER_STATE.lock().unwrap().mute;
            let inner = format!(
                "<CurrentMute>{}</CurrentMute>",
                if mute { "1" } else { "0" }
            );
            soap_ok(
                "urn:schemas-upnp-org:service:RenderingControl:1",
                "GetMute",
                &inner,
            )
        }

        Some("SetMute") => {
            let desired = extract_tag(&body, "DesiredMute")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(false);
            RENDERER_STATE.lock().unwrap().mute = desired;
            // Rockbox doesn't have a dedicated mute — drive volume to min when muting.
            if desired {
                let min = rockbox_sys::sound::min(0);
                rockbox_sys::sound::set(0, min);
            } else {
                let vol = current_volume_pct();
                set_volume_pct(vol);
            }
            soap_ok(
                "urn:schemas-upnp-org:service:RenderingControl:1",
                "SetMute",
                "",
            )
        }

        _ => soap_error(401, "Invalid Action"),
    }
}

fn renderer_connection_manager(
    body: String,
    header_action: Option<String>,
) -> Response<Full<Bytes>> {
    let action = resolve_action(header_action, &body);
    if matches!(action.as_deref(), Some("GetProtocolInfo")) {
        // Build sink protocol info from all supported Rockbox formats.
        let formats = [
            "audio/mpeg",
            "audio/flac",
            "audio/ogg",
            "audio/opus",
            "audio/mp4",
            "audio/aac",
            "audio/wav",
            "audio/aiff",
            "audio/x-w64",
            "audio/x-wavpack",
            "audio/x-ape",
            "audio/x-musepack",
            "audio/ac3",
            "audio/x-ms-wma",
            "audio/x-pn-realaudio",
            "audio/x-tta",
            "audio/x-shorten",
            "audio/basic",
            "audio/x-sony-oma",
            "audio/vox",
            "audio/x-adx",
            "audio/mod",
        ];
        let sink: Vec<String> = formats
            .iter()
            .map(|m| format!("http-get:*:{m}:*"))
            .collect();
        let inner = format!("<Source></Source><Sink>{}</Sink>", sink.join(","));
        return soap_ok(
            "urn:schemas-upnp-org:service:ConnectionManager:1",
            "GetProtocolInfo",
            &inner,
        );
    }
    soap_error(401, "Invalid Action")
}

// ---------------------------------------------------------------------------
// Playback helpers — call into rockbox-sys via spawn_blocking
// ---------------------------------------------------------------------------

fn play_url(uri: &str) {
    use rockbox_sys::{playback, playlist};
    playlist::build_playlist(vec![uri], 0, 1);
    playlist::start(0, 0, 0);
    playback::play(0, 0);
}

fn stop_playback() {
    rockbox_sys::playback::hard_stop();
}

fn pause_playback() {
    rockbox_sys::playback::pause();
}

fn resume_playback() {
    rockbox_sys::playback::resume();
}

fn seek_to(ms: i32) {
    rockbox_sys::playback::ff_rewind(ms);
}

fn get_position_info() -> (String, i32, i32) {
    let uri = RENDERER_STATE
        .lock()
        .unwrap()
        .current_uri
        .clone()
        .unwrap_or_default();

    let (elapsed, duration) = match rockbox_sys::playback::current_track() {
        Some(track) => (track.elapsed as i32, track.length as i32),
        None => (0, 0),
    };
    (uri, elapsed, duration)
}

// SOUND_VOLUME = 0 in Rockbox (first entry in the sound settings enum).
const SOUND_VOLUME: i32 = 0;

fn current_volume_pct() -> i32 {
    let cur = rockbox_sys::sound::current(SOUND_VOLUME);
    let min = rockbox_sys::sound::min(SOUND_VOLUME);
    let max = rockbox_sys::sound::max(SOUND_VOLUME);
    if max == min {
        return 50;
    }
    ((cur - min) * 100) / (max - min)
}

fn set_volume_pct(pct: i32) {
    let min = rockbox_sys::sound::min(SOUND_VOLUME);
    let max = rockbox_sys::sound::max(SOUND_VOLUME);
    let val = min + (pct.clamp(0, 100) * (max - min)) / 100;
    rockbox_sys::sound::set(SOUND_VOLUME, val);
}

// ---------------------------------------------------------------------------
// SSDP for MediaRenderer:1
// ---------------------------------------------------------------------------

async fn run_ssdp(http_port: u16) {
    let std_sock = match create_ssdp_socket() {
        Ok(s) => s,
        Err(e) => {
            tracing::warn!("UPnP renderer SSDP: could not bind port 1900: {e}");
            return;
        }
    };
    let socket = match tokio::net::UdpSocket::from_std(std_sock) {
        Ok(s) => s,
        Err(e) => {
            tracing::error!("UPnP renderer SSDP: tokio socket error: {e}");
            return;
        }
    };

    send_notify_alive(&socket, http_port).await;
    tracing::info!("UPnP renderer SSDP advertising on 239.255.255.250:1900");

    let mut interval = tokio::time::interval(Duration::from_secs(900));
    interval.tick().await;
    let mut buf = vec![0u8; 2048];

    loop {
        tokio::select! {
            result = socket.recv_from(&mut buf) => {
                match result {
                    Ok((len, from)) => {
                        if let Ok(msg) = std::str::from_utf8(&buf[..len]) {
                            handle_msearch(msg, from, &socket, http_port).await;
                        }
                    }
                    Err(e) => tracing::debug!("UPnP renderer SSDP recv error: {e}"),
                }
            }
            _ = interval.tick() => {
                send_notify_alive(&socket, http_port).await;
            }
        }
    }
}

async fn handle_msearch(
    msg: &str,
    from: std::net::SocketAddr,
    socket: &tokio::net::UdpSocket,
    http_port: u16,
) {
    if !msg.starts_with("M-SEARCH") {
        return;
    }
    let st = msg
        .lines()
        .find(|l| l.to_ascii_uppercase().starts_with("ST:"))
        .map(|l| l[3..].trim())
        .unwrap_or("");

    let want = matches!(
        st,
        "ssdp:all"
            | "upnp:rootdevice"
            | "urn:schemas-upnp-org:device:MediaRenderer:1"
            | "urn:schemas-upnp-org:service:AVTransport:1"
            | "urn:schemas-upnp-org:service:RenderingControl:1"
    ) || st.starts_with("uuid:");

    if !want {
        return;
    }

    tokio::time::sleep(Duration::from_millis(100)).await;

    let ip = get_local_ip();
    let uuid = renderer_uuid();
    let location = format!("http://{}:{}/renderer/desc.xml", ip, http_port);
    let friendly = CONFIG
        .lock()
        .map(|c| c.friendly_name.clone())
        .unwrap_or_default();

    for (nt, usn) in renderer_nt_usn_pairs(uuid) {
        let response = format!(
            "HTTP/1.1 200 OK\r\n\
             CACHE-CONTROL: max-age=1800\r\n\
             EXT:\r\n\
             LOCATION: {location}\r\n\
             SERVER: Linux/1.0 UPnP/1.0 Rockbox/1.0\r\n\
             ST: {nt}\r\n\
             USN: {usn}\r\n\
             X-Friendly-Name: {friendly} (Renderer)\r\n\
             \r\n"
        );
        socket.send_to(response.as_bytes(), from).await.ok();
    }
}

async fn send_notify_alive(socket: &tokio::net::UdpSocket, http_port: u16) {
    let ip = get_local_ip();
    let uuid = renderer_uuid();
    let location = format!("http://{}:{}/renderer/desc.xml", ip, http_port);
    let dest: std::net::SocketAddr =
        std::net::SocketAddr::from((Ipv4Addr::new(239, 255, 255, 250), 1900u16));

    for (nt, usn) in renderer_nt_usn_pairs(uuid) {
        let notify = format!(
            "NOTIFY * HTTP/1.1\r\n\
             HOST: 239.255.255.250:1900\r\n\
             CACHE-CONTROL: max-age=1800\r\n\
             LOCATION: {location}\r\n\
             NT: {nt}\r\n\
             NTS: ssdp:alive\r\n\
             SERVER: Linux/1.0 UPnP/1.0 Rockbox/1.0\r\n\
             USN: {usn}\r\n\
             \r\n"
        );
        for _ in 0..3 {
            socket.send_to(notify.as_bytes(), dest).await.ok();
        }
    }
}

fn renderer_nt_usn_pairs(uuid: &str) -> Vec<(String, String)> {
    let dev = "urn:schemas-upnp-org:device:MediaRenderer:1";
    let avt = "urn:schemas-upnp-org:service:AVTransport:1";
    let rc = "urn:schemas-upnp-org:service:RenderingControl:1";
    let cm = "urn:schemas-upnp-org:service:ConnectionManager:1";
    vec![
        (
            "upnp:rootdevice".into(),
            format!("uuid:{uuid}::upnp:rootdevice"),
        ),
        (format!("uuid:{uuid}"), format!("uuid:{uuid}")),
        (dev.into(), format!("uuid:{uuid}::{dev}")),
        (avt.into(), format!("uuid:{uuid}::{avt}")),
        (rc.into(), format!("uuid:{uuid}::{rc}")),
        (cm.into(), format!("uuid:{uuid}::{cm}")),
    ]
}

fn create_ssdp_socket() -> std::io::Result<std::net::UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, Some(Protocol::UDP))?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true)?;
    socket.bind(&SocketAddr::from((Ipv4Addr::UNSPECIFIED, 1900u16)).into())?;
    socket.join_multicast_v4(&Ipv4Addr::new(239, 255, 255, 250), &Ipv4Addr::UNSPECIFIED)?;
    socket.set_multicast_loop_v4(true)?;
    socket.set_nonblocking(true)?;
    Ok(socket.into())
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Extract the SOAP action from the `SOAPAction` HTTP header.
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

/// Parse `h:mm:ss[.mmm]` into milliseconds.
fn parse_time_to_ms(s: &str) -> Option<i32> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() < 3 {
        return None;
    }
    let h: i32 = parts[0].parse().ok()?;
    let m: i32 = parts[1].parse().ok()?;
    let s_parts: Vec<&str> = parts[2].splitn(2, '.').collect();
    let s: i32 = s_parts[0].parse().ok()?;
    let ms: i32 = if s_parts.len() > 1 {
        let frac = format!("{:0<3}", s_parts[1]);
        frac[..3].parse().ok()?
    } else {
        0
    };
    Some((h * 3600 + m * 60 + s) * 1000 + ms)
}

/// Format milliseconds as `h:mm:ss.mmm`.
fn ms_to_time(ms: i32) -> String {
    let ms = ms.max(0) as u64;
    let millis = ms % 1000;
    let secs = ms / 1000;
    let mins = secs / 60;
    let secs = secs % 60;
    let hours = mins / 60;
    let mins = mins % 60;
    format!("{hours}:{mins:02}:{secs:02}.{millis:03}")
}

fn soap_ok(service: &str, action: &str, inner: &str) -> Response<Full<Bytes>> {
    let xml = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/"
            s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:{action}Response xmlns:u="{service}">{inner}</u:{action}Response>
  </s:Body>
</s:Envelope>"#
    );
    xml_response(xml)
}

fn soap_error(code: u32, description: &str) -> Response<Full<Bytes>> {
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

fn xml_response(xml: String) -> Response<Full<Bytes>> {
    Response::builder()
        .status(200)
        .header("Content-Type", "text/xml; charset=\"utf-8\"")
        .body(Full::from(Bytes::from(xml)))
        .unwrap()
}

fn not_found() -> Response<Full<Bytes>> {
    Response::builder()
        .status(StatusCode::NOT_FOUND)
        .body(Full::from(Bytes::new()))
        .unwrap()
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
