use std::time::Duration;
use async_graphql::{http::GraphiQLSource, *};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router
};
use sqlx::{postgres::{PgPoolOptions}};
use crate::mutation::Mutation;
use crate::query::Query;

mod mutation;
mod query;
mod model;
mod guard;

async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
// Main function
#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    
    //Database connection
    let db_pool = PgPoolOptions::new()
    .max_connections(5)
    .acquire_timeout(Duration::from_secs(5))
    .connect("postgres://axum_postgres:axum_postgres@localhost:5432/axum_postgres")
    .await
    .expect("There was some error with connection string");

    // Build the GraphQL Schema
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(db_pool.clone())
        .finish();
    // Build the router
    let router = Router::new()
        .route("/graphql", get(graphql).post_service(GraphQL::new(schema)))
        .with_state(db_pool);
    println!("Shuttle is running");
    // Use the router
    Ok(router.into())
}
