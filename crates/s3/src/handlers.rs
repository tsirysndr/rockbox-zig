use actix_web::http::header::{
    CONTENT_LENGTH, CONTENT_TYPE, ETAG, IF_MATCH, IF_NONE_MATCH, LAST_MODIFIED,
};
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};
use std::time::SystemTime;

use crate::sigv4::{self, AuthInput, SigV4Error};
use crate::xml;
use crate::{AppState, AUDIO_EXTENSIONS, BUCKET, REGION};

const REQUEST_ID: &str = "rockbox-s3";

fn err(status: StatusCode, code: &str, message: &str, resource: &str) -> HttpResponse {
    HttpResponse::build(status)
        .content_type("application/xml")
        .body(xml::error(code, message, resource, REQUEST_ID))
}

fn unauthorised(resource: &str, detail: &str) -> HttpResponse {
    tracing::warn!("s3: rejected request: {}", detail);
    err(
        StatusCode::FORBIDDEN,
        "SignatureDoesNotMatch",
        detail,
        resource,
    )
}

fn check_bucket(bucket: &str, resource: &str) -> Result<(), HttpResponse> {
    if bucket != BUCKET {
        return Err(err(
            StatusCode::NOT_FOUND,
            "NoSuchBucket",
            "The specified bucket does not exist",
            resource,
        ));
    }
    Ok(())
}

fn verify(
    state: &AppState,
    req: &HttpRequest,
    body_sha256_hex: &str,
    resource: &str,
) -> Result<(), HttpResponse> {
    let query = req.query_string();
    let path = req.path();
    let input = AuthInput {
        method: req.method().as_str(),
        path,
        query,
        headers: req.headers(),
        body_sha256_hex,
        access_key: &state.access_key,
        secret_key: &state.secret_key,
        region: REGION,
    };
    match sigv4::verify(&input) {
        Ok(()) => Ok(()),
        Err(SigV4Error::Missing(_)) => Err(err(
            StatusCode::FORBIDDEN,
            "AccessDenied",
            "Authorization header missing",
            resource,
        )),
        Err(SigV4Error::AccessKeyMismatch) => Err(err(
            StatusCode::FORBIDDEN,
            "InvalidAccessKeyId",
            "The AWS access key id provided does not exist",
            resource,
        )),
        Err(SigV4Error::SkewTooLarge) => Err(err(
            StatusCode::FORBIDDEN,
            "RequestTimeTooSkewed",
            "Request timestamp is outside the acceptable window",
            resource,
        )),
        Err(SigV4Error::UnsupportedPayload) => Err(err(
            StatusCode::NOT_IMPLEMENTED,
            "NotImplemented",
            "STREAMING-AWS4-HMAC-SHA256-PAYLOAD is not supported; use UNSIGNED-PAYLOAD or sign the full body",
            resource,
        )),
        Err(e) => Err(unauthorised(resource, &e.to_string())),
    }
}

/// Map a bucket-relative key to an absolute path under `music_dir`. Rejects
/// any key that escapes the music directory (`..`, absolute paths, `//`,
/// embedded NUL).
fn resolve_key(state: &AppState, key: &str) -> Option<PathBuf> {
    if key.is_empty() || key.contains('\0') {
        return None;
    }
    let candidate = Path::new(key);
    if candidate.is_absolute() {
        return None;
    }
    let mut resolved = state.music_dir.clone();
    for component in candidate.components() {
        match component {
            Component::Normal(c) => resolved.push(c),
            // ParentDir / RootDir / Prefix / CurDir — none acceptable.
            _ => return None,
        }
    }
    Some(resolved)
}

fn is_audio_key(key: &str) -> bool {
    Path::new(key)
        .extension()
        .and_then(|e| e.to_str())
        .map(|ext| {
            let lower = ext.to_ascii_lowercase();
            AUDIO_EXTENSIONS.iter().any(|x| *x == lower.as_str())
        })
        .unwrap_or(false)
}

fn http_date(time: SystemTime) -> String {
    let dt: DateTime<Utc> = time.into();
    dt.format("%a, %d %b %Y %H:%M:%S GMT").to_string()
}

fn iso8601(time: SystemTime) -> String {
    let dt: DateTime<Utc> = time.into();
    dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string()
}

fn etag_hex(bytes: &[u8]) -> String {
    // S3 returns MD5 for single-PUT objects; we return SHA-256 hex prefix —
    // clients use it for equality checks, not crypto.
    let h = Sha256::digest(bytes);
    format!("\"{}\"", hex::encode(&h[..16]))
}

// ── ListBuckets — GET / ──────────────────────────────────────────────────────

pub async fn list_buckets(state: web::Data<AppState>, req: HttpRequest) -> HttpResponse {
    let resource = "/".to_string();
    if let Err(r) = verify(&state, &req, &empty_body_hash(), &resource) {
        return r;
    }
    let created = iso8601(SystemTime::now());
    let body = format!(
        "{decl}<ListAllMyBucketsResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\
         <Owner><ID>rockboxd</ID><DisplayName>rockboxd</DisplayName></Owner>\
         <Buckets><Bucket><Name>{bucket}</Name><CreationDate>{created}</CreationDate></Bucket></Buckets>\
         </ListAllMyBucketsResult>",
        decl = xml::XML_DECL,
        bucket = xml::esc(BUCKET),
        created = created,
    );
    HttpResponse::Ok()
        .content_type("application/xml")
        .body(body)
}

// ── ListObjectsV2 — GET /{bucket}?list-type=2 ────────────────────────────────

pub async fn list_objects(
    state: web::Data<AppState>,
    req: HttpRequest,
    bucket: web::Path<String>,
) -> HttpResponse {
    let bucket = bucket.into_inner();
    let resource = format!("/{}", bucket);
    if let Err(r) = check_bucket(&bucket, &resource) {
        return r;
    }
    if let Err(r) = verify(&state, &req, &empty_body_hash(), &resource) {
        return r;
    }

    let q = parse_query(req.query_string());
    let prefix = q.get("prefix").cloned().unwrap_or_default();
    let delimiter = q.get("delimiter").cloned();
    let max_keys: usize = q
        .get("max-keys")
        .and_then(|s| s.parse().ok())
        .unwrap_or(1000)
        .min(1000);

    let mut keys: Vec<String> = Vec::new();
    if let Err(e) = collect_keys(&state.music_dir, &state.music_dir, &mut keys) {
        tracing::warn!("s3: list walk failed: {}", e);
    }
    keys.sort();

    let mut common_prefixes: Vec<String> = Vec::new();
    let mut contents: Vec<String> = Vec::new();
    let mut emitted = 0usize;
    let mut truncated = false;

    for key in &keys {
        if !key.starts_with(&prefix) {
            continue;
        }
        if let Some(d) = &delimiter {
            let rest = &key[prefix.len()..];
            if let Some(idx) = rest.find(d.as_str()) {
                let cp = format!("{}{}{}", prefix, &rest[..idx], d);
                if !common_prefixes.contains(&cp) {
                    if emitted >= max_keys {
                        truncated = true;
                        break;
                    }
                    common_prefixes.push(cp);
                    emitted += 1;
                }
                continue;
            }
        }
        if emitted >= max_keys {
            truncated = true;
            break;
        }
        let path = state.music_dir.join(key);
        let (size, mtime) = match fs::metadata(&path) {
            Ok(m) => (m.len(), m.modified().unwrap_or_else(|_| SystemTime::now())),
            Err(_) => continue,
        };
        contents.push(format!(
            "<Contents><Key>{k}</Key><LastModified>{lm}</LastModified><Size>{sz}</Size><StorageClass>STANDARD</StorageClass></Contents>",
            k = xml::esc(key),
            lm = iso8601(mtime),
            sz = size,
        ));
        emitted += 1;
    }

    let mut body = format!(
        "{decl}<ListBucketResult xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">\
         <Name>{bucket}</Name>\
         <Prefix>{prefix}</Prefix>\
         <KeyCount>{count}</KeyCount>\
         <MaxKeys>{maxk}</MaxKeys>\
         <IsTruncated>{trunc}</IsTruncated>",
        decl = xml::XML_DECL,
        bucket = xml::esc(&bucket),
        prefix = xml::esc(&prefix),
        count = contents.len() + common_prefixes.len(),
        maxk = max_keys,
        trunc = truncated,
    );
    if let Some(d) = &delimiter {
        body.push_str(&format!("<Delimiter>{}</Delimiter>", xml::esc(d)));
    }
    for c in contents {
        body.push_str(&c);
    }
    for cp in common_prefixes {
        body.push_str(&format!(
            "<CommonPrefixes><Prefix>{}</Prefix></CommonPrefixes>",
            xml::esc(&cp)
        ));
    }
    body.push_str("</ListBucketResult>");

    HttpResponse::Ok()
        .content_type("application/xml")
        .body(body)
}

fn collect_keys(root: &Path, dir: &Path, out: &mut Vec<String>) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let ft = entry.file_type()?;
        if ft.is_dir() {
            collect_keys(root, &path, out)?;
        } else if ft.is_file() {
            if let Ok(rel) = path.strip_prefix(root) {
                let key = rel.to_string_lossy().replace('\\', "/");
                if is_audio_key(&key) {
                    out.push(key);
                }
            }
        }
    }
    Ok(())
}

// ── PUT /{bucket}/{key+} ─────────────────────────────────────────────────────

pub async fn put_object(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
    body: web::Bytes,
) -> HttpResponse {
    let (bucket, key) = path.into_inner();
    let resource = format!("/{}/{}", bucket, key);
    if let Err(r) = check_bucket(&bucket, &resource) {
        return r;
    }
    let body_sha = hex::encode(Sha256::digest(&body));
    if let Err(r) = verify(&state, &req, &body_sha, &resource) {
        return r;
    }
    if !is_audio_key(&key) {
        return err(
            StatusCode::BAD_REQUEST,
            "InvalidRequest",
            "Only audio file extensions are accepted (mp3, flac, ogg, m4a, wav, opus, …)",
            &resource,
        );
    }
    let abs = match resolve_key(&state, &key) {
        Some(p) => p,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "InvalidArgument",
                "Object key escapes music_dir",
                &resource,
            )
        }
    };
    if let Some(parent) = abs.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            tracing::error!("s3: mkdir {}: {}", parent.display(), e);
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "Failed to create parent directory",
                &resource,
            );
        }
    }
    if let Err(e) = fs::write(&abs, &body) {
        tracing::error!("s3: write {}: {}", abs.display(), e);
        return err(
            StatusCode::INTERNAL_SERVER_ERROR,
            "InternalError",
            "Failed to write object",
            &resource,
        );
    }
    tracing::info!("s3: PUT {} ({} bytes)", key, body.len());
    HttpResponse::Ok()
        .insert_header((ETAG, etag_hex(&body)))
        .finish()
}

// ── DELETE /{bucket}/{key+} ──────────────────────────────────────────────────

pub async fn delete_object(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    let (bucket, key) = path.into_inner();
    let resource = format!("/{}/{}", bucket, key);
    if let Err(r) = check_bucket(&bucket, &resource) {
        return r;
    }
    if let Err(r) = verify(&state, &req, &empty_body_hash(), &resource) {
        return r;
    }
    let abs = match resolve_key(&state, &key) {
        Some(p) => p,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "InvalidArgument",
                "Object key escapes music_dir",
                &resource,
            )
        }
    };
    match fs::remove_file(&abs) {
        Ok(()) => {
            tracing::info!("s3: DELETE {}", key);
            HttpResponse::NoContent().finish()
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => HttpResponse::NoContent().finish(),
        Err(e) => {
            tracing::error!("s3: remove {}: {}", abs.display(), e);
            err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "Failed to delete object",
                &resource,
            )
        }
    }
}

// ── GET /{bucket}/{key+} ─────────────────────────────────────────────────────

pub async fn get_object(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    object_response(state, req, path, true).await
}

pub async fn head_object(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
) -> HttpResponse {
    object_response(state, req, path, false).await
}

async fn object_response(
    state: web::Data<AppState>,
    req: HttpRequest,
    path: web::Path<(String, String)>,
    include_body: bool,
) -> HttpResponse {
    let (bucket, key) = path.into_inner();
    let resource = format!("/{}/{}", bucket, key);
    if let Err(r) = check_bucket(&bucket, &resource) {
        return r;
    }
    if let Err(r) = verify(&state, &req, &empty_body_hash(), &resource) {
        return r;
    }
    let abs = match resolve_key(&state, &key) {
        Some(p) => p,
        None => {
            return err(
                StatusCode::BAD_REQUEST,
                "InvalidArgument",
                "Object key escapes music_dir",
                &resource,
            )
        }
    };
    let meta = match fs::metadata(&abs) {
        Ok(m) if m.is_file() => m,
        _ => {
            return err(
                StatusCode::NOT_FOUND,
                "NoSuchKey",
                "The specified key does not exist",
                &resource,
            )
        }
    };
    let mtime = meta.modified().unwrap_or_else(|_| SystemTime::now());

    if !include_body {
        return HttpResponse::Ok()
            .insert_header((CONTENT_LENGTH, meta.len()))
            .insert_header((LAST_MODIFIED, http_date(mtime)))
            .insert_header((CONTENT_TYPE, content_type_for(&key)))
            .finish();
    }

    let mut bytes = Vec::with_capacity(meta.len() as usize);
    match fs::File::open(&abs).and_then(|mut f| f.read_to_end(&mut bytes)) {
        Ok(_) => {}
        Err(e) => {
            tracing::error!("s3: read {}: {}", abs.display(), e);
            return err(
                StatusCode::INTERNAL_SERVER_ERROR,
                "InternalError",
                "Failed to read object",
                &resource,
            );
        }
    }
    let etag = etag_hex(&bytes);
    // Trivial conditional GET — sufficient for sync clients.
    if let Some(ifnm) = req
        .headers()
        .get(IF_NONE_MATCH)
        .and_then(|v| v.to_str().ok())
    {
        if ifnm == etag {
            return HttpResponse::NotModified().finish();
        }
    }
    if let Some(im) = req.headers().get(IF_MATCH).and_then(|v| v.to_str().ok()) {
        if im != etag {
            return err(
                StatusCode::PRECONDITION_FAILED,
                "PreconditionFailed",
                "If-Match did not match ETag",
                &resource,
            );
        }
    }

    HttpResponse::Ok()
        .insert_header((CONTENT_LENGTH, meta.len()))
        .insert_header((LAST_MODIFIED, http_date(mtime)))
        .insert_header((CONTENT_TYPE, content_type_for(&key)))
        .insert_header((ETAG, etag))
        .body(bytes)
}

fn content_type_for(key: &str) -> &'static str {
    match Path::new(key)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_ascii_lowercase())
        .as_deref()
    {
        Some("mp3") => "audio/mpeg",
        Some("flac") => "audio/flac",
        Some("ogg") | Some("opus") => "audio/ogg",
        Some("m4a") | Some("aac") | Some("mp4") | Some("alac") => "audio/mp4",
        Some("wav") => "audio/wav",
        Some("wma") => "audio/x-ms-wma",
        Some("ape") => "audio/x-ape",
        Some("aiff") | Some("aif") => "audio/aiff",
        _ => "application/octet-stream",
    }
}

fn parse_query(q: &str) -> std::collections::HashMap<String, String> {
    let mut out = std::collections::HashMap::new();
    for part in q.split('&').filter(|s| !s.is_empty()) {
        let mut it = part.splitn(2, '=');
        let k = it.next().unwrap_or("").to_string();
        let v = it.next().unwrap_or("").to_string();
        out.insert(
            percent_encoding::percent_decode_str(&k)
                .decode_utf8_lossy()
                .into_owned(),
            percent_encoding::percent_decode_str(&v)
                .decode_utf8_lossy()
                .into_owned(),
        );
    }
    out
}

fn empty_body_hash() -> String {
    hex::encode(Sha256::digest(b""))
}
