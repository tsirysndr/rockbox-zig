use async_graphql::{EmptySubscription, Schema};
use schema::{Mutation, Query};

pub mod schema;
pub mod server;
pub type RockboxSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn rockbox_url() -> String {
    let port = std::env::var("ROCKBOX_TCP_PORT").unwrap_or_else(|_| "6063".to_string());
    format!("http://127.0.0.1:{}", port)
}
