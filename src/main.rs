mod handlers;
mod routes;
mod models;
mod auth;
mod db;

use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use db::get_db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    let db = get_db().await;
    println!("🚀 Server started at http://localhost:8080");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .configure(routes::config) 
    })
    .bind(("127.0.0.1", 8080))
    .expect("Failed to bind server to address")
    .run()
    .await
}
