use actix_web::{App, HttpServer, web};
use db::Db;

use crate::routes::user::{create_user, me_handler, sign_in};

pub mod routes;
pub mod middleware;

#[actix_web::main]
async fn main() {
    dotenvy::dotenv().unwrap();
    let db = Db::new().await.unwrap();
    let _ = HttpServer::new(move || {
        App::new()
            .service(web::resource("/signup").route(web::post().to(create_user)))
            .service(web::resource("/signin").route(web::post().to(sign_in)))
            .wrap(my_middleware)
            .service(web::resource("/me").route(web::get().to(me_handler)))
            .app_data(actix_web::web::Data::new(db.clone()))

    })
    .bind("0.0.0.0:3000")
    .unwrap()
    .run()
    .await;

}


async fn my_middleware(
    req: ServiceRequest,
    next: Next<impl MessageBody>,
) -> Result<ServiceResponse<impl MessageBody>, Error> {
    // pre-processing
    next.call(req).await
    // post-processing
}
