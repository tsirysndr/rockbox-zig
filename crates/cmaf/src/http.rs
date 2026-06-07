//! Axum-based HTTP server serving the CMAF manifests + segments.
//!
//! Routes:
//!   GET /                         → 302 → /hls/master.m3u8 (handy default)
//!   GET /hls/master.m3u8          → master playlist
//!   GET /hls/audio.m3u8           → media playlist (sliding window)
//!   GET /dash/manifest.mpd        → DASH MPD
//!   GET /init.mp4                 → init segment
//!   GET /seg/{n}.m4s              → media segment {n}
//!
//! Runs on a dedicated single-thread Tokio runtime so it can coexist with
//! the actix-web runtime used by `crates/server` without fighting over the
//! global runtime.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Redirect, Response},
    routing::get,
    Router,
};

use crate::{dash, hls, SegmentStore};

#[derive(Clone)]
struct AppState {
    store: Arc<SegmentStore>,
}

pub(crate) fn serve(port: u16, store: Arc<SegmentStore>) {
    let rt = match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            tracing::error!("cmaf/http: build tokio runtime: {e}");
            return;
        }
    };

    rt.block_on(async move {
        let app = Router::new()
            .route("/", get(redirect_root))
            .route("/hls/master.m3u8", get(master_m3u8))
            .route("/hls/audio.m3u8", get(audio_m3u8))
            .route("/dash/manifest.mpd", get(dash_mpd))
            .route("/init.mp4", get(init_mp4))
            .route("/seg/:name", get(segment))
            .with_state(AppState { store });

        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(l) => l,
            Err(e) => {
                tracing::error!("cmaf/http: bind :{port} failed: {e}");
                return;
            }
        };
        tracing::info!("cmaf/http: listening on :{port}");
        if let Err(e) = axum::serve(listener, app).await {
            tracing::error!("cmaf/http: serve error: {e}");
        }
    });
}

async fn redirect_root() -> impl IntoResponse {
    Redirect::to("/hls/master.m3u8")
}

async fn master_m3u8() -> impl IntoResponse {
    text_response(hls::master_m3u8(), "application/vnd.apple.mpegurl")
}

async fn audio_m3u8(State(s): State<AppState>) -> impl IntoResponse {
    let (media_seq, segs, _) = s.store.snapshot();
    text_response(
        hls::audio_m3u8(media_seq, &segs),
        "application/vnd.apple.mpegurl",
    )
}

async fn dash_mpd(State(s): State<AppState>) -> impl IntoResponse {
    let (media_seq, _, start_ms) = s.store.snapshot();
    text_response(
        dash::manifest_mpd(start_ms, media_seq),
        "application/dash+xml",
    )
}

async fn init_mp4(State(s): State<AppState>) -> Response {
    match s.store.init() {
        Some(init) => binary_response(StatusCode::OK, (*init).clone(), "audio/mp4"),
        None => (StatusCode::SERVICE_UNAVAILABLE, "init not ready").into_response(),
    }
}

async fn segment(State(s): State<AppState>, Path(name): Path<String>) -> Response {
    let Some(num_str) = name.strip_suffix(".m4s") else {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    };
    let Ok(n) = num_str.parse::<u64>() else {
        return (StatusCode::BAD_REQUEST, "invalid segment number").into_response();
    };
    match s.store.get(n) {
        Some(seg) => binary_response(StatusCode::OK, (*seg).clone(), "audio/mp4"),
        None => (StatusCode::NOT_FOUND, "segment not available").into_response(),
    }
}

fn text_response(body: String, content_type: &'static str) -> Response {
    let mut resp = Response::new(Body::from(body));
    *resp.status_mut() = StatusCode::OK;
    let h = resp.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    resp
}

fn binary_response(status: StatusCode, bytes: Vec<u8>, content_type: &'static str) -> Response {
    let mut resp = Response::new(Body::from(bytes));
    *resp.status_mut() = status;
    let h = resp.headers_mut();
    h.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    h.insert(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-cache, no-store"),
    );
    h.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        HeaderValue::from_static("*"),
    );
    resp
}
