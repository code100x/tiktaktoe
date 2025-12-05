use std::{collections::HashMap};
use actix::{Actor, Addr, Context, Handler, Message};
use uuid::Uuid;

use crate::{Room, RoomMessage, WsClient};



#[derive(Message)]
#[rtype(result="Result<Uuid,String>")]   
pub struct JoinRoom{
    pub room_id:Option<Uuid>,
    pub user_id :Uuid,
    pub addr : Addr<WsClient>
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct LeaveRoom{
    pub room_id:Uuid,
    pub user_id:Uuid
}

#[derive(Message)]
#[rtype(result = "Result<(),String>")]
pub struct PlayerMove{
    pub room_id:Uuid,
    pub user_id : Uuid,
    pub position : usize 
}

pub struct RoomManager{
    pub rooms:HashMap<Uuid,Room>, 
    pub user_room : HashMap<Uuid,Uuid>  
}


impl RoomManager{
    pub fn new()->Self{
        Self { 
            rooms:HashMap::new(),
            user_room:HashMap::new()
        }
    }
}


impl Actor for RoomManager{
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("Room manger actor statrted");
    }
}

impl Handler<JoinRoom> for RoomManager{
    type Result = Result<Uuid,String>;
    fn handle(&mut self, msg: JoinRoom, _: &mut Context<Self>) -> Self::Result {
       
        if let Some(&existing_room_id) = self.user_room.get(&msg.user_id) {
            if let Some(room) = self.rooms.get_mut(&existing_room_id) {
                
                room.addrs.insert(msg.user_id, msg.addr.clone());
                
                let mark = room.mark_for(&msg.user_id).unwrap_or('X');
                let payload = serde_json::json!({
                    "type": "rejoined",
                    "room_id": room.id.to_string(),
                    "board": room.game.board,
                    "your_mark": mark.to_string(),
                    "turn": room.game.turn.to_string(),
                    "status": room.game.status,
                    "winner": room.game.winner,
                    "players": room.players.len(),
                })
                .to_string();

                if let Some(a) = room.addrs.get(&msg.user_id) {
                    let _ = a.do_send(RoomMessage(payload));
                }

                return Ok(existing_room_id);
            }
            
        }
        let room_id = if let Some(rid) = msg.room_id{
            if !self.rooms.contains_key(&rid){
                return Err("room not found".into());
            }
            rid
        }else {
            Uuid::new_v4()
        };

        let room = self.rooms.entry(room_id)
            .or_insert_with(||{   
            log::info!("Created new room: {}", room_id);
            Room::new(room_id) 
        });

        if room.is_full() {
            return Err("room is full".into());
        }
        room.players.push(msg.user_id);
        room.addrs.insert(msg.user_id,msg.addr.clone());
        self.user_room.insert(msg.user_id,room_id);

        room.start_game_if_ready();

        let mark = room.mark_for(&msg.user_id).unwrap();

         let payload = serde_json::json!({
            "type": "joined",
            "room_id": room_id.to_string(),
            "board": room.game.board,
            "your_mark": mark.to_string(),
            "turn": room.game.turn.to_string(),
            "status": room.game.status,
            "players": room.players.len(),
        })
        .to_string();
         
        if let Some(a) = room.addrs.get(&msg.user_id){
            let _ = a.do_send(RoomMessage(payload.clone()));
        }
        for(uid,a) in room.addrs.iter(){
            if uid != &msg.user_id {
                let other_payload = serde_json::json!({
                    "type":"player_joined",
                    "room_id":room_id.to_string(),
                    "board":room.game.board,
                    "turn":room.game.turn.to_string(),
                    "status":room.game.status,
                    "players":room.players.len()
                })
                .to_string();
            let _ = a.do_send(RoomMessage(other_payload));
            }
        }
        log::info!(
            "Player {} joined room {} (players: {})",
            msg.user_id,
            room_id,
            room.players.len()
        );
        Ok(room_id)
    }
}


impl Handler<LeaveRoom> for RoomManager{
    type Result = ();
    fn handle(&mut self, msg: LeaveRoom, ctx: &mut Self::Context) -> Self::Result {
        
        if let Some(room) = self.rooms.get_mut(&msg.room_id){
            room.players.retain(|u|u != &msg.user_id);  
            room.addrs.remove(&msg.user_id);

        
            self.user_room.remove(&msg.user_id);

            log::info!(
                "player {} left from {} (remaining players:{}) ",
                msg.user_id,
                msg.room_id,
                room.players.len()
            );
            //Notify this thing to others player

            let payload = serde_json::json!({
                "type":"player-left",
                "user_id":msg.user_id,
                "room_id": msg.room_id,
                "player":room.players.len()
            })
            .to_string();
            
            for(_,a) in room.addrs.iter(){
                let _ = a.do_send(RoomMessage(payload.clone()));
            }

            if room.players.is_empty(){
                self.rooms.remove(&msg.room_id);
                log::info!("")
            }

        }
    }
}

impl Handler<PlayerMove> for RoomManager{
    type Result = Result<(),String>;
    fn handle(&mut self, msg: PlayerMove, ctx: &mut Self::Context) -> Self::Result {
        let room = self.rooms
            .get_mut(&msg.room_id)
            .ok_or_else(||"room not found".to_string())?;

        if !room.players.contains(&msg.user_id){
            return Err("user not in the room".into());
        }

        let mark = room
            .mark_for(&msg.user_id)
            .ok_or_else(||"user has no mark".to_string())?;

        room.game.apply_move(msg.position,mark);

        log::info!(
            "Player {} ({}) moved to position {} in room {}",
            msg.user_id,
            mark,
            msg.position,
            room.id
        );

        //build updated payload of game state
        let payload = serde_json::json!({
            "payload":"palyer_moved",
            "room_id":msg.room_id,
            "board":room.game.board,
            "turn":room.game.turn,
            "status":room.game.status,
            "winner":room.game.winner,
            "last_move":{
                "position":msg.position,
                "mark":mark.to_string()
            }
        })
        .to_string();

        for(_,a) in room.addrs.iter(){
            let _ = a.do_send(RoomMessage( payload.clone()));
        }
        if room.game.status != "playing" {
            log::info!(
                "Game ended in room {}: {}",
                room.id,
                if let Some(w) = room.game.winner {
                    format!("Winner: {}", w)
                } else {
                    "Draw".to_string()
                }
            );
        }
        Ok(())
    }
}

