use anyhow::Error;
use serde::Deserialize;
use sqlx::{Pool, Sqlite};

use crate::repo;

#[derive(Deserialize)]
struct WikiSummary {
    extract: Option<String>,
}

/// For every genre whose `description` column is NULL, fetch a short
/// description from the Wikipedia REST summary API and store it. Runs in the
/// background at startup; idempotent (skips genres that already have a
/// description). Uses the same rustls-webpki-roots TLS stack as the artist
/// enrichment so it works on Android without the system cert store.
pub async fn update_metadata(pool: Pool<Sqlite>) -> Result<(), Error> {
    let genres = repo::genre::all(pool.clone())
        .await?
        .into_iter()
        .filter(|g| g.description.is_none())
        .collect::<Vec<_>>();

    if genres.is_empty() {
        return Ok(());
    }

    let client = reqwest::Client::builder()
        .user_agent("Rockbox/1.0 (https://github.com/tsirysndr/rockbox-zig)")
        .build()?;

    for genre in genres {
        // Wikipedia page titles use underscores for spaces.
        let title = genre.name.replace(' ', "_");

        // Try "{Genre}" first, then "{Genre}_(music)" as a fallback for
        // disambiguation pages (e.g. "Pop" → "Pop_(music)").
        let description = fetch_wiki_extract(&client, &title).await.or_else(|_| {
            // Return a future-compatible fallback via async block below.
            Ok::<Option<String>, Error>(None)
        })?;

        let description = match description {
            Some(d) => d,
            None => {
                let music_title = format!("{}_(music)", title);
                match fetch_wiki_extract(&client, &music_title).await {
                    Ok(Some(d)) => d,
                    _ => {
                        tracing::debug!("genres: no Wikipedia entry for \"{}\"", genre.name);
                        continue;
                    }
                }
            }
        };

        tracing::info!("genres: updating description for \"{}\"", genre.name);
        repo::genre::update_description(&pool, &genre.id, &description).await?;
    }

    Ok(())
}

async fn fetch_wiki_extract(
    client: &reqwest::Client,
    title: &str,
) -> Result<Option<String>, Error> {
    let url = format!(
        "https://en.wikipedia.org/api/rest_v1/page/summary/{}",
        title
    );
    let resp = client.get(&url).send().await?;
    if !resp.status().is_success() {
        return Ok(None);
    }
    let summary: WikiSummary = resp.json().await?;
    Ok(summary.extract.filter(|s| !s.is_empty()))
}
