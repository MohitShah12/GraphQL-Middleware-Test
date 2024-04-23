#![allow(deprecated)]

use std::str::FromStr;

use async_graphql:: *;
use axum::extract::State;
use axum::response::IntoResponse;
use chrono::{Utc, Duration};
use jsonwebtoken::{encode, Header, EncodingKey};
use serde::{Serialize, Deserialize};
use shuttle_runtime::__internals::serde_json::json;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use bcrypt::{DEFAULT_COST, hash, verify};
use crate::guard::{AuthClaims};
use crate::model::{User, Task};
use axum::Json as AxumJson;

pub struct Mutation;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims{
    iat:usize,
    exp:usize,
    uuid:Uuid
}

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

    pub async fn login_user(&self, context:&Context<'_>, email:String, password:String) -> Result<String, String>{
        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        println!("{}",email);

       let find_user = sqlx::query("SELECT * FROM public.user WHERE email = $1")
       .bind(email)
       .fetch_all(db)
       .await
       .map_err(|err| err.to_string())?;

       if !find_user.is_empty() {
        let user = &find_user[0];
        let user_pass = match user.try_get::<String, _>("password"){
            Ok(pass) => pass,
            Err(err) => return Err(err.to_string())
        };

        let verified_pass =match verify(password, &user_pass){
            Ok(verified_pass) => verified_pass,
            Err(err) => return Err(err.to_string())
        };

        if !verified_pass{
            return Err(json!({"message":"Wrong Credentials authentication failed"}).to_string());
        }

        let now = Utc::now();
        let exp = Duration::hours(24);
        let uuid = match user.try_get::<Uuid, _>("uuid") {
            Ok(uuid) => uuid,
            Err(err) => return Err(err.to_string())
        };

        let claims = Claims{
            iat: now.timestamp() as usize,
            exp:(now + exp).timestamp() as usize,
            uuid:uuid
        };

        let secret = "mYsEcReTKeY".to_string().clone();

        let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())){
            Ok(token) => token,
            Err(err) => return Err(err.to_string())
        };

        // println!("{:?}", uuid);
        Ok(token)
    } else {
        Err("No user was found with given email".to_string())
    }
        

    }

    pub async fn create_task(&self, context:&Context<'_>, title:String, desc:String) -> Result<Option<Task>, String>{
        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let user_uuid = match context.data::<AuthClaims>() {
            Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
            Err(err) => return Err(err.message.to_string()),
        };

        let taskid = Uuid::new_v4();

        let userid = Uuid::from_str(user_uuid.0.clone().unwrap().as_str()).unwrap();

        let new_row = match sqlx::query_as::<_,User>("INSERT INTO public.tasks (taskid, title, description, userid) VALUES($1, $2, $3, $4)")
        .bind(taskid)
        .bind(title.clone())
        .bind(desc.clone())
        .bind(userid)
        .fetch_all(db)
        .await
        {
            Ok(_) => {
                let user = Task{
                    taskid:taskid.to_string(),
                    title,
                    description:desc,
                    userid:user_uuid.0.unwrap()
                };
                Ok::<Task,Error>(user)
            },
            Err(err) => return Err(err.to_string()),
        }.unwrap();

        Ok(Some(new_row))
    }

    pub async fn update_task(&self, context:&Context<'_>, id:String, title:Option<String>, desc:Option<String>) -> Result<String, String>{

        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let _user_uuid = match context.data::<AuthClaims>() {
            Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
            Err(err) => return Err(err.message.to_string()),
        };

        let id = match Uuid::parse_str(id.as_str()){
            Ok(id) => id,
            Err(err) => return Err(err.to_string())
        };

        println!("{}",id);

        let get_task = sqlx::query("SELECT * FROM public.tasks WHERE taskid = $1")
        .bind(id.clone())
        .fetch_all(&db.clone())
        .await
        .map_err(|err| err.to_string())?;

        if get_task.len() == 0 {
            return Err("No tasks foung".to_string());
        }

        if title.is_none() && desc.is_none() {
            return Err("Both title and description cannot be empty".to_string());
        }

        
        let mut query = "UPDATE public.tasks SET taskid = $1".to_owned();

        let mut i = 2;

         if title.is_some(){
            query.push_str(&format!(", title = ${i}"));
            i = i+1;
         };

         if desc.is_some(){
            query.push_str(&format!(", description = ${i}"));
         };

         query.push_str(&format!(" WHERE taskid = $1"));

         let mut s = sqlx::query(&query).bind(id);

         if title.is_some(){
            s = s.bind(title);
         }

         if desc.is_some(){
            s = s.bind(desc);
         }

        let updated_task = match s.fetch_all(db)
         .await
         {
             Ok(_) => id.to_string(),
             Err(err) => return Err(err.to_string()) 
         };
 
         Ok(json!({"message":"User was updated", 
         "id":updated_task}).to_string())
    
    }

    async fn delete_task(&self, context:&Context<'_>, id:String) -> Result<String,String>{

        let db = match context.data::<PgPool>(){
            Ok(db) => db,
            Err(err) => return Err(err.message.to_string())
        };

        let _user_uuid = match context.data::<AuthClaims>() {
            Ok(uuid) => uuid.clone(), // Clone the UUID to avoid lifetime issues
            Err(err) => return Err(err.message.to_string()),
        };

        let id = match Uuid::parse_str(id.as_str()){
            Ok(id) => id,
            Err(err) => return Err(err.to_string())
        };

        let get_task = sqlx::query("SELECT * FROM public.tasks WHERE taskid = $1")
        .bind(id.clone())
        .fetch_all(&db.clone())
        .await
        .map_err(|err| err.to_string())?;

        if get_task.len() == 0 {
            return Err(json!({"Message":"No task found"}).to_string());
        }

        let _delete_row = match sqlx::query("DELETE FROM public.tasks WHERE taskid = $1")
        .bind(id.clone())
        .fetch_all(db)
        .await
        {
            Ok(_) => id.to_string(),
            Err(err) => return Err(err.to_string()) 
        };

        Ok(json!({"message":"Task was deleted"}).to_string())
    }

}

#[derive(Serialize, Deserialize)]
pub struct LoginUser{
    pub email: String,
    pub password: String,
}

pub async fn login_user_new(State(db):State<PgPool>,AxumJson(user_cred): AxumJson<LoginUser>) -> impl IntoResponse{
   let find_user = sqlx::query("SELECT * FROM public.user WHERE email = $1")
   .bind(user_cred.email)
   .fetch_all(&db)
   .await
   .map_err(|err| err.to_string())?;

   if !find_user.is_empty() {
    let user = &find_user[0];
    let user_pass = match user.try_get::<String, _>("password"){
        Ok(pass) => pass,
        Err(err) => return Err(err.to_string())
    };

    let verified_pass =match verify(user_cred.password, &user_pass){
        Ok(verified_pass) => verified_pass,
        Err(err) => return Err(err.to_string())
    };

    if !verified_pass{
        return Err(json!({"message":"Wrong Credentials authentication failed"}).to_string());
    }

    let now = Utc::now();
    let exp = Duration::hours(24);
    let uuid = match user.try_get::<Uuid, _>("uuid") {
        Ok(uuid) => uuid,
        Err(err) => return Err(err.to_string())
    };

    let claims = Claims{
        iat: now.timestamp() as usize,
        exp:(now + exp).timestamp() as usize,
        uuid:uuid
    };

    let secret = "mYsEcReTKeY".to_string().clone();

    let token = match encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref())){
        Ok(token) => token,
        Err(err) => return Err(err.to_string())
    };

    // println!("{:?}", uuid);
    Ok(json!({"Token":token}).to_string())
} else {
    Err("No user was found with given email".to_string())
}
    

}