/// Minimal XML escaping for character data — S3 responses only need the five
/// XML predefined entities. We never serialise attributes here, so quoting is
/// uniform.
pub fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            _ => out.push(c),
        }
    }
    out
}

pub const XML_DECL: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>";

pub fn error(code: &str, message: &str, resource: &str, request_id: &str) -> String {
    format!(
        "{decl}<Error><Code>{code}</Code><Message>{msg}</Message><Resource>{res}</Resource><RequestId>{rid}</RequestId></Error>",
        decl = XML_DECL,
        code = esc(code),
        msg = esc(message),
        res = esc(resource),
        rid = esc(request_id),
    )
}
