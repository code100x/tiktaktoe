use actix::Actor;
use actix_web::{App, HttpServer, HttpRequest, HttpResponse, web, Error};
use actix_web_actors::ws;
use uuid::Uuid;


pub mod actors;
pub use actors::*;
pub mod state;
pub use state::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let room_manager_addr = RoomManager::new().start();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(room_manager_addr.clone()))
            .route("/ws", web::get().to(ws_route))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}


async fn ws_route(
    req: HttpRequest, 
    stream: web::Payload, 
    room_mgr: web::Data<actix::Addr<RoomManager>>
) -> Result<HttpResponse,actix_web::Error> {
    // Parse the query string to extract user_id
    // Example: "user_id=11111111-1111-1111-1111-111111111111"
    let query = req.query_string();
    let params: Vec<_> = query.split('&').collect();
    let mut user_id_opt: Option<Uuid> = None;
    
    // Look for the user_id parameter
    for p in params {
        if let Some(rest) = p.strip_prefix("user_id=") {
            // Try to parse the value as a UUID
            if let Ok(u) = Uuid::parse_str(rest) {
                user_id_opt = Some(u);
            }
        }
    }
    
    // Validate that we got a valid user_id
    let user_id = match user_id_opt {
        Some(u) => u,
        None => {
            // No user_id or invalid format - reject the connection
            log::warn!("WebSocket connection attempt without valid user_id");
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "missing or invalid user_id query param"
            })));
        }
    };
    
    log::info!("WebSocket connection established for user: {}", user_id);
    
    // Create a new WsClient actor for this connection
    // This actor will handle all messages for this specific client
    let ws = WsClient::new(user_id, room_mgr.get_ref().clone());
    
    // Start the WebSocket actor and complete the upgrade
    // This returns an HTTP 101 Switching Protocols response
    ws::start(ws, &req, stream)
}