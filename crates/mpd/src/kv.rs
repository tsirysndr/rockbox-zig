use std::collections::HashMap;

use anyhow::Error;
use rockbox_library::{entity, repo};
use sqlx::{Pool, Sqlite};

#[derive(Clone)]
pub struct KV<V> {
    store: HashMap<String, V>,
}

impl<V> KV<V> {
    pub fn new() -> Self {
        Self {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, key: &str) -> Option<&V> {
        self.store.get(key)
    }

    pub fn set(&mut self, key: &str, value: V) {
        self.store.insert(key.to_string(), value);
    }

    pub fn remove(&mut self, key: &str) {
        self.store.remove(key);
    }

    pub fn keys(&self) -> Vec<String> {
        self.store.keys().cloned().collect()
    }

    pub fn values(&self) -> Vec<&V> {
        self.store.values().collect()
    }

    pub fn id(&self, key: &str) -> Option<usize> {
        self.keys().iter().position(|x| x == key)
    }
}

pub async fn build_tracks_kv(pool: Pool<Sqlite>) -> Result<KV<entity::track::Track>, Error> {
    let tracks = repo::track::all(pool.clone()).await?;
    let mut kv = KV::new();

    for track in tracks {
        kv.set(&track.path, track.clone());
    }

    Ok(kv)
}
