use serde::{Deserialize, Serialize};
use uuid::Uuid;


#[derive(Serialize, Deserialize)]
pub struct UserRequest {
    pub username: String,
    pub password: String
}

#[derive(Serialize,Deserialize)]
pub struct UserResponse{
    pub id : Uuid
}

#[derive(Serialize,Deserialize)]
pub struct SigninResponse {
    pub token: String
}

#[derive(Serialize,Deserialize)]
pub struct Claims {
    pub sub : Uuid,
    pub exp : usize

}

impl Claims{
    pub fn new(sub:Uuid) ->Self{
        Self { 
            sub,
            exp:1000000000000,
        }
    }
}