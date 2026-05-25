use actix_web::HttpResponse;
use serde_json::{json, Value};

const API_VERSION: &str = "1.16.1";
const SERVER_TYPE: &str = "rockbox";

pub fn ok_json(data: Value) -> HttpResponse {
    let mut body = json!({
        "status": "ok",
        "version": API_VERSION,
        "type": SERVER_TYPE,
    });
    // Merge data fields into the envelope body
    if let (Some(obj), Some(data_obj)) = (body.as_object_mut(), data.as_object()) {
        for (k, v) in data_obj {
            obj.insert(k.clone(), v.clone());
        }
    }
    HttpResponse::Ok().json(json!({ "subsonic-response": body }))
}

pub fn error_json(code: u32, message: &str) -> HttpResponse {
    let body = json!({
        "status": "failed",
        "version": API_VERSION,
        "type": SERVER_TYPE,
        "error": { "code": code, "message": message }
    });
    HttpResponse::Ok().json(json!({ "subsonic-response": body }))
}

pub fn ok_xml(inner: &str) -> HttpResponse {
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><subsonic-response xmlns="http://subsonic.org/restapi" status="ok" version="{API_VERSION}">{inner}</subsonic-response>"#
    );
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(body)
}

pub fn error_xml(code: u32, message: &str) -> HttpResponse {
    let body = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?><subsonic-response xmlns="http://subsonic.org/restapi" status="failed" version="{API_VERSION}"><error code="{code}" message="{message}"/></subsonic-response>"#
    );
    HttpResponse::Ok()
        .content_type("application/xml; charset=utf-8")
        .body(body)
}

/// Helper: pick JSON or XML based on the `f` query param.
pub fn respond(format: Option<&str>, json_data: Value, xml_inner: &str) -> HttpResponse {
    match format {
        Some("xml") => ok_xml(xml_inner),
        _ => ok_json(json_data),
    }
}

pub fn respond_error(format: Option<&str>, code: u32, message: &str) -> HttpResponse {
    match format {
        Some("xml") => error_xml(code, message),
        _ => error_json(code, message),
    }
}
