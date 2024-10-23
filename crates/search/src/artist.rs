use crate::{Indexable, Searchable};
use serde::{Deserialize, Serialize};
use tantivy::{doc, schema::*, TantivyDocument};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Artist {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

impl Indexable for Artist {
    fn to_document(&self) -> TantivyDocument {
        let schema: Schema = self.build_schema();

        let id = schema.get_field("id").unwrap();
        let name = schema.get_field("name").unwrap();
        let bio = schema.get_field("bio").unwrap();
        let image = schema.get_field("image").unwrap();

        let mut document = doc!(
            id => self.id.to_owned(),
            name => self.name.to_owned(),
        );

        if let Some(value) = &self.bio {
            document.add_text(bio, value);
        }

        if let Some(value) = &self.image {
            document.add_text(image, value);
        }

        document
    }

    fn build_schema(&self) -> Schema {
        let mut schema_builder: SchemaBuilder = Schema::builder();

        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("name", TEXT | STORED);
        schema_builder.add_text_field("bio", TEXT | STORED);
        schema_builder.add_text_field("image", STRING | STORED);

        schema_builder.build()
    }
}

impl Searchable for Artist {
    fn schema(&self) -> Schema {
        self.build_schema()
    }

    fn default_fields(&self) -> Vec<String> {
        vec!["name".to_string(), "bio".to_string()]
    }
}
