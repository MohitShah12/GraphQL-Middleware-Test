use std::{time::Duration};
use async_graphql::{*};
use async_graphql::{http::GraphiQLSource};
use async_graphql_axum::{GraphQLRequest, GraphQLResponse};

use axum::routing::post;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router, Extension
};
use sqlx::{postgres::{PgPoolOptions}};
use crate::{mutation::Mutation};
use crate::query::Query;

pub(crate) type ServiceSchema = Schema<Query, Mutation, EmptySubscription>;


mod mutation;
mod query;
mod model;
mod guard;

use crate::guard::{AuthClaims};

use crate::mutation::login_user_new;

async fn graphql() -> impl IntoResponse {
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}
async fn graphql_handler(
    auth:AuthClaims,
    Extension(schema): Extension<ServiceSchema>, 
    req:GraphQLRequest,
) -> GraphQLResponse  {
    let request = req.into_inner().data(auth.clone());
    schema.execute(request).await.into()
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
    let schema = Schema::build(Query, Mutation, EmptySubscription).data(db_pool.clone()).finish();

    // Build the router
    let router = Router::new()
        .route("/graphql", get(graphql).post(graphql_handler))
        .layer(Extension(schema))
        .route("/login", post(login_user_new))
        .with_state(db_pool);
    println!("Shuttle is running");

    // Use the router
    Ok(router.into())
}
