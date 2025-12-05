use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::Result;

use crate::Db;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Room {
    pub id: Uuid,

    pub player_x_id: Uuid,
    pub player_o_id: Option<Uuid>,

    pub board_state: String,
    pub next_turn: String,
    pub winner: Option<String>,

    pub status: String,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


impl Db {
    pub async fn create_room(&self, player_x_id: Uuid) -> Result<Room> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            INSERT INTO rooms (player_x_id)
            VALUES ($1)
            RETURNING *
            "#
        )
        .bind(player_x_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(room)
    }

    pub async fn get_room_by_room_id(&self, room_id: Uuid) -> Result<Room> {
        let room = sqlx::query_as::<_, Room>(
            r#"
            SELECT *
            FROM rooms
            WHERE id = $1
            "#
        )
        .bind(room_id)
        .fetch_one(&self.pool)
        .await?;

        Ok(room)
    }
 
    pub async fn join_room(&self, room_id: Uuid, player_o_id: Uuid) -> Result<Room> {
      let room = sqlx::query_as::<_, Room>(
            r#"
            UPDATE rooms
            SET player_o_id = $2,
                status = 'playing',
                updated_at = NOW()
            WHERE id = $1
              AND player_o_id IS NULL
              AND status = 'waiting'
            RETURNING *
            "#
      )
      .bind(room_id)
      .bind(player_o_id)
      .fetch_one(&self.pool)
      .await
      .map_err(|err| {
          anyhow::anyhow!("Failed to join room: {}", err)
      })?;
      Ok(room)
    }
   
}
