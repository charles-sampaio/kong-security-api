use actix_web::{web, HttpResponse, Responder};
use crate::models::{CreateTenantRequest, UpdateTenantRequest, TenantResponse};
use crate::services::TenantService;
use mongodb::Database;

/// Cria um novo tenant
/// 
/// # Exemplo de requisição:
/// ```json
/// {
///   "name": "ACME Corporation",
///   "description": "Main tenant for ACME Corporation"
/// }
/// ```
#[utoipa::path(
    post,
    path = "/api/tenants",
    tag = "Tenants",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created successfully", body = TenantResponse),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn create_tenant(
    db: web::Data<Database>,
    tenant_data: web::Json<CreateTenantRequest>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service.create_tenant(tenant_data.into_inner()).await {
        Ok(tenant) => {
            let response: TenantResponse = tenant.into();
            HttpResponse::Created().json(response)
        }
        Err(e) => {
            HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Failed to create tenant",
                "message": e
            }))
        }
    }
}

/// Lista todos os tenants
/// 
/// Query parameter:
/// - active_only: boolean (opcional) - se true, retorna apenas tenants ativos
#[utoipa::path(
    get,
    path = "/api/tenants",
    tag = "Tenants",
    params(
        ("active_only" = Option<bool>, Query, description = "Filter only active tenants")
    ),
    responses(
        (status = 200, description = "List of tenants", body = Vec<TenantResponse>),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_tenants(
    db: web::Data<Database>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> impl Responder {
    let active_only = query
        .get("active_only")
        .and_then(|v| v.parse::<bool>().ok())
        .unwrap_or(false);

    let tenant_service = TenantService::new(&db);

    match tenant_service.list_tenants(active_only).await {
        Ok(tenants) => {
            let responses: Vec<TenantResponse> = tenants
                .into_iter()
                .map(|t| t.into())
                .collect();
            HttpResponse::Ok().json(responses)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to list tenants",
            "message": e
        })),
    }
}

/// Busca um tenant específico pelo tenant_id
#[utoipa::path(
    get,
    path = "/api/tenants/{tenant_id}",
    tag = "Tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID to retrieve")
    ),
    responses(
        (status = 200, description = "Tenant found", body = TenantResponse),
        (status = 404, description = "Tenant not found")
    )
)]
pub async fn get_tenant(
    db: web::Data<Database>,
    tenant_id: web::Path<String>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service.get_tenant(&tenant_id).await {
        Ok(Some(tenant)) => {
            let response: TenantResponse = tenant.into();
            HttpResponse::Ok().json(response)
        }
        Ok(None) => HttpResponse::NotFound().json(serde_json::json!({
            "error": "Tenant not found",
            "message": format!("Tenant with ID '{}' does not exist", tenant_id)
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to retrieve tenant",
            "message": e
        })),
    }
}

/// Atualiza um tenant existente
/// 
/// # Exemplo de requisição:
/// ```json
/// {
///   "name": "ACME Corporation Updated",
///   "description": "Updated description",
///   "active": true
/// }
/// ```
#[utoipa::path(
    put,
    path = "/api/tenants/{tenant_id}",
    tag = "Tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID to update")
    ),
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Tenant updated successfully", body = TenantResponse),
        (status = 404, description = "Tenant not found"),
        (status = 400, description = "Invalid request")
    )
)]
pub async fn update_tenant(
    db: web::Data<Database>,
    tenant_id: web::Path<String>,
    update_data: web::Json<UpdateTenantRequest>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service
        .update_tenant(&tenant_id, update_data.into_inner())
        .await
    {
        Ok(tenant) => {
            let response: TenantResponse = tenant.into();
            HttpResponse::Ok().json(response)
        }
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Tenant not found",
                    "message": e
                }))
            } else {
                HttpResponse::BadRequest().json(serde_json::json!({
                    "error": "Failed to update tenant",
                    "message": e
                }))
            }
        }
    }
}

/// Desativa um tenant (soft delete)
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/deactivate",
    tag = "Tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID to deactivate")
    ),
    responses(
        (status = 200, description = "Tenant deactivated successfully"),
        (status = 404, description = "Tenant not found")
    )
)]
pub async fn deactivate_tenant(
    db: web::Data<Database>,
    tenant_id: web::Path<String>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service.deactivate_tenant(&tenant_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Tenant deactivated successfully"
        })),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Tenant not found",
                    "message": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to deactivate tenant",
                    "message": e
                }))
            }
        }
    }
}

/// Ativa um tenant
#[utoipa::path(
    post,
    path = "/api/tenants/{tenant_id}/activate",
    tag = "Tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID to activate")
    ),
    responses(
        (status = 200, description = "Tenant activated successfully"),
        (status = 404, description = "Tenant not found")
    )
)]
pub async fn activate_tenant(
    db: web::Data<Database>,
    tenant_id: web::Path<String>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service.activate_tenant(&tenant_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Tenant activated successfully"
        })),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Tenant not found",
                    "message": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to activate tenant",
                    "message": e
                }))
            }
        }
    }
}

/// Deleta permanentemente um tenant (use com cuidado!)
#[utoipa::path(
    delete,
    path = "/api/tenants/{tenant_id}",
    tag = "Tenants",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID to delete permanently")
    ),
    responses(
        (status = 200, description = "Tenant deleted successfully"),
        (status = 404, description = "Tenant not found")
    )
)]
pub async fn delete_tenant(
    db: web::Data<Database>,
    tenant_id: web::Path<String>,
) -> impl Responder {
    let tenant_service = TenantService::new(&db);

    match tenant_service.delete_tenant(&tenant_id).await {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "message": "Tenant deleted permanently"
        })),
        Err(e) => {
            if e.contains("not found") {
                HttpResponse::NotFound().json(serde_json::json!({
                    "error": "Tenant not found",
                    "message": e
                }))
            } else {
                HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": "Failed to delete tenant",
                    "message": e
                }))
            }
        }
    }
}

pub fn configure_tenant_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/tenants")
            .route("", web::post().to(create_tenant))
            .route("", web::get().to(list_tenants))
            .route("/{tenant_id}", web::get().to(get_tenant))
            .route("/{tenant_id}", web::put().to(update_tenant))
            .route("/{tenant_id}", web::delete().to(delete_tenant))
            .route("/{tenant_id}/activate", web::post().to(activate_tenant))
            .route("/{tenant_id}/deactivate", web::post().to(deactivate_tenant)),
    );
}
