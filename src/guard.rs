use axum::{http::{Request},response::Response, body::Body, middleware::Next};
use axum_extra::{headers::{HeaderMapExt, Authorization, authorization::Bearer}};
use jsonwebtoken::{TokenData, decode, DecodingKey, Validation};
use serde::Deserialize;
use shuttle_runtime::__internals::serde_json::json;
use uuid::Uuid;

#[derive(Deserialize, Debug)]
pub struct AuthClaims{
    pub uuid:Uuid
}

pub async fn _guard(mut req:Request<Body>, next:Next) -> Result<Response, String>{
    let token = req.headers().typed_get::<Authorization<Bearer>>()
    .ok_or(json!({"error":"No token was provided"}).to_string())?
    .token()
    .to_owned();

    let user = match _decode_jwt(token){
        Ok(user) => user,
        Err(err) => return Err(err.to_string())
    }
    .claims;
    println!("{}", user.uuid.clone());
    req.extensions_mut().insert(user.uuid);

    Ok(next.run(req).await)
}

fn _decode_jwt(jwt:String) -> Result<TokenData<AuthClaims>, String>{
    let secret = "mYsEcReTKeY".to_string().clone();
    let decoded_token = match decode(&jwt, &DecodingKey::from_secret(secret.as_ref()), &Validation::default()){
        Ok(token) => token,
        Err(err) => return Err(err.to_string())
    };
    return Ok(decoded_token);
}