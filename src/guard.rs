use axum::{http::{request::Parts}, extract::{FromRequestParts}, async_trait};
use axum_extra::{headers::{HeaderMapExt, Authorization, authorization::Bearer}};
use jsonwebtoken::{TokenData, decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use shuttle_runtime::__internals::serde_json::json;

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims{
    exp:usize,
    uuid:Uuid
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthClaims(pub Option<String>);

fn decode_jwt(jwt:String) -> Result<TokenData<Claims>, String>{
    let secret = "mYsEcReTKeY".to_string().clone();
    let decoded_token = match decode(&jwt, &DecodingKey::from_secret(secret.as_ref()), &Validation::default()){
        Ok(token) => token,
        Err(err) => return Err(err.to_string())
    };
    return Ok(decoded_token);
}

#[async_trait]
impl <S> FromRequestParts<S> for AuthClaims 
where   
    S: Send + Sync
{
    type Rejection = String;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection>{
       let headers = parts.headers.clone();

       let token = headers.typed_get::<Authorization<Bearer>>()
       .ok_or(json!({"error":"No token provided"}).to_string())?
       .token()
       .to_owned();

        let user = match decode_jwt(token){
            Ok(user) => user,
            Err(err) => return Err(json!({"error":err}).to_string())
        }
        .claims;
        println!("{}", user.uuid.clone());
        
        Ok(AuthClaims(Some(user.uuid.to_string())))
    }
    
}