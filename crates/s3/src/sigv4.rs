use actix_web::http::header::HeaderMap;
use hmac::{Hmac, Mac};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

type HmacSha256 = Hmac<Sha256>;

/// AWS canonical-URI encoding: same as RFC 3986 unreserved set
/// (A-Z a-z 0-9 - _ . ~), plus '/'. Everything else is %-encoded.
const URI_PATH: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~')
    .remove(b'/');

/// AWS canonical-query encoding: RFC 3986 unreserved set; '/' IS encoded here.
const URI_QUERY: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'_')
    .remove(b'.')
    .remove(b'~');

#[derive(Debug)]
pub enum SigV4Error {
    Missing(&'static str),
    Malformed(&'static str),
    AccessKeyMismatch,
    ScopeMismatch,
    SignatureMismatch,
    SkewTooLarge,
    UnsupportedPayload,
}

impl std::fmt::Display for SigV4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing(h) => write!(f, "missing header: {}", h),
            Self::Malformed(s) => write!(f, "malformed: {}", s),
            Self::AccessKeyMismatch => write!(f, "access key id does not match"),
            Self::ScopeMismatch => write!(f, "credential scope does not match"),
            Self::SignatureMismatch => write!(f, "signature mismatch"),
            Self::SkewTooLarge => write!(f, "request timestamp skew too large"),
            Self::UnsupportedPayload => {
                write!(f, "STREAMING-AWS4-HMAC-SHA256-PAYLOAD not supported")
            }
        }
    }
}

pub struct AuthInput<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub query: &'a str,
    pub headers: &'a HeaderMap,
    pub body_sha256_hex: &'a str,
    pub access_key: &'a str,
    pub secret_key: &'a str,
    pub region: &'a str,
}

/// Verify an `Authorization: AWS4-HMAC-SHA256 …` header against the signing
/// secret. Returns Ok(()) on success.
pub fn verify(input: &AuthInput<'_>) -> Result<(), SigV4Error> {
    let auth =
        header_str(input.headers, "authorization").ok_or(SigV4Error::Missing("Authorization"))?;
    let amz_date =
        header_str(input.headers, "x-amz-date").ok_or(SigV4Error::Missing("x-amz-date"))?;
    let content_sha = header_str(input.headers, "x-amz-content-sha256").unwrap_or_default();

    if content_sha.starts_with("STREAMING-AWS4-HMAC-SHA256-PAYLOAD") {
        return Err(SigV4Error::UnsupportedPayload);
    }

    let parsed = parse_authorization(&auth)?;

    if parsed.credential.access_key != input.access_key {
        return Err(SigV4Error::AccessKeyMismatch);
    }
    if parsed.credential.region != input.region
        || parsed.credential.service != "s3"
        || parsed.credential.terminator != "aws4_request"
    {
        return Err(SigV4Error::ScopeMismatch);
    }

    // x-amz-date format: YYYYMMDDTHHMMSSZ
    if !amz_date.starts_with(&parsed.credential.date) {
        return Err(SigV4Error::ScopeMismatch);
    }
    if let Ok(ts) = chrono::NaiveDateTime::parse_from_str(&amz_date, "%Y%m%dT%H%M%SZ") {
        let now = chrono::Utc::now().naive_utc();
        let skew = (now - ts).num_seconds().abs();
        if skew > 900 {
            return Err(SigV4Error::SkewTooLarge);
        }
    }

    // Payload hash for the canonical request — clients use UNSIGNED-PAYLOAD
    // or the actual hex digest of the body.
    let payload_hash = if content_sha.is_empty() || content_sha == "UNSIGNED-PAYLOAD" {
        "UNSIGNED-PAYLOAD".to_string()
    } else if content_sha == input.body_sha256_hex {
        content_sha.to_string()
    } else {
        // Client lied about the payload hash.
        return Err(SigV4Error::SignatureMismatch);
    };

    let canonical_uri = utf8_percent_encode(input.path, URI_PATH).to_string();
    let canonical_query = canonical_query(input.query);
    let (canonical_headers, signed_headers_actual) =
        canonical_headers(input.headers, &parsed.signed_headers);

    if signed_headers_actual != parsed.signed_headers.join(";") {
        return Err(SigV4Error::Malformed("signed headers"));
    }

    let canonical_request = format!(
        "{}\n{}\n{}\n{}\n\n{}\n{}",
        input.method.to_ascii_uppercase(),
        canonical_uri,
        canonical_query,
        canonical_headers,
        signed_headers_actual,
        payload_hash,
    );

    let cred_scope = format!(
        "{}/{}/{}/{}",
        parsed.credential.date,
        parsed.credential.region,
        parsed.credential.service,
        parsed.credential.terminator,
    );
    let string_to_sign = format!(
        "AWS4-HMAC-SHA256\n{}\n{}\n{}",
        amz_date,
        cred_scope,
        hex::encode(Sha256::digest(canonical_request.as_bytes())),
    );

    let signing_key = derive_signing_key(
        input.secret_key,
        &parsed.credential.date,
        &parsed.credential.region,
        &parsed.credential.service,
    );
    let signature = hex::encode(hmac_sha256(&signing_key, string_to_sign.as_bytes()));

    if constant_time_eq(signature.as_bytes(), parsed.signature.as_bytes()) {
        Ok(())
    } else {
        Err(SigV4Error::SignatureMismatch)
    }
}

struct ParsedAuth {
    credential: Credential,
    signed_headers: Vec<String>,
    signature: String,
}

struct Credential {
    access_key: String,
    date: String,
    region: String,
    service: String,
    terminator: String,
}

fn parse_authorization(auth: &str) -> Result<ParsedAuth, SigV4Error> {
    let after = auth
        .strip_prefix("AWS4-HMAC-SHA256")
        .ok_or(SigV4Error::Malformed("scheme"))?
        .trim_start();

    let mut credential = None;
    let mut signed_headers = None;
    let mut signature = None;

    for part in after.split(',') {
        let part = part.trim();
        if let Some(v) = part.strip_prefix("Credential=") {
            let mut s = v.splitn(5, '/');
            credential = Some(Credential {
                access_key: s
                    .next()
                    .ok_or(SigV4Error::Malformed("credential"))?
                    .to_string(),
                date: s
                    .next()
                    .ok_or(SigV4Error::Malformed("credential"))?
                    .to_string(),
                region: s
                    .next()
                    .ok_or(SigV4Error::Malformed("credential"))?
                    .to_string(),
                service: s
                    .next()
                    .ok_or(SigV4Error::Malformed("credential"))?
                    .to_string(),
                terminator: s
                    .next()
                    .ok_or(SigV4Error::Malformed("credential"))?
                    .to_string(),
            });
        } else if let Some(v) = part.strip_prefix("SignedHeaders=") {
            signed_headers = Some(
                v.split(';')
                    .map(|h| h.trim().to_ascii_lowercase())
                    .collect::<Vec<_>>(),
            );
        } else if let Some(v) = part.strip_prefix("Signature=") {
            signature = Some(v.trim().to_string());
        }
    }

    Ok(ParsedAuth {
        credential: credential.ok_or(SigV4Error::Malformed("Credential"))?,
        signed_headers: signed_headers.ok_or(SigV4Error::Malformed("SignedHeaders"))?,
        signature: signature.ok_or(SigV4Error::Malformed("Signature"))?,
    })
}

fn canonical_query(query: &str) -> String {
    if query.is_empty() {
        return String::new();
    }
    let mut pairs: Vec<(String, String)> = query
        .split('&')
        .filter(|p| !p.is_empty())
        .map(|p| {
            let mut it = p.splitn(2, '=');
            let k = it.next().unwrap_or("");
            let v = it.next().unwrap_or("");
            (decode(k), decode(v))
        })
        .collect();
    pairs.sort();
    pairs
        .into_iter()
        .map(|(k, v)| {
            format!(
                "{}={}",
                utf8_percent_encode(&k, URI_QUERY),
                utf8_percent_encode(&v, URI_QUERY),
            )
        })
        .collect::<Vec<_>>()
        .join("&")
}

fn decode(s: &str) -> String {
    percent_encoding::percent_decode_str(s)
        .decode_utf8_lossy()
        .into_owned()
}

fn canonical_headers(headers: &HeaderMap, signed_headers: &[String]) -> (String, String) {
    let mut map: BTreeMap<String, String> = BTreeMap::new();
    for name in signed_headers {
        if let Some(v) = header_str(headers, name) {
            // Trim outer whitespace and collapse internal runs.
            let cleaned = collapse_ws(v.trim());
            map.insert(name.clone(), cleaned);
        }
    }
    let canonical = map
        .iter()
        .map(|(k, v)| format!("{}:{}\n", k, v))
        .collect::<String>();
    let names = map.keys().cloned().collect::<Vec<_>>().join(";");
    // Strip trailing \n — canonical request format wants headers terminated by
    // a single \n then signed_headers on the next line; we re-add a \n at the
    // join point in verify().
    let canonical = canonical.trim_end_matches('\n').to_string();
    (canonical, names)
}

fn collapse_ws(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut in_ws = false;
    for c in s.chars() {
        if c == ' ' || c == '\t' {
            if !in_ws {
                out.push(' ');
                in_ws = true;
            }
        } else {
            out.push(c);
            in_ws = false;
        }
    }
    out
}

fn header_str<'a>(headers: &'a HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(|s| s.to_string())
}

fn derive_signing_key(secret: &str, date: &str, region: &str, service: &str) -> Vec<u8> {
    let k_secret = format!("AWS4{}", secret);
    let k_date = hmac_sha256(k_secret.as_bytes(), date.as_bytes());
    let k_region = hmac_sha256(&k_date, region.as_bytes());
    let k_service = hmac_sha256(&k_region, service.as_bytes());
    hmac_sha256(&k_service, b"aws4_request")
}

fn hmac_sha256(key: &[u8], data: &[u8]) -> Vec<u8> {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC accepts any key length");
    mac.update(data);
    mac.finalize().into_bytes().to_vec()
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Determinism check — same inputs always derive the same signing key,
    /// and varying any one of {secret, date, region, service} changes it.
    /// Cross-implementation correctness is covered by integration testing
    /// against real S3 clients (awscli, mc, rclone).
    #[test]
    fn signing_key_is_deterministic_and_varies() {
        let a = derive_signing_key("secret", "20240101", "us-east-1", "s3");
        let b = derive_signing_key("secret", "20240101", "us-east-1", "s3");
        assert_eq!(a, b, "same inputs must produce same key");

        assert_ne!(
            a,
            derive_signing_key("other", "20240101", "us-east-1", "s3"),
        );
        assert_ne!(
            a,
            derive_signing_key("secret", "20240102", "us-east-1", "s3"),
        );
        assert_ne!(
            a,
            derive_signing_key("secret", "20240101", "us-west-2", "s3"),
        );
        assert_ne!(
            a,
            derive_signing_key("secret", "20240101", "us-east-1", "iam"),
        );
        assert_eq!(a.len(), 32, "HMAC-SHA256 output is 32 bytes");
    }

    #[test]
    fn collapses_whitespace() {
        assert_eq!(collapse_ws("a   b\tc"), "a b c");
    }

    #[test]
    fn canonical_query_sorts_and_encodes() {
        assert_eq!(canonical_query(""), "");
        assert_eq!(
            canonical_query("b=2&a=1"),
            "a=1&b=2",
            "query params must be sorted by name"
        );
        assert_eq!(
            canonical_query("prefix=foo/bar&list-type=2"),
            "list-type=2&prefix=foo%2Fbar",
            "values must be %-encoded (slash too)"
        );
    }
}
