mod models;
mod auth;
mod database;
mod services;
mod api;
mod utils;
mod config;

use actix_web::{App, HttpServer, middleware::Logger, web};
use dotenv::dotenv;
use database::connect_to_database;
use api::handlers::auth_handlers::*;
use api::handlers::log_handlers::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    let db = connect_to_database().await.expect("Failed to connect to database");
    println!("Server started");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(db.clone()))
            .service(
                web::scope("/api")
                    .route("/login", web::post().to(login))
                    .route("/register", web::post().to(register))
                    .route("/protected", web::get().to(protected))
                    .service(
                        web::scope("/logs")
                            .route("/my-logins", web::get().to(get_my_logs))
                    )
                    .service(
                        web::scope("/admin")
                            .route("/logs", web::get().to(get_all_logs))
                            .route("/logs/stats", web::get().to(get_login_stats))
                    )
            )
    })
    .bind(("127.0.0.1", 8080))
    .expect("Failed to bind server to address")
    .run()
    .await
}
