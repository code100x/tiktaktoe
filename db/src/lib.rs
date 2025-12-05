use anyhow::Ok;
use sqlx::PgPool;
use anyhow::Result;

use sqlx::{ postgres::PgPoolOptions};
pub mod models;

#[derive(Clone)]
pub struct Db{
    pub pool : PgPool
}

impl Db {
    pub async fn new()->Result<Self>{
        let db_rul = "postgres://anand:secret123@localhost:5432/mydb";
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_rul).await?;
        sqlx::query("SELECT 1")
            .execute(&pool)
            .await?;
        print!("Database connection successful");
        Ok(Self { 
            pool
         })

    }

}