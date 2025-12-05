use std::env;
use actix_web::web::{Data, Json};
use db::{Db};
use jsonwebtoken::{EncodingKey, Header, encode};
use crate::{ models::{Claims, SigninResponse, UserRequest, UserResponse}};



pub async fn create_user(db: Data<Db>, body: Json<UserRequest>) -> Result<Json<UserResponse>, actix_web::error::Error> {
    let user = db.create_user(&body.username, &body.password)
        .await
        .map_err(|e| actix_web::error::ErrorConflict(e.to_string()))?;

    Ok(Json(UserResponse{ 
        id: user.id 
    }))
}

pub async fn sign_in(db: Data<Db>,body: Json<UserRequest>)->Result<Json<SigninResponse>,actix_web::error::Error>{
    let user = db.get_user_by_username(&body.username)
        .await
        .map_err(|e| actix_web::error::ErrorConflict(e.to_string()))?;

    if user.password != body.password {
        return Err(actix_web::error::ErrorConflict("Incorrect Password"));
    }

    let token = encode(
        &Header::default(),
        &Claims::new(user.id),
        &EncodingKey::from_secret(
            env::var("SECRET_KEY")
               .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?.as_bytes()
        )  
    ).map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;
    Ok(Json(SigninResponse{token}))
}

