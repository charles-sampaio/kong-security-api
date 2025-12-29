use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use std::time::SystemTime;
use mongodb::bson::DateTime;

use crate::auth::{create_jwt_token, jwt::generate_refresh_token};
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
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
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
    // Para OAuth público, use tenant_id padrão se não fornecido
    let tenant_id = get_tenant_id(&req).unwrap_or_else(|| "default".to_string());

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

    // Gerar JWT e Refresh Token
    let token = match create_jwt_token(&user) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("JWT error: {}", e);
            return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "error": "Failed to generate token"
            })));
        }
    };
    
    let user_id_hex = user._id.unwrap().to_hex();
    let refresh_token = generate_refresh_token(&user_id_hex);
    
    // Adiciona refresh token ao usuário no banco
    let mut updated_user = user.clone();
    let mut refresh_tokens = updated_user.refresh_tokens.unwrap_or_default();
    refresh_tokens.push(refresh_token.clone());
    
    // Mantém apenas os últimos 5 refresh tokens
    if refresh_tokens.len() > 5 {
        refresh_tokens = refresh_tokens.iter().rev().take(5).rev().cloned().collect();
    }
    
    updated_user.refresh_tokens = Some(refresh_tokens);
    user_service.update_user(&updated_user).await.ok();

    // Registrar log de login
    let mut login_log = LoginLog::new(
        tenant_id.clone(),
        updated_user.email.clone(),
        true,
        ip_address,
        user_agent,
    );
    login_log.user_id = Some(user_id_hex.clone());
    login_log.token_generated = true;
    log_service.save_login_log(&login_log).await.ok();

    // Detecta se é uma requisição AJAX (Accept: application/json)
    let accept_header = req.headers()
        .get("accept")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");
    
    let is_ajax = accept_header.contains("application/json");
    
    if is_ajax {
        // Para requisições AJAX, retorna JSON diretamente (popup web)
        println!("🌐 AJAX request detected, returning JSON response");
        return Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "access_token": token,
            "refresh_token": refresh_token,
            "user": {
                "id": user_id_hex,
                "email": updated_user.email,
                "name": updated_user.name.unwrap_or_default()
            }
        })));
    }
    
    // Para navegadores normais, redireciona para oauth-callback.html (popup web ou mobile)
    // A página HTML irá detectar se é popup (window.opener) ou mobile (deep link)
    let callback_url = format!(
        "/oauth-callback.html?access_token={}&refresh_token={}&user_id={}&email={}&name={}",
        urlencoding::encode(&token),
        urlencoding::encode(&refresh_token),
        urlencoding::encode(&user_id_hex),
        urlencoding::encode(&updated_user.email),
        urlencoding::encode(&updated_user.name.unwrap_or_default())
    );
    
    println!("🔄 Redirecting to oauth-callback.html:");
    println!("   User ID: {}", user_id_hex);
    println!("   Email: {}", user.email);
    
    Ok(HttpResponse::Found()
        .append_header(("Location", callback_url))
        .finish())
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
    // Para OAuth público, use tenant_id padrão se não fornecido
    let tenant_id = get_tenant_id(&req).unwrap_or_else(|| "default".to_string());

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

    // Para mobile apps, redireciona para o app com os dados na URL
    let redirect_url = format!(
        "exp://localhost:8081/--/auth/callback?access_token={}&user_id={}&email={}&name={}",
        urlencoding::encode(&token),
        urlencoding::encode(&user._id.unwrap().to_hex()),
        urlencoding::encode(&user.email),
        urlencoding::encode(&user.name.unwrap_or_default())
    );

    Ok(HttpResponse::Found()
        .append_header(("Location", redirect_url))
        .finish())
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
