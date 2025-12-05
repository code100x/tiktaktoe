use actix_web::{web::{Data, Json}, Error};
use db::{Db, models::Room};
use crate::models::{ UserJoinRoomRequest, UserRoomRequest};

pub async fn create_room(
    db: Data<Db>,
    body: Json<UserRoomRequest>,
) -> Result<Json<Room>, Error> {

    let room = db
        .create_room(body.id)
        .await
        .map_err(|e| {
            println!("DB Error: {:?}", e);
            actix_web::error::ErrorInternalServerError("Failed to create room")
        })?;

    Ok(Json(room))
}

pub async fn get_room(
    db:Data<Db>,
    body: Json<UserRoomRequest>
)->Result<Json<Room>,Error>{
    let room = db
        .get_room_by_room_id(body.id)
        .await
        .map_err(|e|{
            print!("DB error : {:?}",e);
            actix_web::error::ErrorInternalServerError("Failed to get room")
        })?;
    Ok(Json(room))
}

pub async fn join_rooms(
    db:Data<Db>,
    body: Json<UserJoinRoomRequest>
)->Result<Json<Room>,Error>{
    let room = db
        .join_room(body.room_id,body.player_o_id)
        .await
        .map_err(|e|{
            print!("DB error : {:?}",e);
            actix_web::error::ErrorInternalServerError("Failde to get room")
        })?;
    Ok(Json(room))
    
}