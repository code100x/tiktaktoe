use serde::{Deserialize, Serialize};

#[derive(Serialize,Debug,Deserialize,Clone)]
pub struct GameState {
    pub board : [Option<char>;9],
    pub winner :Option<char>,
    pub status : String,
    pub turn : char
}


impl GameState {
    pub fn new()->Self {
        Self { 
            board:[None;9],
            winner : None,
            status:"waiting".into(),
            turn:'X'
         }
    }
    pub fn is_playing(&self)->bool{
        self.status == "playing"
    }
    pub fn apply_move (&mut self,position:usize,mark:char)->Result<(),String>{

        if !self.is_playing(){
            return Err("game is not started yet".into());
        }
        if mark != self.turn {
            return Err("its not your turn buddy".into());
        }
        if position >= 9 {
            return  Err("Invalid position".into());
        }
        if self.board[position].is_some() {
            return Err("cell is already accoupied ser choose another".into());
        }
        self.board[position]= Some(mark);

        self.evaluate();

        if self.status == "playing"{
            self.turn = if self.turn == 'X'{
                'O'
            }else {
                'X'
            }

        
        }
        Ok(())

    }
    pub fn evaluate(&mut self){
        const WINS:[(usize,usize,usize);8] = [
            (0,1,2),(3,4,5),(6,7,8),
            (0,3,6),(1,4,7),(2,5,8),
            (0,4,8),(2,4,6)
        ];
        for (a, b, c) in WINS {
            if let (Some(x), Some(y), Some(z)) = (self.board[a], self.board[b], self.board[c]) {
                if x == y && y == z {
                    self.status = "won".into();
                    self.winner = Some(x);
                    return;
                }
            }
        }
        if self.board.iter().all(|c|c.is_some()){
            self.status = "draw".into();
            self.winner = None
        }
    }

}
