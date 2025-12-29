use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Serialize, Deserialize};
use mongodb::Database;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::SystemTime;
use mongodb::bson::DateTime;
use validator::Validate;
use crate::auth::{create_jwt_token, verify_jwt_token, jwt::{verify_refresh_token, generate_jwt, generate_refresh_token}};
use crate::models::{User, LoginLog};
use crate::services::{UserService, LogService};
use crate::middleware::validation::{LoginRequest, RegisterRequest, format_validation_errors};
use crate::middleware::tenant_validator::get_tenant_id;

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
    name: String,
    roles: Vec<String>,
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

#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 400, description = "Validation error")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
pub async fn login(
    req: HttpRequest,
    db: web::Data<Database>,
    log_service: web::Data<LogService>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    // Validar entrada
    if let Err(errors) = login_req.0.validate() {
        let error_message = format_validation_errors(errors);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "message": error_message
        })));
    }

    // Extrair tenant_id da requisição (adicionado pelo middleware)
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Tenant ID required",
                "message": "Tenant ID must be provided"
            })));
        }
    };

    let user_service = UserService::new(db.get_ref().clone());
    
    let ip_address = get_client_ip(&req);
    let user_agent = get_user_agent(&req);
    
    // Create initial login log
    let mut login_log = LoginLog::new(
        tenant_id.clone(),
        login_req.email.clone(),
        false, // Initially failed, will be updated if successful
        ip_address,
        user_agent,
    );

    // Find user by email and tenant
    match user_service.find_by_email_and_tenant(&login_req.email, &tenant_id).await {
        Ok(Some(user)) => {
            // Verify password (check if user has password set - OAuth users don't)
            let password_valid = match &user.password {
                Some(pwd) => verify(&login_req.password, pwd).unwrap_or(false),
                None => {
                    // User registered via OAuth, cannot login with password
                    login_log.set_failure("User registered via OAuth - please use OAuth login".to_string());
                    if let Err(e) = log_service.save_login_log(&login_log).await {
                        eprintln!("Failed to save login log: {}", e);
                    }
                    return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                        "error": "This account uses OAuth authentication. Please sign in with Google or Apple."
                    })));
                }
            };
            
            if password_valid {
                // Generate JWT token
                match create_jwt_token(&user) {
                    Ok(token) => {
                        // Update login log as successful
                        login_log.set_success(user._id.unwrap().to_hex(), true, false);
                        
                        // Save login log
                        if let Err(e) = log_service.save_login_log(&login_log).await {
                            eprintln!("Failed to save login log: {}", e);
                        }

                        let user_email = user.email.clone();
                        let response = AuthResponse {
                            token,
                            refresh_token: None, // Login tradicional não usa refresh token por enquanto
                            user: UserResponse {
                                id: user._id.unwrap().to_hex(),
                                email: user.email,
                                name: user_email, // Use email as name for now
                                roles: user.roles.unwrap_or_default(),
                            },
                        };
                        Ok(HttpResponse::Ok().json(response))
                    }
                    Err(e) => {
                        login_log.set_failure("Token generation failed".to_string());
                        if let Err(e) = log_service.save_login_log(&login_log).await {
                            eprintln!("Failed to save login log: {}", e);
                        }
                        eprintln!("Token generation error: {}", e);
                        Ok(HttpResponse::InternalServerError().json("Internal server error"))
                    }
                }
            } else {
                login_log.set_failure("Invalid password".to_string());
                if let Err(e) = log_service.save_login_log(&login_log).await {
                    eprintln!("Failed to save login log: {}", e);
                }
                Ok(HttpResponse::Unauthorized().json("Invalid credentials"))
            }
        }
        Ok(None) => {
            login_log.set_failure("User not found".to_string());
            if let Err(e) = log_service.save_login_log(&login_log).await {
                eprintln!("Failed to save login log: {}", e);
            }
            Ok(HttpResponse::Unauthorized().json("Invalid credentials"))
        }
        Err(e) => {
            login_log.set_failure("Database error".to_string());
            if let Err(e) = log_service.save_login_log(&login_log).await {
                eprintln!("Failed to save login log: {}", e);
            }
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/register",
    tag = "Authentication",
    request_body = RegisterRequest,
    responses(
        (status = 201, description = "User registered successfully"),
        (status = 409, description = "User already exists"),
        (status = 400, description = "Validation error")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn register(
    req: HttpRequest,
    db: web::Data<Database>,
    register_req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    // Validar entrada
    if let Err(errors) = register_req.0.validate() {
        let error_message = format_validation_errors(errors);
        return Ok(HttpResponse::BadRequest().json(serde_json::json!({
            "error": "Validation failed",
            "message": error_message
        })));
    }

    // Extrair tenant_id da requisição (adicionado pelo middleware)
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Tenant ID required",
                "message": "Tenant ID must be provided"
            })));
        }
    };

    let user_service = UserService::new(db.get_ref().clone());

    // Check if user already exists in this tenant
    match user_service.find_by_email_and_tenant(&register_req.email, &tenant_id).await {
        Ok(Some(_)) => return Ok(HttpResponse::Conflict().json("User already exists")),
        Ok(None) => {}
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Internal server error"));
        }
    }

    // Hash password
    let hashed_password = match hash(&register_req.password, DEFAULT_COST) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Password hashing error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Internal server error"));
        }
    };

    // Create new user
    let now = SystemTime::now();
    let _datetime = DateTime::from_system_time(now);
    
    let new_user = User::new(
        tenant_id,
        register_req.email.clone(),
        hashed_password,
    );

    // Save user to database
    match user_service.create_user(new_user).await {
        Ok(user_id) => {
            let response = UserResponse {
                id: user_id.to_hex(),
                email: register_req.email.clone(),
                name: register_req.email.clone(), // Use email as name for now
                roles: vec!["user".to_string()],
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

#[derive(Deserialize, Validate)]
pub struct RefreshTokenRequest {
    #[validate(length(min = 10, message = "Refresh token is required"))]
    pub refresh_token: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/refresh",
    tag = "Authentication",
    request_body = RefreshTokenRequest,
    responses(
        (status = 200, description = "Token refreshed successfully", body = AuthResponse),
        (status = 401, description = "Invalid or expired refresh token"),
        (status = 400, description = "Validation error")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
pub async fn refresh_token(
    req: HttpRequest,
    refresh_req: web::Json<RefreshTokenRequest>,
    db: web::Data<Database>,
    log_service: web::Data<LogService>,
) -> Result<HttpResponse> {
    let tenant_id = get_tenant_id(&req).unwrap_or_else(|| "default".to_string());
    
    // Valida o request
    if let Err(e) = refresh_req.validate() {
        return Ok(HttpResponse::BadRequest().json(format_validation_errors(e)));
    }
    
    // Verifica o refresh token
    let refresh_claims = match verify_refresh_token(&refresh_req.refresh_token) {
        Some(claims) => claims,
        None => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid or expired refresh token"
            })));
        }
    };
    
    let user_service = UserService::new(db.get_ref().clone());
    
    // Busca o usuário
    let user = match user_service.find_by_id(&tenant_id, &refresh_claims.sub).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "User not found"
            })));
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            return Ok(HttpResponse::InternalServerError().json("Internal server error"));
        }
    };
    
    // Verifica se o refresh token está na lista de tokens válidos do usuário
    let empty_vec = vec![];
    let refresh_tokens = user.refresh_tokens.as_ref().unwrap_or(&empty_vec);
    if !refresh_tokens.contains(&refresh_req.refresh_token) {
        return Ok(HttpResponse::Unauthorized().json(serde_json::json!({
            "error": "Refresh token not found or has been revoked"
        })));
    }
    
    // Gera novo access token e refresh token
    let new_access_token = generate_jwt(
        &user._id.unwrap().to_hex(),
        &user.email,
        user.roles.clone().unwrap_or_default(),
        user.is_active,
        "crypto-exchange-aggregator",
        "kong-security-api"
    );
    
    let new_refresh_token = generate_refresh_token(&user._id.unwrap().to_hex());
    
    // Atualiza a lista de refresh tokens no banco (remove o antigo, adiciona o novo)
    let mut updated_tokens = refresh_tokens.clone();
    updated_tokens.retain(|t| t != &refresh_req.refresh_token); // Remove o token usado
    updated_tokens.push(new_refresh_token.clone()); // Adiciona o novo
    
    // Mantém apenas os últimos 5 refresh tokens
    if updated_tokens.len() > 5 {
        updated_tokens = updated_tokens.iter().rev().take(5).rev().cloned().collect();
    }
    
    let mut updated_user = user.clone();
    updated_user.refresh_tokens = Some(updated_tokens);
    updated_user.last_login = Some(DateTime::from_system_time(SystemTime::now()));
    
    if let Err(e) = user_service.update_user(&updated_user).await {
        eprintln!("Failed to update user refresh tokens: {}", e);
    }
    
    // Log de refresh (opcional)
    let ip_address = get_client_ip(&req);
    let user_agent = get_user_agent(&req);
    let mut login_log = LoginLog::new(
        tenant_id.clone(),
        user.email.clone(),
        true,
        ip_address,
        user_agent,
    );
    login_log.user_id = Some(updated_user._id.unwrap().to_hex());
    login_log.token_generated = true;
    log_service.save_login_log(&login_log).await.ok();
    
    let response = AuthResponse {
        token: new_access_token,
        refresh_token: Some(new_refresh_token),
        user: UserResponse {
            id: updated_user._id.unwrap().to_hex(),
            email: updated_user.email,
            name: updated_user.name.unwrap_or_else(|| "User".to_string()),
            roles: updated_user.roles.unwrap_or_default(),
        },
    };
    
    Ok(HttpResponse::Ok().json(response))
}

#[utoipa::path(
    get,
    path = "/api/auth/protected",
    tag = "Authentication",
    responses(
        (status = 200, description = "Access granted", body = UserResponse),
        (status = 401, description = "Invalid or missing token")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn protected(req: HttpRequest) -> Result<HttpResponse> {
    match verify_jwt_token(&req) {
        Ok(user) => {
            let user_email = user.email.clone();
            let response = UserResponse {
                id: user._id.unwrap().to_hex(),
                email: user.email,
                name: user_email, // Use email as name for now
                roles: user.roles.unwrap_or_default(),
            };
            Ok(HttpResponse::Ok().json(response))
        }
        Err(_) => Ok(HttpResponse::Unauthorized().json("Invalid token")),
    }
}