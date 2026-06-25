//! Admin web UI served at /admin/*.
//!
//! The dashboard is a React SPA that talks directly to this S3 endpoint via
//! SigV4-signed requests (AWS SDK v3 in the browser). The Rust side only
//! serves the embedded static bundle — no separate JSON API.

use actix_web::http::header;
use actix_web::{web, HttpRequest, HttpResponse};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "$CARGO_MANIFEST_DIR/s3webui/dist"]
struct Assets;

pub fn configure(cfg: &mut web::ServiceConfig) {
    // Bare /admin → redirect to /admin/ so the SPA's relative asset paths
    // (base="/admin/") resolve correctly. Without the redirect, a bare
    // /admin request falls through to the S3 catch-all `/{bucket}` route
    // and is mistaken for a bucket named "admin".
    cfg.service(web::resource("/admin").route(web::get().to(redirect_to_admin_slash)));
    cfg.service(web::resource("/admin/").route(web::get().to(serve_index)));
    cfg.service(web::resource("/admin/{path:.*}").route(web::get().to(serve_asset)));
}

async fn redirect_to_admin_slash() -> HttpResponse {
    HttpResponse::PermanentRedirect()
        .insert_header((header::LOCATION, "/admin/"))
        .finish()
}

async fn serve_index() -> HttpResponse {
    serve("index.html")
}

async fn serve_asset(req: HttpRequest) -> HttpResponse {
    let raw = req.match_info().query("path");
    if raw.is_empty() {
        return serve("index.html");
    }
    if Assets::get(raw).is_some() {
        return serve(raw);
    }
    // SPA fallback: let the client-side router handle unknown paths.
    serve("index.html")
}

fn serve(path: &str) -> HttpResponse {
    let Some(content) = Assets::get(path) else {
        return HttpResponse::NotFound().body("admin UI not built");
    };
    let mime = mime_guess::from_path(path).first_or_octet_stream();
    let cache_control = if path == "index.html" {
        "no-cache"
    } else if path.starts_with("assets/") {
        "public, max-age=31536000, immutable"
    } else {
        "public, max-age=3600"
    };
    HttpResponse::Ok()
        .insert_header((header::CONTENT_TYPE, mime.essence_str()))
        .insert_header((header::CACHE_CONTROL, cache_control))
        .body(content.data.to_vec())
}
