use crate::Db;
use anyhow::{Ok, Result};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
#[derive(Serialize,Deserialize)]
pub struct CreateUserResponse {
    pub id : Uuid
}

#[derive(Serialize,Deserialize)]
pub struct User{
    pub id :Uuid,
    pub username : String,
    pub password : String
}

impl Db{

    pub async fn create_user(&self, username: &String, password: &String) -> Result<CreateUserResponse> {
        let u = sqlx::query_as!(CreateUserResponse, "INSERT INTO users (username, password) VALUES ($1, $2) RETURNING id", username, password)
            .fetch_one(&self.pool)
            .await?;

        Ok(CreateUserResponse {
            id: u.id
        })

    }
    pub async fn get_user_by_username(&self, username: &String) -> Result<User> {
        let u = sqlx::query_as!(User, "SELECT id, username, password FROM users WHERE username=$1", username)
            .fetch_one(&self.pool)
            .await?;

        Ok(u)
    }


}