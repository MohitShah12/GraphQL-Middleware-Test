use std::str::FromStr;

use async_graphql::{*};
use sqlx:: PgPool;
use uuid::Uuid;
use crate::model::{User, Task};
use crate::guard::{AuthClaims};

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
        
        // println!("HELLO:{:?}",context.data::<AuthClaims>());

        let _user_uuid = match context.data::<AuthClaims>() {
            Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
            Err(err) => return Err(err.message.to_string()),
        };
    
        // let user_uuid:&AuthClaims = context.data_opt().unwrap();

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

    async fn get_todos(&self, context:&Context<'_>) -> Result<Option<Vec<Task>>, String> {
        let user_uuid = match context.data::<AuthClaims>() {
            Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
            Err(err) => return Err(err.message.to_string()),
        };
    
        // let user_uuid:&AuthClaims = context.data_opt().unwrap();

        let userid = Uuid::from_str(user_uuid.0.clone().unwrap().as_str()).unwrap();

        let db_pool = match context.data::<PgPool>() {
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string()),
        };

        let res = match sqlx::query_as::<_, Task>(
            "SELECT *,taskid::text,userid::text FROM public.tasks WHERE userid = $1",
        )
        .bind(userid)
        .fetch_all(db_pool)
        .await
        {
            Ok(res) => res,
            Err(err) => return Err(err.to_string()),
        };

        Ok(Some(res))
    }


}