use async_graphql::{ComplexObject, SimpleObject};
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