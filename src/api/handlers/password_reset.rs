use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use mongodb::bson;
use utoipa::ToSchema;
use crate::services::{PasswordResetService, UserService};
use crate::auth::password::hash_password;
use crate::middleware::tenant_validator::get_tenant_id;

/// Request para solicitar reset de senha
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PasswordResetRequest {
    pub email: String,
}

/// Request para confirmar reset de senha com novo password
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct PasswordResetConfirm {
    pub token: String,
    pub new_password: String,
}

/// Response de sucesso genérico
#[derive(Debug, Serialize, ToSchema)]
pub struct SuccessResponse {
    pub message: String,
}

/// Response para solicitação de reset de senha
#[derive(Debug, Serialize, ToSchema)]
pub struct PasswordResetRequestResponse {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

/// Response para validação de token
#[derive(Debug, Serialize, ToSchema)]
pub struct TokenValidationResponse {
    pub valid: bool,
    pub email: String,
    pub expires_at: bson::DateTime,
}

/// Response para confirmação de reset
#[derive(Debug, Serialize, ToSchema)]
pub struct PasswordResetConfirmResponse {
    pub success: bool,
    pub message: String,
}

/// Response de erro
#[derive(Debug, Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/api/auth/password-reset/request",
    tag = "Password Reset",
    request_body = PasswordResetRequest,
    responses(
        (status = 200, description = "Reset token generated successfully"),
        (status = 404, description = "User not found"),
        (status = 400, description = "Invalid request")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
/// POST /api/auth/password-reset/request
/// Solicita um token de reset de senha
pub async fn request_password_reset(
    req: HttpRequest,
    data: web::Json<PasswordResetRequest>,
    reset_service: web::Data<PasswordResetService>,
    db: web::Data<mongodb::Database>,
) -> impl Responder {
    let email = &data.email;
    
    // Extrair tenant_id da requisição (adicionado pelo middleware)
    let tenant_id = match get_tenant_id(&req) {
        Some(id) => id,
        None => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Tenant ID required",
                "message": "Tenant ID must be provided"
            }));
        }
    };
    
    let user_service = UserService::new(db.get_ref().clone());
    
    // Verificar se usuário existe no tenant
    match user_service.find_by_email_and_tenant(email, &tenant_id).await {
        Ok(Some(_user)) => {
            // Obter IP do cliente para auditoria
            let ip_address = req
                .connection_info()
                .realip_remote_addr()
                .map(|s| s.to_string());
            
            // Gerar token (válido por 1 hora)
            match reset_service.create_reset_token(&tenant_id, email, 1, ip_address).await {
                Ok(token) => {
                    // TODO: Enviar email com o token
                    // Em modo desenvolvimento, retornamos o token no response
                    log::info!("Password reset token generated for {}: {}", email, token);
                    
                    // Em produção, remover o campo 'token' (definir como None)
                    let is_dev = cfg!(debug_assertions);
                    
                    HttpResponse::Ok().json(PasswordResetRequestResponse {
                        success: true,
                        message: format!("Password reset token sent to {}", email),
                        token: if is_dev { Some(token) } else { None },
                    })
                }
                Err(e) => {
                    log::error!("Failed to create reset token: {}", e);
                    HttpResponse::InternalServerError().json(ErrorResponse {
                        error: "Failed to process request".to_string(),
                    })
                }
            }
        }
        Ok(None) => {
            // Por segurança, não revelar se email existe ou não
            // Retornar sucesso mesmo se email não existir
            HttpResponse::Ok().json(PasswordResetRequestResponse {
                success: true,
                message: format!("If the email {} exists, a token has been sent.", email),
                token: None,
            })
        }
        Err(e) => {
            log::error!("Failed to fetch user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process request".to_string(),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/password-reset/validate",
    tag = "Password Reset",
    request_body = PasswordResetConfirm,
    responses(
        (status = 200, description = "Token is valid"),
        (status = 400, description = "Invalid or expired token")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
/// POST /api/auth/password-reset/validate
/// Valida se um token é válido
pub async fn validate_reset_token(
    data: web::Json<serde_json::Value>,
    reset_service: web::Data<PasswordResetService>,
) -> impl Responder {
    let token = match data.get("token").and_then(|v| v.as_str()) {
        Some(t) => t,
        None => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Token não fornecido".to_string(),
            });
        }
    };
    
    match reset_service.validate_token(token).await {
        Ok(Some(reset_token)) => HttpResponse::Ok().json(TokenValidationResponse {
            valid: true,
            email: reset_token.email,
            expires_at: reset_token.expires_at,
        }),
        Ok(None) => HttpResponse::BadRequest().json(ErrorResponse {
            error: "Invalid or expired token".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to validate token: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to validate token".to_string(),
            })
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/auth/password-reset/confirm",
    tag = "Password Reset",
    request_body = PasswordResetConfirm,
    responses(
        (status = 200, description = "Password reset successfully"),
        (status = 400, description = "Invalid or expired token"),
        (status = 404, description = "User not found")
    ),
    params(
        ("X-Tenant-ID" = String, Header, description = "Tenant ID for multi-tenancy")
    )
)]
/// POST /auth/password-reset/confirm
/// Confirma o reset de senha com novo password
pub async fn confirm_password_reset(
    data: web::Json<PasswordResetConfirm>,
    reset_service: web::Data<PasswordResetService>,
    db: web::Data<mongodb::Database>,
) -> impl Responder {
    // Validar token
    let reset_token = match reset_service.validate_token(&data.token).await {
        Ok(Some(token)) => token,
        Ok(None) => {
            return HttpResponse::BadRequest().json(ErrorResponse {
                error: "Invalid or expired token".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to validate token: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to validate token".to_string(),
            });
        }
    };
    
    // Validar força da nova senha
    if data.new_password.len() < 8 {
        return HttpResponse::BadRequest().json(ErrorResponse {
            error: "Password must be at least 8 characters long".to_string(),
        });
    }
    
    // Hash da nova senha
    let hashed_password = match hash_password(&data.new_password) {
        Ok(hash) => hash,
        Err(e) => {
            log::error!("Failed to hash password: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to process password".to_string(),
            });
        }
    };
    
    // Buscar usuário
    let user_service = UserService::new(db.get_ref().clone());
    let user = match user_service.find_by_email(&reset_token.email).await {
        Ok(Some(u)) => u,
        Ok(None) => {
            return HttpResponse::NotFound().json(ErrorResponse {
                error: "User not found".to_string(),
            });
        }
        Err(e) => {
            log::error!("Failed to fetch user: {}", e);
            return HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to fetch user".to_string(),
            });
        }
    };
    
    // Atualizar senha do usuário
    let mut updated_user = user.clone();
    updated_user.password = hashed_password;
    
    match user_service.update_user(&user._id.unwrap(), &updated_user).await {
        Ok(true) => {
            // Marcar token como usado
            let _ = reset_service.mark_token_as_used(&data.token).await;
            
            // Invalidar todos os outros tokens deste email
            let _ = reset_service
                .invalidate_all_tokens_for_email(&reset_token.email)
                .await;
            
            log::info!("Password successfully reset for: {}", reset_token.email);
            
            HttpResponse::Ok().json(PasswordResetConfirmResponse {
                success: true,
                message: "Password updated successfully".to_string(),
            })
        }
        Ok(false) => HttpResponse::InternalServerError().json(ErrorResponse {
            error: "Failed to update password".to_string(),
        }),
        Err(e) => {
            log::error!("Failed to update user: {}", e);
            HttpResponse::InternalServerError().json(ErrorResponse {
                error: "Failed to update password".to_string(),
            })
        }
    }
}
