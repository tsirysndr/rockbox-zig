use actix_web::HttpResponse;

type HandlerResult = actix_web::Result<HttpResponse>;

pub async fn get_openapi() -> HandlerResult {
    let spec = include_str!("../../openapi.json");
    Ok(HttpResponse::Ok()
        .content_type("application/json")
        .body(spec))
}

pub async fn index() -> HandlerResult {
    let html = include_str!("../../docs/index.html");
    Ok(HttpResponse::Ok().content_type("text/html").body(html))
}
