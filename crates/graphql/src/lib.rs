use async_graphql::{EmptySubscription, Schema};
use schema::{Mutation, Query};

pub mod schema;
pub mod server;
pub type RockboxSchema = Schema<Query, Mutation, EmptySubscription>;
