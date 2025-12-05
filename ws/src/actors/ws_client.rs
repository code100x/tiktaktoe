use actix::{ prelude::*};
use actix_web_actors::ws;
use serde::Deserialize;
use std::{time::{Duration, Instant}};
use uuid::Uuid;


use crate::{ JoinRoom, LeaveRoom, PlayerMove, RoomManager};


#[derive(Message,Clone)]
#[rtype(result="()")]
pub struct RoomMessage(pub String);


pub  struct WsClient {
    pub user_id : Uuid,
    pub room_mgr : Addr<RoomManager>,
    pub current_room : Option<Uuid>,
    hb:Instant
}

impl WsClient {
    pub fn new(user_id:Uuid,room_mgr:Addr<RoomManager>)->Self{
        Self { 
            user_id,
            room_mgr,
            current_room:None,
            hb:Instant::now()
         }
    }


}

impl Actor for WsClient{
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        log::info!("ws client started for user:{}",self.user_id);
        self.hb(ctx);

        let welcome_payload = serde_json::json!({
            "type":"connected",
            "user_id":self.user_id.to_string(),
            "message":"Connected tic-tac-toe server"
        })
        .to_string();

        ctx.text(welcome_payload);
    }
    //called when actor us stoping
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        log::info!("ws client stoppinf for user:{}",self.user_id);

        if let Some(room_id) = self.current_room{
            let leave = LeaveRoom{
                room_id,
                user_id:self.user_id
            };
            self.room_mgr.do_send(leave);
        }
        Running::Stop 
    }
}

impl Handler<RoomMessage> for WsClient {
    type Result = ();

    fn handle(&mut self, msg: RoomMessage, ctx: &mut Self::Context) -> Self::Result {
        // Simply forward the JSON string to the WebSocket client
        ctx.text(msg.0);
    }
}


#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientCmd{
    Join{
        room_id : Option<String>
    },
    Move {
        room_id:String,
        position:usize
    },
    Leave{
        room_id :String
    }
}

impl StreamHandler<Result<ws::Message,ws::ProtocolError>> for WsClient{
    fn handle(&mut self, msg: Result<ws::Message,ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Text(text))=>{
                log::debug!("Recive Message from {}: {}",self.user_id,text);
                
                match serde_json::from_str::<ClientCmd>(&text){
                    Ok(cmd)=>match cmd {
                        ClientCmd::Join { room_id }=>{
                            let room_uuid = room_id.and_then(|s|Uuid::parse_str(&s).ok());

                            let join = JoinRoom{
                                room_id:room_uuid,
                                user_id:self.user_id,
                                addr:ctx.address() 
                            };

                            let mgr = self.room_mgr.clone();
                         
                            async move {
                                mgr.send(join).await
                            }
                            .into_actor(self)  
                            .then(move |result, act, ctx| {
                                match result {
                                    Ok(Ok(room_id)) => {
                                        act.current_room = Some(room_id);
                                    }
                                    Ok(Err(e)) => {
                                        let err = serde_json::json!({
                                            "type": "error",
                                            "message": e
                                        })
                                        .to_string();
                                        ctx.text(err);
                                    }
                                    Err(_) => {
                                        let err = serde_json::json!({
                                            "type": "error",
                                            "message": "internal server error"
                                        })
                                        .to_string();
                                        ctx.text(err);
                                    }
                                }
                                fut::ready(())
                            })
                            .spawn(ctx);
                        }
                        ClientCmd::Move { room_id, position }=>{
                            if let Ok(room_uuid) = Uuid::parse_str(&room_id){
                                let mv = PlayerMove { 
                                    room_id:room_uuid, 
                                    user_id:self.user_id,
                                    position 
                                };
                                let mgr = self.room_mgr.clone();

                                async move {
                                    mgr.send(mv).await
                                }
                                .into_actor(self)
                                .then(move |result,act,ctx| {
                                    if let Ok(Err(e)) = result {
                                        let err = serde_json::json!({
                                            "type": "error",
                                            "message": e
                                        })
                                        .to_string();
                                        ctx.text(err);
                                    }
                                    fut::ready(())
                                })
                                .spawn(ctx);

                                }else {
                                // Invalid room ID format
                                let err = serde_json::json!({
                                    "type": "error",
                                    "message": "invalid room id"
                                })
                                .to_string();
                                ctx.text(err);
                            }    
                        }
                        ClientCmd::Leave { room_id }=>{
                            if let Ok(room_uuid) = Uuid::parse_str(&room_id){
                                let leave_room = LeaveRoom{
                                    room_id:room_uuid,
                                    user_id:self.user_id
                                };
                                self.room_mgr.do_send(leave_room);  //fire and forget why
                                self.current_room = None;

                                let msg = serde_json::json!({
                                    "type":"left",
                                    "room_id":room_id
                                }).to_string();
                                ctx.text(msg);
                            }
                            else {
                                let err = serde_json::json!({
                                    "type":"error",
                                    "mesg":"invaid room id"
                                }).to_string();
                                ctx.text(err);
                            }
                        }  
                    }
                    _ =>{
                        log::warn!("Invalid JSON command from {}", self.user_id);
                        let err = serde_json::json!({
                            "type": "error",
                            "message": "invalid json command"
                        })
                        .to_string();
                        ctx.text(err);
                    }
                }

            }
            _ =>{}
            
        }
    }
}