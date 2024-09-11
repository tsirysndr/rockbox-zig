use std::sync::{mpsc::Sender, Arc, Mutex};

use actix_cors::Cors;
use actix_web::{
    http::header::HOST,
    web::{self, Data},
    App, HttpRequest, HttpResponse, HttpServer, Result,
};
use async_graphql::{http::GraphiQLSource, EmptySubscription, Schema};
use async_graphql_actix_web::{GraphQLRequest, GraphQLResponse};
use owo_colors::OwoColorize;
use rockbox_sys::events::RockboxCommand;

use crate::{
    schema::{Mutation, Query},
    RockboxSchema,
};

#[actix_web::post("/graphql")]
async fn index_graphql(schema: web::Data<RockboxSchema>, req: GraphQLRequest) -> GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}

#[actix_web::get("/graphiql")]
async fn index_graphiql(req: HttpRequest) -> Result<HttpResponse> {
    let host = req
        .headers()
        .get(HOST)
        .unwrap()
        .to_str()
        .unwrap()
        .split(":")
        .next()
        .unwrap();

    let http_port = std::env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
    let graphql_endpoint = format!("http://{}:{}/graphql", host, http_port);
    let ws_endpoint = format!("ws://{}:{}/graphql", host, http_port);
    Ok(HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            GraphiQLSource::build()
                .endpoint(&graphql_endpoint)
                .subscription_endpoint(&ws_endpoint)
                .finish(),
        ))
}

pub async fn start(cmd_tx: Arc<Mutex<Sender<RockboxCommand>>>) -> std::io::Result<()> {
    let schema = Schema::build(
        Query::default(),
        Mutation::default(),
        EmptySubscription::default(),
    )
    .data(cmd_tx)
    .finish();
    let graphql_port = std::env::var("ROCKBOX_GRAPHQL_PORT").unwrap_or("6062".to_string());
    let addr = format!("{}:{}", "0.0.0.0", graphql_port);

    println!(
        "{} server is running on {}",
        "Rockbox GraphQL".bright_purple(),
        addr.bright_green()
    );

    HttpServer::new(move || {
        let cors = Cors::permissive();
        App::new()
            .app_data(Data::new(schema.clone()))
            .wrap(cors)
            .service(index_graphql)
            .service(index_graphiql)
    })
    .bind(addr)?
    .run()
    .await
}
