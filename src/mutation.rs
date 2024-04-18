#![allow(deprecated)]
use async_graphql:: *;
use chrono::Utc;
use shuttle_runtime::__internals::serde_json::json;
use sqlx::{PgPool};
use uuid::Uuid;
use bcrypt::{DEFAULT_COST, hash};

use crate::model::User;

pub struct Mutation;

#[Object]
impl Mutation{
   pub async fn create_user(&self, context:&Context<'_>, name:String, email:String, password:String) -> Result<Option<User>,String>{
        let db =match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let uuid = Uuid::new_v4();
        let created_at = Utc::now().naive_utc().timestamp();
        let hash = hash(password, DEFAULT_COST).unwrap();

        let new_row = match sqlx::query_as::<_,User>("INSERT INTO public.user (name, email, password, uuid, created_at)
            VALUES($1, $2, $3, $4, $5)")
        .bind(name.clone())
        .bind(email.clone())
        .bind(hash)
        .bind(uuid)
        .bind(created_at)
        .fetch_all(db)
        .await
        {
            Ok(_) => {
                let user = User{
                    name,
                    email,
                    uuid:uuid.to_string(),
                    created_at:created_at.to_string()
                };
                Ok::<User,Error>(user)
            },
            Err(err) => return Err(err.to_string()),
        }.unwrap();

        Ok(Some(new_row))
        // println!("What'sup");
        // println!("HEY: {:?}",new_row.uuid);
    }

    pub async fn update_user(&self, context:&Context<'_>, name:Option<String>, id:String) -> Result<String, String>{
        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let id = match Uuid::parse_str(id.as_str()){
            Ok(id) => id,
            Err(err) => return Err(err.to_string())
        };

        let get_user = sqlx::query("SELECT * FROM public.user WHERE uuid = $1")
        .bind(id.clone())
        .fetch_all(&db.clone())
        .await
        .map_err(|err| err.to_string())?;
        
        if get_user.len() == 0 {
            return Err(json!({"Message":"No user was found with given id"}).to_string());
        }

        let update_row_id = match sqlx::query_as::<_, User>("
            UPDATE public.user SET name = $1 WHERE uuid = $2
        ")
        .bind(name)
        .bind(id)
        .fetch_all(db)
        .await
        {
            Ok(_) => id.to_string(),
            Err(err) => return Err(err.to_string()) 
        };

        Ok(json!({"message":"User was updated", 
        "id":update_row_id}).to_string())
    }

    pub async fn delete_user(&self, context:&Context<'_>, id:String) -> Result<String, String> {
        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let id = match Uuid::parse_str(id.as_str()){
            Ok(id) => id,
            Err(err) => return Err(err.to_string())
        };

        let get_user = sqlx::query("SELECT * FROM public.user WHERE uuid = $1")
        .bind(id.clone())
        .fetch_all(&db.clone())
        .await
        .map_err(|err| err.to_string())?;
        
        if get_user.len() == 0 {
            return Err(json!({"Message":"No user was found with given id"}).to_string());
        }

        let _delete_row = match sqlx::query_as::<_,User>("DELETE FROM public.user WHERE uuid = $1")
        .bind(id)
        .fetch_all(db)
        .await
        {
            Ok(_) => id.to_string(),
            Err(err) => return Err(err.to_string()) 
        };

        Ok(json!({"message":"User was deleted"}).to_string())
    }
}