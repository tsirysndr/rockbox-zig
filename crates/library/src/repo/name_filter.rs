//! Shared SQL builders for Jellyfin's `NameStartsWith` /
//! `NameStartsWithOrGreater` / `NameLessThan` filter params and the
//! `/Items/Prefixes` alpha-jump rail. Keeps the LIKE escaping and
//! placeholder-numbering logic in one place across the artist/album/track
//! repo modules.

use sqlx::{Error, Pool, Sqlite};

/// Build the WHERE clause + bound values for an alpha-jump filter against
/// `column`. Returns `("", [])` when no param is set; otherwise the clause
/// is `WHERE …` ready to splice into a query, and the binds correspond to
/// `?1`, `?2`, … in order.
pub fn sql(
    column: &str,
    name_starts_with: Option<&str>,
    name_starts_with_or_greater: Option<&str>,
    name_less_than: Option<&str>,
) -> (String, Vec<String>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut binds: Vec<String> = Vec::new();
    if let Some(prefix) = name_starts_with.filter(|s| !s.is_empty()) {
        let escaped = prefix
            .replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_");
        binds.push(format!("{escaped}%"));
        clauses.push(format!(
            "{column} LIKE ?{} ESCAPE '\\' COLLATE NOCASE",
            binds.len()
        ));
    }
    if let Some(b) = name_starts_with_or_greater.filter(|s| !s.is_empty()) {
        let first = b.chars().next().unwrap().to_string();
        binds.push(first);
        clauses.push(format!(
            "UPPER(SUBSTR({column}, 1, 1)) >= UPPER(?{})",
            binds.len()
        ));
    }
    if let Some(b) = name_less_than.filter(|s| !s.is_empty()) {
        let first = b.chars().next().unwrap().to_string();
        binds.push(first);
        clauses.push(format!(
            "UPPER(SUBSTR({column}, 1, 1)) < UPPER(?{})",
            binds.len()
        ));
    }
    let where_sql = if clauses.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", clauses.join(" AND "))
    };
    (where_sql, binds)
}

/// Distinct uppercase first letters of `column` in `table`, with non-alpha
/// rows grouped under "#". The optional `extra_where` is appended to the
/// WHERE clause so callers can scope to e.g. `is_remote = 0` or "has tracks".
pub async fn prefixes(
    pool: &Pool<Sqlite>,
    table: &str,
    column: &str,
    extra_where: Option<&str>,
) -> Result<Vec<String>, Error> {
    let where_clause = match extra_where {
        Some(extra) => format!("WHERE {column} != '' AND {extra}"),
        None => format!("WHERE {column} != ''"),
    };
    let sql = format!("SELECT DISTINCT UPPER(SUBSTR({column}, 1, 1)) FROM {table} {where_clause}");
    let rows: Vec<(String,)> = sqlx::query_as(&sql).fetch_all(pool).await?;
    let mut letters: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut has_other = false;
    for (c,) in rows {
        match c.chars().next() {
            Some(ch) if ch.is_ascii_alphabetic() => {
                letters.insert(c);
            }
            _ => {
                has_other = true;
            }
        }
    }
    let mut out: Vec<String> = letters.into_iter().collect();
    if has_other {
        out.insert(0, "#".to_string());
    }
    Ok(out)
}
