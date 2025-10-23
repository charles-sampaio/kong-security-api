use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use mongodb::Database;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::time::SystemTime;
use mongodb::bson::DateTime;
use crate::auth::{create_jwt_token, verify_jwt_token};
use crate::models::{User, LoginLog};
use crate::services::{UserService, LogService};

#[derive(Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    email: String,
    password: String,
    name: String,
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

pub async fn login(
    req: HttpRequest,
    db: web::Data<Database>,
    login_req: web::Json<LoginRequest>,
) -> Result<HttpResponse> {
    let user_service = UserService::new(db.get_ref().clone());
    let log_service = LogService::new(db.get_ref().clone());
    
    let ip_address = get_client_ip(&req);
    let user_agent = get_user_agent(&req);
    
    // Create initial login log
    let mut login_log = LoginLog::new(
        login_req.email.clone(),
        false, // Initially failed, will be updated if successful
        ip_address,
        user_agent,
    );

    // Find user by email
    match user_service.find_by_email(&login_req.email).await {
        Ok(Some(user)) => {
            // Verify password
            if verify(&login_req.password, &user.password).unwrap_or(false) {
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

pub async fn register(
    db: web::Data<Database>,
    register_req: web::Json<RegisterRequest>,
) -> Result<HttpResponse> {
    let user_service = UserService::new(db.get_ref().clone());

    // Check if user already exists
    match user_service.find_by_email(&register_req.email).await {
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
        register_req.email.clone(),
        hashed_password,
    );

    // Save user to database
    match user_service.create_user(&new_user).await {
        Ok(_) => {
            let new_user_email = new_user.email.clone();
            let response = UserResponse {
                id: new_user._id.unwrap().to_hex(),
                email: new_user.email,
                name: new_user_email, // Use email as name for now
                roles: new_user.roles.unwrap_or_default(),
            };
            Ok(HttpResponse::Created().json(response))
        }
        Err(e) => {
            eprintln!("Database error: {}", e);
            Ok(HttpResponse::InternalServerError().json("Internal server error"))
        }
    }
}

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