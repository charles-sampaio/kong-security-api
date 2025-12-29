mod models;
mod auth;
mod database;
mod services;
mod api;
mod utils;
mod config;
mod middleware;
mod api_doc;
mod cache;

use actix_web::{App, HttpServer, middleware::{Logger, Compress}, web, HttpResponse, http::header};
use actix_cors::Cors;
use dotenv::dotenv;
use database::connect_to_database;
use services::{UserService, LogService, PasswordResetService, TenantService, OAuthService, OAuthConfig};
use api::handlers::auth_handlers::*;
use api::handlers::log_handlers::*;
use api::handlers::password_reset::{
    request_password_reset,
    validate_reset_token,
    confirm_password_reset,
};
use api::handlers::oauth_handlers;
use middleware::TenantValidator;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use api_doc::ApiDoc;
use cache::{RedisCache, CacheConfig};
use std::env;

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    dotenv().ok();
    env_logger::init();

    // Conectar ao MongoDB com pool otimizado
    let db = connect_to_database().await.expect("Failed to connect to database");
    
    // Initialize database indexes
    log::info!("🔧 Initializing database indexes...");
    if let Err(e) = database::indexes::initialize_indexes(&db).await {
        log::error!("❌ Failed to initialize indexes: {}", e);
        panic!("Database indexes initialization failed");
    }
    
    // Inicializar cache Redis (opcional - sistema funciona sem cache)
    let cache = match RedisCache::new(CacheConfig::default()) {
        Ok(c) => {
            match c.ping().await {
                Ok(_) => {
                    log::info!("✅ Redis cache connected and ready");
                    Some(c)
                }
                Err(e) => {
                    log::warn!("⚠️  Redis ping failed: {}. Running without cache.", e);
                    None
                }
            }
        }
        Err(e) => {
            log::warn!("⚠️  Redis connection failed: {}. Running without cache.", e);
            None
        }
    };
    
    // Initialize services with cache
    let tenant_service = web::Data::new(TenantService::new(&db, cache.clone()));
    let user_service = web::Data::new(UserService::new(db.clone()));
    let log_service = web::Data::new(LogService::new(db.clone(), cache.clone()));
    let reset_service = web::Data::new(PasswordResetService::new(&db));
    
    // Initialize OAuth Service (Google + Apple)
    log::info!("🔧 Initializing OAuth services...");
    let google_config = OAuthConfig {
        client_id: env::var("GOOGLE_CLIENT_ID").expect("GOOGLE_CLIENT_ID must be set"),
        client_secret: env::var("GOOGLE_CLIENT_SECRET").expect("GOOGLE_CLIENT_SECRET must be set"),
        redirect_url: env::var("GOOGLE_REDIRECT_URL").expect("GOOGLE_REDIRECT_URL must be set"),
        auth_url: "https://accounts.google.com/o/oauth2/v2/auth".to_string(),
        token_url: "https://oauth2.googleapis.com/token".to_string(),
    };
    
    let apple_config = OAuthConfig {
        client_id: env::var("APPLE_CLIENT_ID").expect("APPLE_CLIENT_ID must be set"),
        client_secret: env::var("APPLE_CLIENT_SECRET").expect("APPLE_CLIENT_SECRET must be set"),
        redirect_url: env::var("APPLE_REDIRECT_URL").expect("APPLE_REDIRECT_URL must be set"),
        auth_url: "https://appleid.apple.com/auth/authorize".to_string(),
        token_url: "https://appleid.apple.com/auth/token".to_string(),
    };
    
    let oauth_service = web::Data::new(
        OAuthService::new(Some(google_config), Some(apple_config))
            .expect("Failed to initialize OAuth service")
    );
    log::info!("✅ OAuth services initialized (Google + Apple)");
    
    log::info!("✅ Database connected successfully");
    
    // Log startup information with URLs
    log_startup_info(cache.is_some());

    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();

    HttpServer::new(move || {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000") // Frontend web origin
            .allowed_origin("http://localhost:8080") // API origin
            .allowed_origin("http://localhost:8081") // Expo Metro bundler (React Native)
            .allowed_origin("http://localhost:19006") // Expo web
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
            .wrap(Compress::default()) // Compressão automática (gzip/brotli)
            .wrap(Logger::default())
            
            // Application data
            .app_data(web::Data::new(db.clone()))
            .app_data(tenant_service.clone())
            .app_data(user_service.clone())
            .app_data(log_service.clone())
            .app_data(reset_service.clone())
            .app_data(oauth_service.clone())
            .app_data(web::JsonConfig::default().limit(512 * 1024)) // 512KB JSON limit (reduzido)
            
            // Health check endpoint (sem validação de tenant)
            .route("/api/health", web::get().to(health_check))
            
            // OAuth callback HTML page (serve arquivo estático)
            .route("/oauth-callback.html", web::get().to(|| async {
                let html = include_str!("../static/oauth-callback.html");
                HttpResponse::Ok()
                    .content_type("text/html; charset=utf-8")
                    .body(html)
            }))
            
            // Swagger UI (sem validação de tenant)
            .service(
                SwaggerUi::new("/api/swagger-ui/{_:.*}")
                    .url("/api/api-docs/openapi.json", openapi.clone())
            )
            
            // Public OAuth routes (NO Tenant Validation required)
            .service(
                web::scope("/api/auth")
                    .route("/google", web::get().to(oauth_handlers::google_auth_url))
                    .route("/google/callback", web::get().to(oauth_handlers::google_callback))
                    .route("/apple", web::get().to(oauth_handlers::apple_auth_url))
                    .route("/apple/callback", web::get().to(oauth_handlers::apple_callback))
            )
            
            // API routes WITH Tenant Validation
            .service(
                web::scope("/api")
                    .wrap(TenantValidator::new(db.clone()))
                    // Authentication (requires tenant)
                    .service(
                        web::scope("/auth")
                            .route("/login", web::post().to(login))
                            .route("/register", web::post().to(register))
                            .route("/refresh", web::post().to(refresh_token))
                            .route("/protected", web::get().to(protected))
                            // Password Reset endpoints
                            .route("/password-reset/request", web::post().to(request_password_reset))
                            .route("/password-reset/validate", web::post().to(validate_reset_token))
                            .route("/password-reset/confirm", web::post().to(confirm_password_reset))
                    )
                    // Tenants Management (Admin only)
                    .service(
                        web::scope("/tenants")
                            .route("", web::post().to(api::handlers::tenant_handlers::create_tenant))
                            .route("", web::get().to(api::handlers::tenant_handlers::list_tenants))
                            .route("/{tenant_id}", web::get().to(api::handlers::tenant_handlers::get_tenant))
                            .route("/{tenant_id}", web::put().to(api::handlers::tenant_handlers::update_tenant))
                            .route("/{tenant_id}", web::delete().to(api::handlers::tenant_handlers::delete_tenant))
                            .route("/{tenant_id}/activate", web::post().to(api::handlers::tenant_handlers::activate_tenant))
                            .route("/{tenant_id}/deactivate", web::post().to(api::handlers::tenant_handlers::deactivate_tenant))
                    )
                    // Logs
                    .service(
                        web::scope("/logs")
                            .route("/my-logins", web::get().to(get_my_logs))
                    )
                    // Admin
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

/// Log server startup information
fn log_startup_info(cache_enabled: bool) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║          🚀 Kong Security API - Server Started              ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  📍 Server URL:    http://localhost:8080/api                ║");
    println!("║  📊 Swagger UI:    http://localhost:8080/api/swagger-ui/    ║");
    println!("║  📖 OpenAPI Spec:  http://localhost:8080/api/api-docs/openapi.json ║");
    println!("║  ❤️  Health Check:  http://localhost:8080/api/health         ║");
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
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ⚡ Performance Features:                                    ║");
    println!("║     {} Redis Cache (Tenants & Logs)", if cache_enabled { "✅" } else { "⚠️ " });
    println!("║     ✅ MongoDB Connection Pooling (max 10)                   ║");
    println!("║     ✅ Response Compression (gzip/brotli)                    ║");
    println!("║     ✅ Optimized Database Queries                            ║");
    println!("║     ✅ Zstd/Snappy MongoDB Compression                       ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}

/// Health check endpoint
async fn health_check(
    db: web::Data<mongodb::Database>,
    tenant_service: web::Data<TenantService>,
) -> HttpResponse {
    // Check database connection
    let db_status = match db.list_collection_names().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    // Check cache status (via tenant_service)
    let cache_status = if tenant_service.get_ref().cache.is_some() {
        "enabled"
    } else {
        "disabled"
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "1.1.0",
        "database": db_status,
        "cache": cache_status,
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
        ],
        "performance_features": [
            "Redis Cache (Tenants & Logs)",
            "MongoDB Connection Pooling",
            "Response Compression",
            "Optimized Queries",
            "Zstd/Snappy Compression"
        ]
    }))
}
