#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Listening on 127.0.0.1:8080");
    HttpServer::new(|| App::new().route("/ws", web::get().to(ws_entry)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn ws_entry(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, actix_web::Error> {
    let actor = ws_actor::MyWs::new();
    ws::start(actor, &req, stream)
}

//# high level what is code does means 
// This code defines a WebSocket actor in Actix.
// Each WebSocket client connection creates one instance of this actor, which is driven by the Actix event loop (Arbiter).
// This actor:
// receives WebSocket frames from the client
// processes them
// sends frames back to the client
//It runs in: Thread (worker) → Arbiter → WebsocketContext → Actor (MyWs)










// src/actors/room_manager.rs
use actix::prelude::*;
use std::collections::HashMap;
use uuid::Uuid;

// Forward-declare WsClient type so RegisterClient can carry its Addr.
// Ensure module order in lib/main allows this.
use crate::actors::ws_client::WsClient;
//RoomManager is an Actor that keeps track of all connected WebSocket clients.

pub struct RoomManager {
    pub clients : HashMap<Uuid, Addr<WsClient>>
    //it stores UUid Addss of ws client actor
}

impl RoomManager { //initalisation of empty hashmap
    pub fn new () ->Self {
        Self { clients: HashMap::new() }
    }
}

impl Actor for RoomManager {  // this is how we declare that RoomManager is an Actix Actor and every actor needs a Context , Context <Self> means :
    //this actor run inside Actix event loop , it can recevie messages , it has its own mailbox and there is no started function mesn nothing run authmatically on startup
    type Context =  Context<Self>;
}

//This is a message sent to RoomManager when a new WebSocket connects.
//its contains userid and addres of ws client actor
pub struct  RegisterClient {
    pub user_id : Uuid,
    pub addr : Addr<WsClient>
}
//mark this as an actix message
impl Message for RegisterClient {  //This tells Actix: "RegisterClient is a message"
    type Result = (); //mean The actor handling this message does not return anything ,No future is returned to the sender
}


impl Handler<RegisterClient> for RoomManager {
    type Result = ();

  
  
   {
        self.clients.insert(msg.user_id, msg.addr);
        println!("Registered client: {}", msg.user_id);
    }
}


pub struct UnregisterClient {
    pub user_id : Uuid
}

impl Message for UnregisterClient{
    type Result =  ();
}
impl Handler <UnregisterClient> for RoomManager {
    type Result = ();

    fn handle(&mut self,msg:UnregisterClient,_:&mut Context<Self>){
        self.clients.remove(&msg.user_id);
        print!("Unregister Client:{}",msg.user_id)
    }
}











// src/actors/ws_client.rs
use actix::prelude::*;
use actix_web_actors::ws;
use uuid::Uuid;
use crate::actors::room_manager::{RegisterClient, UnregisterClient, RoomManager};
use std::time::{Duration, Instant};


pub struct WsClient {
    pub user_id : Uuid,
    pub room_mgr : Addr<RoomManager>,
    pub hb : Instant //Timestamp of the last heartbeat (ping/pong).
}

impl WsClient {
    pub fn new(user_id:Uuid,room_mgr : Addr<RoomManager>) ->Self {
        Self { 
            user_id,
            room_mgr,
            hb : Instant::now()
        }
    }
    
}
// This tells Actix: WsClient is an actor , It runs inside a WebsocketContext
//Why websocket Context : Unlike normal actors(Context<self>)

//this special context handles:
// WebSocket frame input/output
// Automatic ping/pong
// Sending text/binary frames to the client
// Connection closing


impl Actor for WsClient {
    type Context = ws::WebsocketContext<Self>;

    fn started (&mut self,ctx:&mut Self::Context) {
        self.room_mgr.do_send( 
            RegisterClient{
                user_id:self.user_id,
                addr: ctx.address()    //Gets the actor address of the WsClient
            }
        );
        println!("WsClient started for user: {}", self.user_id);

    }
    fn stopping(&mut self, _:&mut Self::Context)->Running{
        self.room_mgr.do_send(UnregisterClient{
            user_id : self.user_id
        });
        Running::Stop
    }

}

//self.room_mgr is an Addr<RoomManager>
//this address 
//knows which thread RoomManager is running on
//knows where RoomManager’s mailbox is
//allows sending messages without waiting for a response
//do_send(...) places a message into RoomManager's mailbox
//=>   This is an asynchronous, fire-and-forget delivery.
//Under the hood: WsClient Thread
                //     |
                // do_send()
                //     |
                // [ RoomManager mailbox queue ] --> [RoomManager actor loop]

// Since the RoomManager is running in its own event loop:
// It will eventually pick the message from its mailbox.
// It will call its handler for RegisterClient:

//ANd the important thing is  => The handle() code does NOT run inside WsClient’s thread.
//The handle() code does NOT run inside WsClient’s thread.







//Now stream heandler => is a traits that tell Actix whenevner websocket frame comes call my handler  So your actor is “reacting” to messages as they arrive.
//A WebSocket frame is a packet of data with metadata describing what kind of message it is.
// Each frame contains:
// Opcode → type of frame
// (text, binary, ping, pong, close, continuation)
// Payload length
// Payload data (actual message)
// Flags (FIN bit, masked bit, etc.)
//Frames are the building blocks of WebSockets.
impl StreamHandler<Result<ws::Message,ws::ProtocolError>> for WsClient { //WsClient will handle a stream of WebSocket frames , Each frame is wrapped in Result<ws::Message, ProtocolError>
    //Actix calls this every time a WebSocket frame arrives.
    fn handle(&mut self,  msg: Result<ws::Message,ws::ProtocolError>, ctx: &mut Self::Context) {
          match msg {
            Ok(ws::Message::Ping(data))=>{
                self.hb = Instant::now();
                let _ = ctx.pong(&data);
            }
            Ok(ws::Message::Pong(_))=>{
                self.hb = Instant::now()
            }
            Ok(ws::Message::Text(text))=>{
                ctx.text(text);
            }
            Ok(ws::Message::Binary(_bin)) => {

            }
            _ =>{}
          }
    }

}


// MESSAGES DO NOT CALL FUNCTIONS
// This is the MOST important thing for your mental model.
// Actors NEVER call each other directly.
// This is NOT happening:
// ws_client.register_client();

// Instead:
// Actors communicate through message passing, like this:
// do_send(RegisterClient { ... })
// do_send() pushes a message into the target actor’s mailbox.
// Then sometime later:

// RoomManager event loop wakes up
// Reads message
// Runs the handler
// This is why Actix scales:
// No locks
// No shared state
// No threads blocking
// Everything is async and queued






// Imagine each actor is a "person" sitting in a room with an inbox (mailbox).
// WsClient
// Receives WebSocket events
// Sends requests to RoomManager
// RoomManager
// Receives registration/unregistration messages
// Keeps a big map of clients
// Can send messages back
// Actors do NOT interrupt each other.
// They do NOT run at the same time.
// They process messages one-by-one.

// This makes concurrency SIMPLE and SAFE.



// SHORT SUMMARY FOR PERFECT MENTAL MODEL
// RoomManager:
// Knows everyone connected
//Stores actor addresses
// Cleans up on disconnect
// Runs as a single async actor
// WsClient:
// Represents a single socket connection
// Receives WebSocket frames
// Sends WebSocket responses
// Registers itself with RoomManager
// Unregisters on disconnect
// Dies cleanly
// Messages:
// Actors communicate ONLY through messages
// Messages go into mailboxes
// Actors NEVER call each other directly
// WebSocket frames:
// Arrive as a STREAM
// Your actor handler reacts to each frame
// System:
// Async
// Non-blocking
// Event-driven
// Safe concurrency











# ctx in an Actix actor handler means:

The actor’s context — the runtime environment in which the actor is running.

In simple words:ctx = the control panel for your actor.
It gives your actor the ability to:
send messages to itself
schedule timers
stop itself
write WebSocket frames (for WsClient)
spawn futures
access the actor’s event loop

Everything an actor “does” besides simple computation goes through ctx.

# WHY DOES EVERY HANDLER RECEIVE ctx?

Because Actix actors are asynchronous and event-driven.

When an actor handles a message, sometimes it needs to:
send a response
schedule another action
push something to the client (for WebSockets)
stop itself (ctx.stop())
wait for async future (ctx.spawn)

The context is what allows all this.