use async_graphql::{ComplexObject, SimpleObject};
use serde::Serialize;
// Define a complex object
#[derive(Debug, SimpleObject, Default)]
#[graphql(complex)]
#[derive(sqlx::FromRow, Clone)]
pub struct User {
    pub name: String,
    pub email: String,
    pub uuid:String,
    pub created_at:String
}

#[ComplexObject]
impl User {
    async fn users(&self) -> String {
        format!("{:?}", self)
    }
}

#[derive(Debug, SimpleObject, Default, Serialize)]
#[graphql(complex)]
#[derive(sqlx::FromRow, Clone)]
pub struct Task {
    pub taskid:String,
    pub title: String,
    pub description: String,
    pub userid:String
}

#[ComplexObject]
impl Task {
    async fn tasks(&self) -> String {
        format!("{:?}", self)
    }
}