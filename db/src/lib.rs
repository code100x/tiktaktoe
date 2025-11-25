use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

use sqlx::{PgPool, postgres::PgPoolOptions};
pub mod models;

#[derive(Clone)]
pub struct Db {
    pub pool: PgPool
}

impl Db {
    pub async fn new() -> Result<Self> {
        let db_url = env::var("DATABASE_URL")?;
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url).await?;

        Ok(Self {
            pool
        })
    }
}

