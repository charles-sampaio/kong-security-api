use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    error::ErrorUnauthorized,
    Error, HttpMessage,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use mongodb::Database;
use crate::services::TenantService;

/// Middleware para validar tenant_id
pub struct TenantValidator {
    db: Database,
}

impl TenantValidator {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

impl<S, B> Transform<S, ServiceRequest> for TenantValidator
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = TenantValidatorMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(TenantValidatorMiddleware {
            service: Rc::new(service),
            db: self.db.clone(),
        }))
    }
}

pub struct TenantValidatorMiddleware<S> {
    service: Rc<S>,
    db: Database,
}

impl<S, B> Service<ServiceRequest> for TenantValidatorMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let db = self.db.clone();

        Box::pin(async move {
            // Extrai tenant_id do header X-Tenant-ID ou query parameter
            let tenant_id = req
                .headers()
                .get("X-Tenant-ID")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
                .or_else(|| {
                    req.query_string()
                        .split('&')
                        .find_map(|pair| {
                            let mut parts = pair.split('=');
                            if parts.next() == Some("tenant_id") {
                                parts.next().map(|s| s.to_string())
                            } else {
                                None
                            }
                        })
                });

            let tenant_id = match tenant_id {
                Some(id) => id,
                None => {
                    return Err(ErrorUnauthorized(serde_json::json!({
                        "error": "Tenant ID is required",
                        "message": "Please provide tenant_id in X-Tenant-ID header or as query parameter"
                    }).to_string()));
                }
            };

            // Valida se o tenant existe e está ativo (sem cache no middleware)
            let tenant_service = TenantService::new(&db, None);
            match tenant_service.validate_tenant(&tenant_id).await {
                Ok((true, _)) => {
                    // Adiciona tenant_id às extensões da requisição para uso posterior
                    req.extensions_mut().insert(tenant_id.clone());
                    
                    let res = service.call(req).await?;
                    Ok(res)
                }
                Ok((false, _)) => {
                    Err(ErrorUnauthorized(serde_json::json!({
                        "error": "Tenant validation failed",
                        "message": "Tenant is inactive"
                    }).to_string()))
                }
                Err(e) => {
                    Err(ErrorUnauthorized(serde_json::json!({
                        "error": "Tenant validation failed",
                        "message": e
                    }).to_string()))
                }
            }
        })
    }
}

/// Helper para extrair tenant_id das extensões da requisição
pub fn get_tenant_id(req: &actix_web::HttpRequest) -> Option<String> {
    req.extensions().get::<String>().cloned()
}
