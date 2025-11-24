mod models;
mod auth;
mod database;
mod services;
mod api;
mod utils;
mod config;
mod middleware;
mod api_doc;

use actix_web::{App, HttpServer, middleware::Logger, web, HttpResponse, http::header};
use actix_cors::Cors;
use dotenv::dotenv;
use database::connect_to_database;
use services::{UserService, LogService, PasswordResetService};
use api::handlers::auth_handlers::*;
use api::handlers::log_handlers::*;
use api::handlers::tenant_handlers::configure_tenant_routes;
use api::handlers::password_reset::{
    request_password_reset,
    validate_reset_token,
    confirm_password_reset,
};
use middleware::TenantValidator;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use api_doc::ApiDoc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    let db = connect_to_database().await.expect("Failed to connect to database");
    
    // Initialize database indexes
    log::info!("🔧 Initializing database indexes...");
    if let Err(e) = database::indexes::initialize_indexes(&db).await {
        log::error!("❌ Failed to initialize indexes: {}", e);
        panic!("Database indexes initialization failed");
    }
    
    // Initialize services
    let user_service = web::Data::new(UserService::new(db.clone()));
    let log_service = web::Data::new(LogService::new(db.clone()));
    let reset_service = web::Data::new(PasswordResetService::new(&db));
    
    log::info!("✅ Database connected successfully");
    
    // Log startup information with URLs
    log_startup_info();

    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Frontend origin
            .allowed_origin("http://localhost:8080") // API origin
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600)
            .supports_credentials();

        App::new()
            // Global middleware
            .wrap(cors)
            .wrap(Logger::default())
            
            // Application data
            .app_data(web::Data::new(db.clone()))
            .app_data(user_service.clone())
            .app_data(log_service.clone())
            .app_data(reset_service.clone())
            .app_data(web::JsonConfig::default().limit(1024 * 1024)) // 1MB JSON limit
            
            // Health check endpoint (sem validação de tenant)
            .route("/health", web::get().to(health_check))
            
            // Swagger UI (sem validação de tenant)
            .service(
                SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-docs/openapi.json", openapi.clone())
            )
            
            // Tenant Management routes (sem validação de tenant para gerenciar tenants)
            .configure(configure_tenant_routes)
            
            // API routes with Tenant Validation
            .service(
                web::scope("")
                    .wrap(TenantValidator::new(db.clone()))
                    // Authentication
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(login))
                            .route("/register", web::post().to(register))
                            .route("/protected", web::get().to(protected))
                            // Password Reset endpoints
                            .route("/password-reset/request", web::post().to(request_password_reset))
                            .route("/password-reset/validate", web::post().to(validate_reset_token))
                            .route("/password-reset/confirm", web::post().to(confirm_password_reset))
                    )
                    // Logs
                    .service(
                        web::scope("/api/logs")
                            .route("/my-logins", web::get().to(get_my_logs))
                    )
                    // Admin
                    .service(
                        web::scope("/api/admin")
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

/// Log server startup information
fn log_startup_info() {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          🚀 Kong Security API - Server Started              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  📍 Server URL:    http://localhost:8080                    ║");
    println!("║  📊 Swagger UI:    http://localhost:8080/swagger-ui/        ║");
    println!("║  📖 OpenAPI Spec:  http://localhost:8080/api-docs/openapi.json ║");
    println!("║  ❤️  Health Check:  http://localhost:8080/health             ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  🔒 Security Features Enabled:                              ║");
    println!("║     ✅ CORS Protection                                       ║");
    println!("║     ✅ JWT RS256 Authentication                              ║");
    println!("║     ✅ BCrypt Password Hashing                               ║");
    println!("║     ✅ Input Validation & Sanitization                       ║");
    println!("║     ✅ Password Strength Requirements                        ║");
    println!("║     ✅ SQL Injection Prevention                              ║");
    println!("║     ✅ XSS Prevention                                        ║");
    println!("║     ✅ Comprehensive Audit Logging                           ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}

/// Health check endpoint
async fn health_check(db: web::Data<mongodb::Database>) -> HttpResponse {
    // Check database connection
    let db_status = match db.list_collection_names().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "1.0.0",
        "database": db_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "security_features": [
            "CORS Protection",
            "JWT RS256 Authentication",
            "BCrypt Password Hashing",
            "Password Strength Validation",
            "Email Format Validation",
            "Input Sanitization",
            "SQL Injection Prevention",
            "Comprehensive Audit Logging"
        ]
    }))
}
