use crate::{Indexable, Searchable};
use serde::{Deserialize, Serialize};
use tantivy::{doc, schema::*, TantivyDocument};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct File {
    pub name: String,
    pub time_write: i64,
    pub is_directory: bool,
}

impl Indexable for File {
    fn to_document(&self) -> TantivyDocument {
        let schema: Schema = self.build_schema();
        let name = schema.get_field("name").unwrap();
        let time_write = schema.get_field("time_write").unwrap();
        let is_directory = schema.get_field("is_directory").unwrap();

        let document = doc!(
            name => self.name.to_owned(),
            time_write => self.time_write,
            is_directory => self.is_directory,
        );

        document
    }

    fn build_schema(&self) -> Schema {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_i64_field("time_write", STORED);
        schema_builder.add_bool_field("is_directory", STORED);

        schema_builder.build()
    }
}

impl Searchable for File {
    fn schema(&self) -> Schema {
        self.build_schema()
    }

    fn default_fields(&self) -> Vec<String> {
        vec!["name".to_string()]
    }
}

impl From<TantivyDocument> for File {
    fn from(doc: TantivyDocument) -> Self {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        let name_field = schema_builder.add_text_field("name", TEXT | STORED);
        let time_write_field = schema_builder.add_i64_field("time_write", STORED);
        let is_directory_field = schema_builder.add_bool_field("is_directory", STORED);

        let name = doc
            .get_first(name_field)
            .unwrap()
            .as_str()
            .unwrap()
            .to_string();
        let time_write = doc.get_first(time_write_field).unwrap().as_i64().unwrap();
        let is_directory = doc
            .get_first(is_directory_field)
            .unwrap()
            .as_bool()
            .unwrap();

        Self {
            name,
            time_write,
            is_directory,
        }
    }
}
