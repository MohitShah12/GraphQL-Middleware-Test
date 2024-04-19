use axum::{http::{Request, request::Parts},response::Response, body::Body, middleware::Next, extract::{FromRequest, FromRequestParts}, Extension, RequestPartsExt};
use axum_extra::{headers::{HeaderMapExt, Authorization, authorization::Bearer}};
use jsonwebtoken::{TokenData, decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use shuttle_runtime::__internals::serde_json::json;
use uuid::Uuid;
use async_graphql::{Context, InputObject, Object};






// pub async fn guard(mut req:Request<Body>, next:Next) -> Result<Response, String>{
//     let token = req.headers().typed_get::<Authorization<Bearer>>()
//     .ok_or(json!({"error":"No token was provided"}).to_string())?
//     .token()
//     .to_owned();

//     let user = match decode_jwt(token){
//         Ok(user) => user,
//         Err(err) => return Err(err.to_string())
//     }
//     .claims;
//     println!("{}", user.uuid.clone());
//     let auth_user = AuthClaims{
//         uuid:user.uuid.to_string(),
//     };

//     req.extensions_mut().insert(auth_user);

//     println!("{:?}",req.extensions());

//     Ok(next.run(req).await)
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims{
    exp:usize,
    uuid:Uuid
}

#[derive(Serialize, Deserialize, Debug, Clone, InputObject)]
pub struct AuthClaims{
    pub uuid:String
}

fn decode_jwt(jwt:String) -> Result<TokenData<Claims>, String>{
    let secret = "mYsEcReTKeY".to_string().clone();
    let decoded_token = match decode(&jwt, &DecodingKey::from_secret(secret.as_ref()), &Validation::default()){
        Ok(token) => token,
        Err(err) => return Err(err.to_string())
    };
    return Ok(decoded_token);
}

#[axum::async_trait]
impl <S> FromRequestParts<S> for AuthClaims 
where   
    S: Send + Sync
{
    type Rejection = &'static str;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection>{
       let headers = parts.headers.clone();

       let token = headers.typed_get::<Authorization<Bearer>>()
       .ok_or("No token provided")?
       .token()
       .to_owned();

        let user = match decode_jwt(token){
            Ok(user) => user,
            Err(err) => return Err("There was problem with decoding token")
        }
        .claims;
        println!("{}", user.uuid.clone());

        let auth_user = AuthClaims{
            uuid:user.uuid.to_string(),
        };
        
        Ok(auth_user)
    }
    
}