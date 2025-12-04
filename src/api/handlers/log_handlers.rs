use actix_web::{web, HttpRequest, HttpResponse, Result};
use serde::Deserialize;
use crate::auth::verify_jwt_token;
use crate::services::LogService;
use crate::middleware::tenant_validator::get_tenant_id;

#[derive(Deserialize)]
pub struct LogsQuery {
    limit: Option<i64>,
}

#[utoipa::path(
    get,
    path = "/api/logs/my-logins",
    tag = "Logs",
    responses(
        (status = 200, description = "User's login logs retrieved successfully"),
        (status = 401, description = "Invalid or missing token")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy"),
        ("limit" = Option<i64>, Query, description = "Maximum number of logs to return")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_my_logs(
    req: HttpRequest,
    log_service: web::Data<LogService>,
    query: web::Query<LogsQuery>,
) -> Result<HttpResponse> {
    // Verify JWT token
    let user = match verify_jwt_token(&req) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };
    
    // Get tenant_id from request extensions
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            // Fallback to method without cache if no tenant_id
            match log_service.get_user_logs(&user._id.unwrap().to_hex(), query.limit).await {
                Ok(logs) => return Ok(HttpResponse::Ok().json(serde_json::json!({
                    "data": logs,
                    "from_cache": false
                }))),
                Err(e) => {
                    eprintln!("Error fetching user logs: {}", e);
                    return Ok(HttpResponse::InternalServerError().json("Failed to fetch logs"));
                }
            }
        }
    };
    
    // Get logs for the current user with cache support
    match log_service.get_user_logs_by_tenant(&user._id.unwrap().to_hex(), &tenant_id, query.limit).await {
        Ok((logs, from_cache)) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "data": logs,
            "from_cache": from_cache
        }))),
        Err(e) => {
            eprintln!("Error fetching user logs: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch logs"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/admin/logs",
    tag = "Logs",
    responses(
        (status = 200, description = "All logs retrieved successfully (Admin only)"),
        (status = 401, description = "Invalid or missing token"),
        (status = 403, description = "Admin access required")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy"),
        ("limit" = Option<i64>, Query, description = "Maximum number of logs to return")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_all_logs(
    req: HttpRequest,
    log_service: web::Data<LogService>,
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
    
    match log_service.get_all_logs(query.limit).await {
        Ok(logs) => Ok(HttpResponse::Ok().json(logs)),
        Err(e) => {
            eprintln!("Error fetching all logs: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch logs"))
        }
    }
}

#[utoipa::path(
    get,
    path = "/api/admin/logs/stats",
    tag = "Logs",
    responses(
        (status = 200, description = "Login statistics retrieved successfully (Admin only)"),
        (status = 401, description = "Invalid or missing token"),
        (status = 403, description = "Admin access required")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_login_stats(
    req: HttpRequest,
    log_service: web::Data<LogService>,
) -> Result<HttpResponse> {
    // Verify JWT token and check admin role
    let user = match verify_jwt_token(&req) {
        Ok(user) => user,
        Err(_) => return Ok(HttpResponse::Unauthorized().json("Invalid token")),
    };

    if !user.is_admin() {
        return Ok(HttpResponse::Forbidden().json("Admin access required"));
    }
    
    // Get stats for last 30 days by default
    match log_service.get_login_stats(30).await {
        Ok(stats) => Ok(HttpResponse::Ok().json(stats)),
        Err(e) => {
            eprintln!("Error fetching login stats: {}", e);
            Ok(HttpResponse::InternalServerError().json("Failed to fetch stats"))
        }
    }
}