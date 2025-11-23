use actix_web::{web, HttpRequest, HttpResponse, Result};
use mongodb::Database;
use serde::Deserialize;
use crate::auth::verify_jwt_token;
use crate::models::{User, LoginLog};
use crate::services::LogService;

#[derive(Deserialize)]
pub struct LogsQuery {
    limit: Option<i64>,
}

pub async fn get_my_logs(
    req: HttpRequest,
    db: web::Data<Database>,
    query: web::Query<LogsQuery>,
) -> Result<HttpResponse> {
    // Verify JWT token
    let user = match verify_jwt_token(&req) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    let log_service = LogService::new(db.get_ref().clone());
    
    // Get logs for the current user
    match log_service.get_user_logs(&user._id.unwrap().to_hex(), query.limit).await {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => {
            eprintln!("Error fetching user logs: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch logs"))
        }
    }
}

pub async fn get_all_logs(
    req: HttpRequest,
    db: web::Data<Database>,
    query: web::Query<LogsQuery>,
) -> Result<HttpResponse> {
    // Verify JWT token and check admin role
    let user = match verify_jwt_token(&req) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let log_service = LogService::new(db.get_ref().clone());
    
    match log_service.get_all_logs(query.limit).await {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => {
            eprintln!("Error fetching all logs: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch logs"))
        }
    }
}

pub async fn get_login_stats(
    req: HttpRequest,
    db: web::Data<Database>,
) -> Result<HttpResponse> {
    // Verify JWT token and check admin role
    let user = match verify_jwt_token(&req) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }

    let log_service = LogService::new(db.get_ref().clone());
    
    // Get stats for last 30 days by default
    match log_service.get_login_stats(30).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => {
            eprintln!("Error fetching login stats: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch stats"))
        }
    }
}