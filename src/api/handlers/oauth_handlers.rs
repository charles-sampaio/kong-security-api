use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use std::time::SystemTime;
use mongodb::bson::DateTime;

use crate::auth::create_jwt_token;
use crate::models::{User, user::OAuthProvider, LoginLog};
use crate::services::{UserService, LogService, OAuthService};
use crate::middleware::tenant_validator::get_tenant_id;

#[derive(Deserialize)]
pub struct OAuthCallbackQuery {
    code: String,
    state: Option<String>, // CSRF token
    #[serde(rename = "id_token")]
    id_token: Option<String>, // Apple pode retornar isso
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

#[derive(Serialize)]
pub struct UserResponse {
    id: String,
    email: String,
    name: Option<String>,
    picture: Option<String>,
    roles: Vec<String>,
}

#[derive(Serialize)]
pub struct AuthUrlResponse {
    auth_url: String,
    state: String, // CSRF token para o frontend verificar
}

fn get_client_ip(req: &HttpRequest) -> Option<String> {
    req.connection_info()
        .peer_addr()
        .map(|addr| addr.to_string())
}

fn get_user_agent(req: &HttpRequest) -> Option<String> {
    req.headers()
        .get("user-agent")
        .and_then(|value| value.to_str().ok())
        .map(|s| s.to_string())
}

/// GET /api/auth/google
/// Retorna a URL de autorização do Google
#[utoipa::path(
    get,
    path = "/api/auth/google",
    tag = "OAuth",
    responses(
        (status = 200, description = "Google authorization URL generated"),
        (status = 500, description = "OAuth not configured")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
pub async fn google_auth_url(
    oauth_service: web::Data<OAuthService>,
) -> Result<HttpResponse> {
    match oauth_service.get_google_auth_url() {
        Ok((auth_url, state)) => {
            Ok(HttpResponse::Ok().json(AuthUrlResponse {
                auth_url,
                state,
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth configuration error",
                "message": e
            })))
        }
    }
}

/// GET /api/auth/apple
/// Retorna a URL de autorização da Apple
#[utoipa::path(
    get,
    path = "/api/auth/apple",
    tag = "OAuth",
    responses(
        (status = 200, description = "Apple authorization URL generated"),
        (status = 500, description = "OAuth not configured")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
pub async fn apple_auth_url(
    oauth_service: web::Data<OAuthService>,
) -> Result<HttpResponse> {
    match oauth_service.get_apple_auth_url() {
        Ok((auth_url, state)) => {
            Ok(HttpResponse::Ok().json(AuthUrlResponse {
                auth_url,
                state,
            }))
        }
        Err(e) => {
            Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "OAuth configuration error",
                "message": e
            })))
        }
    }
}

/// GET /api/auth/google/callback
/// Callback do Google OAuth (troca code por token e cria/loga usuário)
#[utoipa::path(
    get,
    path = "/api/auth/google/callback",
    tag = "OAuth",
    responses(
        (status = 200, description = "User authenticated successfully", body = AuthResponse),
        (status = 401, description = "Authentication failed"),
        (status = 500, description = "Server error")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy"),
        ("code" = String, Query, description = "Authorization code from Google"),
        ("state" = Option<String>, Query, description = "CSRF token")
    )
)]
pub async fn google_callback(
    req: HttpRequest,
    query: web::Query<OAuthCallbackQuery>,
    db: web::Data<Database>,
    log_service: web::Data<LogService>,
    oauth_service: web::Data<OAuthService>,
) -> Result<HttpResponse> {
    // Extrair tenant_id
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Tenant ID required"
            })));
        }
    };

    // Autenticar com Google
    let user_info = match oauth_service.authenticate_google(query.code.clone()).await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Google OAuth error: {}", e);
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication failed",
                "message": format!("{}", e)
            })));
        }
    };

    let user_service = UserService::new(db.get_ref().clone());
    let ip_address = get_client_ip(&req);
    let user_agent = get_user_agent(&req);

    // Buscar ou criar usuário
    let user = match user_service.find_by_oauth(&tenant_id, OAuthProvider::Google, &user_info.oauth_id).await {
        Ok(Some(mut existing_user)) => {
            // Usuário existe, atualizar last_login
            existing_user.last_login = Some(DateTime::from_system_time(SystemTime::now()));
            user_service.update_user(&existing_user).await.ok();
            existing_user
        }
        Ok(None) => {
            // Usuário não existe, criar novo
            let new_user = User::from_oauth(
                tenant_id.clone(),
                user_info.email.clone(),
                OAuthProvider::Google,
                user_info.oauth_id.clone(),
                user_info.name.clone(),
                user_info.picture.clone(),
            );
            
            match user_service.create_user(new_user.clone()).await {
                Ok(_) => new_user,
                Err(e) => {
                    eprintln!("Failed to create user: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create user"
                    })));
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            })));
        }
    };

    // Gerar JWT
    let token = match create_jwt_token(&user) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate token"
            })));
        }
    };

    // Registrar log de login
    let mut login_log = LoginLog::new(
        tenant_id.clone(),
        user.email.clone(),
        true,
        ip_address,
        user_agent,
    );
    login_log.user_id = Some(user._id.unwrap().to_hex());
    login_log.token_generated = true;
    log_service.save_login_log(&login_log).await.ok();

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse {
            id: user._id.unwrap().to_hex(),
            email: user.email,
            name: user.name,
            picture: user.picture,
            roles: user.roles.unwrap_or_default(),
        },
    }))
}

/// GET /api/auth/apple/callback
/// Callback da Apple OAuth (troca code por token e cria/loga usuário)
#[utoipa::path(
    get,
    path = "/api/auth/apple/callback",
    tag = "OAuth",
    responses(
        (status = 200, description = "User authenticated successfully", body = AuthResponse),
        (status = 401, description = "Authentication failed"),
        (status = 500, description = "Server error")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy"),
        ("code" = String, Query, description = "Authorization code from Apple"),
        ("id_token" = Option<String>, Query, description = "ID token from Apple"),
        ("state" = Option<String>, Query, description = "CSRF token")
    )
)]
pub async fn apple_callback(
    req: HttpRequest,
    query: web::Query<OAuthCallbackQuery>,
    db: web::Data<Database>,
    log_service: web::Data<LogService>,
    oauth_service: web::Data<OAuthService>,
) -> Result<HttpResponse> {
    // Extrair tenant_id
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Tenant ID required"
            })));
        }
    };

    // Autenticar com Apple
    let user_info = match oauth_service.authenticate_apple(
        query.code.clone(),
        query.id_token.clone()
    ).await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Apple OAuth error: {}", e);
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Authentication failed",
                "message": format!("{}", e)
            })));
        }
    };

    let user_service = UserService::new(db.get_ref().clone());
    let ip_address = get_client_ip(&req);
    let user_agent = get_user_agent(&req);

    // Buscar ou criar usuário
    let user = match user_service.find_by_oauth(&tenant_id, OAuthProvider::Apple, &user_info.oauth_id).await {
        Ok(Some(mut existing_user)) => {
            // Usuário existe, atualizar last_login
            existing_user.last_login = Some(DateTime::from_system_time(SystemTime::now()));
            user_service.update_user(&existing_user).await.ok();
            existing_user
        }
        Ok(None) => {
            // Usuário não existe, criar novo
            let new_user = User::from_oauth(
                tenant_id.clone(),
                user_info.email.clone(),
                OAuthProvider::Apple,
                user_info.oauth_id.clone(),
                user_info.name.clone(),
                user_info.picture.clone(),
            );
            
            match user_service.create_user(new_user.clone()).await {
                Ok(_) => new_user,
                Err(e) => {
                    eprintln!("Failed to create user: {}", e);
                    return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                        "error": "Failed to create user"
                    })));
                }
            }
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Database error"
            })));
        }
    };

    // Gerar JWT (Apple callback)
    let token = match create_jwt_token(&user) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate token"
            })));
        }
    };

    // Registrar log de login (Apple)
    let mut login_log = LoginLog::new(
        tenant_id.clone(),
        user.email.clone(),
        true,
        ip_address,
        user_agent,
    );
    login_log.user_id = Some(user._id.unwrap().to_hex());
    login_log.token_generated = true;
    log_service.save_login_log(&login_log).await.ok();

    Ok(HttpResponse::Ok().json(AuthResponse {
        token,
        user: UserResponse {
            id: user._id.unwrap().to_hex(),
            email: user.email,
            name: user.name,
            picture: user.picture,
            roles: user.roles.unwrap_or_default(),
        },
    }))
}

pub fn configure_oauth_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/auth")
            .route("/google", web::get().to(google_auth_url))
            .route("/google/callback", web::get().to(google_callback))
            .route("/apple", web::get().to(apple_auth_url))
            .route("/apple/callback", web::get().to(apple_callback))
    );
}
