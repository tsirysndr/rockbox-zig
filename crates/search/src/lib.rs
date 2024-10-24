use std::env;

use album::Album;
use anyhow::Error;
use artist::Artist;
use file::File;
use liked_album::LikedAlbum;
use liked_track::LikedTrack;
use tantivy::collector::TopDocs;
use tantivy::directory::MmapDirectory;
use tantivy::query::{FuzzyTermQuery, QueryParser};
use tantivy::{schema::Schema, Index, TantivyDocument};
use tantivy::{ReloadPolicy, Term};
use track::Track;

pub mod album;
pub mod artist;
pub mod file;
pub mod liked_album;
pub mod liked_track;
pub mod track;

#[derive(Clone)]
pub struct Indexes {
    pub albums: Index,
    pub artists: Index,
    pub tracks: Index,
    pub liked_albums: Index,
    pub liked_tracks: Index,
    pub files: Index,
}

pub fn create_indexes() -> Result<Indexes, Error> {
    let home = env::var("HOME")?;
    let rockbox_dir = format!("{}/.config/rockbox.org", home);
    let index_dir = format!("{}/indexes", rockbox_dir);

    let albums = create_index(Album::default().schema(), &format!("{}/albums", index_dir))?;
    let artists = create_index(
        Artist::default().schema(),
        &format!("{}/artists", index_dir),
    )?;
    let tracks = create_index(Track::default().schema(), &format!("{}/tracks", index_dir))?;
    let liked_albums = create_index(
        LikedAlbum::default().schema(),
        &format!("{}/liked_albums", index_dir),
    )?;
    let liked_tracks = create_index(
        LikedTrack::default().schema(),
        &format!("{}/liked_tracks", index_dir),
    )?;
    let files = create_index(File::default().schema(), &format!("{}/files", index_dir))?;

    Ok(Indexes {
        albums,
        artists,
        tracks,
        liked_albums,
        liked_tracks,
        files,
    })
}

fn create_index(schema: Schema, index_path: &str) -> Result<Index, Error> {
    std::fs::create_dir_all(index_path)?;
    let dir = MmapDirectory::open(index_path)?;
    let index: Index = Index::open_or_create(dir, schema.clone())?;
    Ok(index)
}

pub fn delete_all_documents(index: &Index) -> Result<(), Error> {
    let mut index_writer = index.writer::<TantivyDocument>(50_000_000)?;
    index_writer.delete_all_documents()?;
    index_writer.commit()?;
    Ok(())
}

pub trait Indexable {
    fn to_document(&self) -> TantivyDocument;
    fn build_schema(&self) -> Schema;
}

pub trait Searchable {
    fn schema(&self) -> Schema;
    fn default_fields(&self) -> Vec<String>; // Default fields to search in
}

pub fn index_entity<T: Indexable>(index: &Index, entity: &T) -> Result<(), Error> {
    let mut index_writer = index.writer(50_000_000)?;
    let doc = entity.to_document();
    index_writer.add_document(doc)?;
    index_writer.commit()?;
    Ok(())
}

pub fn search_entities<T: Searchable>(
    index: &Index,
    query_string: &str,
    entity: &T, // The entity type to search
) -> tantivy::Result<Vec<(f32, TantivyDocument)>> {
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::OnCommitWithDelay)
        .try_into()?;
    let searcher = reader.searcher();

    // Get the schema and fields for the entity
    let schema = entity.schema();
    let default_fields: Vec<String> = entity.default_fields();

    // Convert field names to Field objects
    let fields: Vec<tantivy::schema::Field> = default_fields
        .iter()
        .filter_map(|field_name| Some(schema.get_field(field_name).unwrap()))
        .collect();

    // Parse the query
    let query_parser = QueryParser::for_index(index, fields.clone());
    let query = query_parser.parse_query(query_string)?;

    // Execute the search and collect the top 10 results
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    // Return the documents and their scores
    let mut results: Vec<(f32, TantivyDocument)> = top_docs
        .into_iter()
        .map(|(score, doc_address)| (score, searcher.doc(doc_address).unwrap()))
        .collect();

    if results.is_empty() {
        // loop through the fields and search for fuzzy matches
        for field in fields {
            let term = Term::from_field_text(field, query_string);
            let query = FuzzyTermQuery::new(term, 1, true);
            let fuzzy_results: Vec<(f32, TantivyDocument)> = searcher
                .search(&query, &TopDocs::with_limit(10))?
                .into_iter()
                .map(|(score, doc_address)| (score, searcher.doc(doc_address).unwrap()))
                .collect();
            results.extend(fuzzy_results);
        }
    }

    Ok(results)
}
