use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse, body::BoxBody,
};
use futures_util::future::LocalBoxFuture;
use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RateLimiter {
    /// Requisições permitidas por janela de tempo
    max_requests: u32,
    /// Duração da janela de tempo
    window: Duration,
    /// Armazenamento de rate limits por IP
    storage: Arc<Mutex<HashMap<String, RequestCounter>>>,
}

#[derive(Clone)]
struct RequestCounter {
    count: u32,
    window_start: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_secs),
            storage: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Rate limiter padrão: 100 requisições por minuto
    pub fn default() -> Self {
        Self::new(100, 60)
    }

    /// Rate limiter estrito para endpoints sensíveis: 10 requisições por minuto
    pub fn strict() -> Self {
        Self::new(10, 60)
    }

    /// Rate limiter para login: 5 tentativas por 5 minutos
    pub fn login() -> Self {
        Self::new(5, 300)
    }

    async fn check_rate_limit(&self, key: &str) -> Result<(), String> {
        let mut storage = self.storage.lock().await;
        let now = Instant::now();

        let counter = storage.entry(key.to_string()).or_insert(RequestCounter {
            count: 0,
            window_start: now,
        });

        // Verificar se a janela expirou
        if now.duration_since(counter.window_start) >= self.window {
            // Resetar contador
            counter.count = 1;
            counter.window_start = now;
            return Ok(());
        }

        // Incrementar contador
        counter.count += 1;

        if counter.count > self.max_requests {
            let remaining = self.window - now.duration_since(counter.window_start);
            return Err(format!(
                "Rate limit exceeded. Try again in {} seconds",
                remaining.as_secs()
            ));
        }

        Ok(())
    }

    /// Limpar entradas antigas (deve ser executado periodicamente)
    pub async fn cleanup(&self) {
        let mut storage = self.storage.lock().await;
        let now = Instant::now();
        
        storage.retain(|_, counter| {
            now.duration_since(counter.window_start) < self.window * 2
        });
    }
}

impl<S> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service: Rc::new(service),
            limiter: self.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: Rc<S>,
    limiter: RateLimiter,
}

impl<S> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let limiter = self.limiter.clone();

        Box::pin(async move {
            // Obter IP do cliente
            let ip = req
                .peer_addr()
                .map(|addr| addr.ip().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            // Verificar rate limit
            match limiter.check_rate_limit(&ip).await {
                Ok(_) => {
                    // Continuar com a requisição
                    svc.call(req).await
                }
                Err(msg) => {
                    log::warn!("🚨 Rate limit exceeded for IP: {}", ip);
                    
                    let (request, _payload) = req.into_parts();
                    let response = HttpResponse::TooManyRequests()
                        .json(serde_json::json!({
                            "error": "Rate limit exceeded",
                            "message": msg
                        }));
                    
                    Ok(ServiceResponse::new(request, response))
                }
            }
        })
    }
}

/// Rate limiter específico por usuário (baseado em JWT)
#[derive(Clone)]
pub struct UserRateLimiter {
    limiter: RateLimiter,
}

impl UserRateLimiter {
    pub fn new(max_requests: u32, window_secs: u64) -> Self {
        Self {
            limiter: RateLimiter::new(max_requests, window_secs),
        }
    }

    pub fn default() -> Self {
        Self::new(1000, 3600) // 1000 requisições por hora por usuário
    }
}

impl<S> Transform<S, ServiceRequest> for UserRateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = UserRateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(UserRateLimiterMiddleware {
            service: Rc::new(service),
            limiter: self.limiter.clone(),
        }))
    }
}

pub struct UserRateLimiterMiddleware<S> {
    service: Rc<S>,
    limiter: RateLimiter,
}

impl<S> Service<ServiceRequest> for UserRateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let svc = self.service.clone();
        let limiter = self.limiter.clone();

        Box::pin(async move {
            // Usar IP como chave para rate limit por usuário
            let user_key = {
                let ip = req
                    .peer_addr()
                    .map(|addr| addr.ip().to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                format!("ip:{}", ip)
            };

            // Verificar rate limit
            match limiter.check_rate_limit(&user_key).await {
                Ok(_) => svc.call(req).await,
                Err(msg) => {
                    log::warn!("🚨 User rate limit exceeded: {}", user_key);
                    
                    let (request, _payload) = req.into_parts();
                    let response = HttpResponse::TooManyRequests()
                        .json(serde_json::json!({
                            "error": "Rate limit exceeded",
                            "message": msg
                        }));
                    
                    Ok(ServiceResponse::new(request, response))
                }
            }
        })
    }
}
