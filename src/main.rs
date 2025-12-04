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

use actix_web::{web, HttpResponse, http::header, middleware::{Logger, Compress}};
use actix_cors::Cors;
use shuttle_actix_web::ShuttleActixWeb;
use shuttle_runtime::SecretStore;
use mongodb::Client;
use services::{UserService, LogService, PasswordResetService, TenantService};
use api::handlers::auth_handlers::*;
use api::handlers::log_handlers::*;
use api::handlers::password_reset::{
    request_password_reset,
    validate_reset_token,
    confirm_password_reset,
};
use middleware::TenantValidator;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use api_doc::ApiDoc;
use cache::{RedisCache, CacheConfig};

#[shuttle_runtime::main]
async fn main(
    #[shuttle_runtime::Secrets] secrets: SecretStore,
) -> ShuttleActixWeb<impl FnOnce(&mut web::ServiceConfig) + Send + Clone + 'static> {
    
    // Initialize logger
    env_logger::init();
    
    // Get secrets
    let mongodb_uri = secrets.get("MONGODB_URI")
        .expect("MONGODB_URI secret not found");
    let redis_url = secrets.get("REDIS_URL"); // Redis é opcional
    let _jwt_secret = secrets.get("JWT_SECRET")
        .expect("JWT_SECRET secret not found");
    
    log::info!("🚀 Starting Kong Security API with Shuttle...");
    
    // Connect to MongoDB
    let client = Client::with_uri_str(&mongodb_uri)
        .await
        .expect("Failed to connect to MongoDB");
    
    let db = client.database("kong-security-api");
    
    // Initialize database indexes
    log::info!("🔧 Initializing database indexes...");
    if let Err(e) = database::indexes::initialize_indexes(&db).await {
        log::error!("❌ Failed to initialize indexes: {}", e);
        panic!("Database indexes initialization failed");
    }
    
    // Initialize Redis cache (optional)
    let cache = if let Some(redis_url_str) = redis_url {
        match RedisCache::new(CacheConfig {
            redis_url: redis_url_str.clone(),
            max_connections: 10,
            connection_timeout: std::time::Duration::from_secs(5),
            default_ttl: 300, // 5 minutos padrão
        }) {
            Ok(c) => {
                match c.ping().await {
                    Ok(_) => {
                        log::info!("✅ Redis cache connected");
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
        }
    } else {
        log::info!("ℹ️  Redis URL not provided. Running without cache.");
        None
    };
    
    // Initialize services
    let tenant_service = web::Data::new(TenantService::new(&db, cache.clone()));
    let user_service = web::Data::new(UserService::new(db.clone()));
    let log_service = web::Data::new(LogService::new(db.clone(), cache.clone()));
    let reset_service = web::Data::new(PasswordResetService::new(&db));
    
    log::info!("✅ Services initialized successfully");
    
    // Generate OpenAPI spec
    let openapi = ApiDoc::openapi();
    
    // Log startup info
    log_startup_info(cache.is_some());
    
    let config = move |cfg: &mut web::ServiceConfig| {
        // Configure CORS
        let cors = Cors::default()
            .allowed_origin("http://localhost:3000")
            .allowed_origin("http://localhost:8080")
            .allowed_origin_fn(|origin, _| {
                // Allow Shuttle domains
                origin.as_bytes().ends_with(b"shuttleapp.rs")
            })
            .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
            .allowed_headers(vec![
                header::AUTHORIZATION,
                header::ACCEPT,
                header::CONTENT_TYPE,
            ])
            .max_age(3600)
            .supports_credentials();
        
        cfg
            // Global middleware
            .app_data(web::Data::new(db.clone()))
            .app_data(tenant_service.clone())
            .app_data(user_service.clone())
            .app_data(log_service.clone())
            .app_data(reset_service.clone())
            .app_data(web::JsonConfig::default().limit(512 * 1024))
            .service(
                web::scope("")
                    .wrap(cors)
                    .wrap(Compress::default())
                    .wrap(Logger::default())
                    
                    // Health check endpoint
                    .route("/api/health", web::get().to(health_check))
                    
                    // Swagger UI
                    .service(
                        SwaggerUi::new("/api/swagger-ui/{_:.*}")
                            .url("/api/api-docs/openapi.json", openapi.clone())
                    )
                    
                    // API routes with Tenant Validation
                    .service(
                        web::scope("/api")
                            .wrap(TenantValidator::new(db.clone()))
                            // Authentication
                            .service(
                                web::scope("/auth")
                                    .route("/login", web::post().to(login))
                                    .route("/register", web::post().to(register))
                                    .route("/protected", web::get().to(protected))
                                    // Password Reset
                                    .route("/password-reset/request", web::post().to(request_password_reset))
                                    .route("/password-reset/validate", web::post().to(validate_reset_token))
                                    .route("/password-reset/confirm", web::post().to(confirm_password_reset))
                            )
                            // Tenants Management
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
            );
    };
    
    Ok(config.into())
}

/// Log server startup information
fn log_startup_info(cache_enabled: bool) {
    println!("\n╔══════════════════════════════════════════════════════════════╗");
    println!("║       🚀 Kong Security API - Shuttle Deployment             ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  📍 Deployment:    Shuttle.rs                               ║");
    println!("║  📊 Swagger UI:    /api/swagger-ui/                        ║");
    println!("║  ❤️  Health Check:  /api/health                             ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  🔒 Security Features Enabled:                              ║");
    println!("║     ✅ CORS Protection                                       ║");
    println!("║     ✅ JWT RS256 Authentication                              ║");
    println!("║     ✅ BCrypt Password Hashing                               ║");
    println!("║     ✅ Multi-Tenant Isolation                                ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  ⚡ Performance Features:                                    ║");
    println!("║     {} Redis Cache", if cache_enabled { "✅" } else { "⚠️ " });
    println!("║     ✅ MongoDB Connection Pooling                            ║");
    println!("║     ✅ Response Compression                                  ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");
}

/// Health check endpoint
async fn health_check(
    db: web::Data<mongodb::Database>,
    tenant_service: web::Data<TenantService>,
) -> HttpResponse {
    let db_status = match db.list_collection_names().await {
        Ok(_) => "connected",
        Err(_) => "disconnected",
    };

    let cache_status = if tenant_service.get_ref().cache.is_some() {
        "enabled"
    } else {
        "disabled"
    };

    HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "version": "2.0.0-shuttle",
        "platform": "shuttle.rs",
        "database": db_status,
        "cache": cache_status,
        "timestamp": chrono::Utc::now().to_rfc3339(),
    }))
}

