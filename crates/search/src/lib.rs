use anyhow::Error;
use tantivy::collector::TopDocs;
use tantivy::query::QueryParser;
use tantivy::ReloadPolicy;
use tantivy::{schema::Schema, Index, TantivyDocument};

pub mod album;
pub mod artist;
pub mod file;
pub mod liked_album;
pub mod liked_track;
pub mod track;

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
    let query_parser = QueryParser::for_index(index, fields);
    let query = query_parser.parse_query(query_string)?;

    // Execute the search and collect the top 10 results
    let top_docs = searcher.search(&query, &TopDocs::with_limit(10))?;

    // Return the documents and their scores
    let results = top_docs
        .into_iter()
        .map(|(score, doc_address)| (score, searcher.doc(doc_address).unwrap()))
        .collect();

    Ok(results)
}
