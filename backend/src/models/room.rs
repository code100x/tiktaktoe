use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize,Deserialize)]
pub struct UserRoomRequest{
    pub id : Uuid
}


#[derive(Serialize,Deserialize)]
pub struct  UserJoinRoomRequest{
    pub room_id : Uuid,
    pub player_o_id :Uuid
}