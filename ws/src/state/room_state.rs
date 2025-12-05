use std::{collections::HashMap};
use actix::Addr;
use uuid::Uuid;

use crate::{GameState, WsClient};

pub struct Room {
    pub id : Uuid,
    pub players : Vec<Uuid>, //two player max and order matters player[0] = 'X' player[1] = 'O'
    pub addrs : HashMap<Uuid,Addr<WsClient>>,
    pub game :GameState
}

impl Room{
    pub fn new(id:Uuid)->Self {
        Self { 
            id,
            players :Vec::new(),
            addrs : HashMap::new(),
            game :GameState::new()
         }
    }
    //get the mark for a give player
    pub fn mark_for(&self,user:&Uuid)->Option<char>{
        match self.players.iter().position(|u|u ==user)? {
            0 =>Some('X'),
            1 =>Some('O'),
            _=>None
        }
    }

    pub fn is_full(&mut self)->bool{
        self.players.len() >= 2 
    }
    //start the game if we have 2 player
     pub fn start_game_if_ready(&mut self) {
        if self.players.len() == 2 && self.game.status == "waiting" {
            self.game.status = "playing".into();
        }
    }
}
