use async_graphql::{*};
use axum::body::Body;
use axum::http::Request;
use sqlx:: PgPool;
use uuid::Uuid;
use crate::model::User;
use crate::guard::{Claims, AuthClaims};

pub struct Query;

// Implement methods for the query object
#[Object]
impl Query {
    // Asynchronously return a string
    async fn hello(&self) -> String {
        "Hello, world!!!!!!".to_string()
    }
    async fn hello_rorld(&self) -> &'static str {
        "Hello, world!"
    }
    
    async fn get_users(&self, context:&Context<'_>) -> Result<Option<Vec<User>>, String>{

        // let user_uuid = match context.data::<AuthClaims>() {
        //     Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
        //     Err(_) => return Err("UUID not found in request extensions".to_string()),
        // };
    
        let user_uuid:&AuthClaims = context.data_opt().unwrap();

        println!("HEY {:?}",user_uuid);

        let db_pool = match context.data::<PgPool>() {
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string()),
        };

        let res = match sqlx::query_as::<_, User>(
            "SELECT *,uuid::text,created_at::text FROM public.user",
        )
        .fetch_all(db_pool)
        .await
        {
            Ok(res) => res,
            Err(err) => return Err(err.to_string()),
        };

        Ok(Some(res))
    }

}