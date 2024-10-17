use actix_web::{
    web::{self},
    HttpResponse, Responder,
};
use mime_guess::from_path;
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "rockbox/dist/"]
struct Asset;

fn handle_embedded_file(path: &str) -> HttpResponse {
    match Asset::get(path) {
        Some(content) => HttpResponse::Ok()
            .content_type(from_path(path).first_or_octet_stream().as_ref())
            .body(content.data.into_owned()),
        None => HttpResponse::NotFound().body("404 Not Found"),
    }
}

#[actix_web::get("/")]
pub async fn index() -> impl Responder {
    handle_embedded_file("index.html")
}

pub async fn index_spa() -> impl Responder {
    handle_embedded_file("index.html")
}

#[actix_web::get("/{_:.*}")]
pub async fn dist(path: web::Path<String>) -> impl Responder {
    handle_embedded_file(path.as_str())
}
