use actix_web::{App, HttpServer, web,};
pub mod routes;
pub use routes::*;
pub mod models;
pub mod  middleware;
pub use middleware::*;

#[actix_web::main]
async fn main(){
    dotenvy::dotenv().unwrap();
    let db = db::Db::new()
        .await
        .expect("Failed to connect to database");
    let _ = HttpServer::new(move || {  //move || makes a closure that captures the db variable so each worker thread gets a clone.
        App::new()
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/signin").route(web::post().to(sign_in)))
            .service(web::resource("/create_room").route(web::post().to(create_room)))
            .service(web::resource("/get_room").route(web::get().to(get_room)))
            .service(web::resource("/join_room").route(web::post().to(join_rooms)))
            .app_data(actix_web::web::Data::new(db.clone()))
    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await;
    
}
