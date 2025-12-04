use deadpool_redis::{Config, Pool, Runtime};
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

/// Configuração do cache Redis
#[derive(Clone)]
pub struct CacheConfig {
    pub redis_url: String,
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub default_ttl: u64, // TTL padrão em segundos
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string()),
            max_connections: 10, // Pool pequeno para economizar recursos
            connection_timeout: Duration::from_secs(5),
            default_ttl: 300, // 5 minutos
        }
    }
}

/// Cliente Redis otimizado para performance
#[derive(Clone)]
pub struct RedisCache {
    pool: Pool,
    default_ttl: u64,
}

impl RedisCache {
    /// Criar novo cliente Redis com pool de conexões otimizado
    pub fn new(config: CacheConfig) -> Result<Self, String> {
        let cfg = Config::from_url(&config.redis_url);
        let pool = cfg.create_pool(Some(Runtime::Tokio1))
            .map_err(|e| format!("Failed to create Redis pool: {}", e))?;

        Ok(Self {
            pool,
            default_ttl: config.default_ttl,
        })
    }

    /// Obter valor do cache (deserializado)
    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Result<Option<T>, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let value: Option<String> = conn
            .get(key)
            .await
            .map_err(|e| format!("Redis GET error: {}", e))?;

        match value {
            Some(json) => {
                let data = serde_json::from_str(&json)
                    .map_err(|e| format!("JSON deserialization error: {}", e))?;
                Ok(Some(data))
            }
            None => Ok(None),
        }
    }

    /// Definir valor no cache com TTL customizado
    pub async fn set_with_ttl<T: Serialize>(
        &self,
        key: &str,
        value: &T,
        ttl_seconds: u64,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let json = serde_json::to_string(value)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        conn.set_ex::<_, _, ()>(key, json, ttl_seconds)
            .await
            .map_err(|e| format!("Redis SET error: {}", e))?;

        Ok(())
    }

    /// Definir valor no cache com TTL padrão
    pub async fn set<T: Serialize>(&self, key: &str, value: &T) -> Result<(), String> {
        self.set_with_ttl(key, value, self.default_ttl).await
    }

    /// Deletar chave do cache
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        conn.del::<_, ()>(key)
            .await
            .map_err(|e| format!("Redis DEL error: {}", e))?;

        Ok(())
    }

    /// Deletar múltiplas chaves (invalidação em lote)
    pub async fn delete_pattern(&self, pattern: &str) -> Result<u64, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        // Buscar todas as chaves que correspondem ao padrão
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| format!("Redis KEYS error: {}", e))?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Deletar todas as chaves encontradas
        let count: u64 = conn
            .del(&keys)
            .await
            .map_err(|e| format!("Redis DEL error: {}", e))?;

        Ok(count)
    }

    /// Verificar se chave existe
    pub async fn exists(&self, key: &str) -> Result<bool, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let exists: bool = conn
            .exists(key)
            .await
            .map_err(|e| format!("Redis EXISTS error: {}", e))?;

        Ok(exists)
    }

    /// Incrementar contador (útil para rate limiting)
    pub async fn increment_with_ttl(&self, key: &str, ttl_seconds: u64) -> Result<i64, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let count: i64 = conn
            .incr(key, 1)
            .await
            .map_err(|e| format!("Redis INCR error: {}", e))?;

        // Definir TTL se for a primeira vez
        if count == 1 {
            let _: () = conn.expire(key, ttl_seconds as i64)
                .await
                .map_err(|e| format!("Redis EXPIRE error: {}", e))?;
        }

        Ok(count)
    }

    /// Obter TTL restante de uma chave
    pub async fn ttl(&self, key: &str) -> Result<i64, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let ttl: i64 = conn
            .ttl(key)
            .await
            .map_err(|e| format!("Redis TTL error: {}", e))?;

        Ok(ttl)
    }

    /// Health check do Redis
    pub async fn ping(&self) -> Result<String, String> {
        let mut conn = self.pool.get().await.map_err(|e| e.to_string())?;
        
        let response: String = redis::cmd("PING")
            .query_async(&mut *conn)
            .await
            .map_err(|e| format!("Redis PING error: {}", e))?;

        Ok(response)
    }

    /// Obter estatísticas do pool de conexões
    pub fn pool_status(&self) -> (usize, usize) {
        let status = self.pool.status();
        (status.size, status.available)
    }
}

// Chaves de cache padronizadas
pub mod cache_keys {
    /// Lista de todos os tenants ativos
    pub const TENANTS_LIST: &str = "tenants:list:active";
    
    /// Lista de TODOS os tenants (ativos e inativos)
    pub const TENANTS_LIST_ALL: &str = "tenants:list:all";
    
    /// Tenant específico por ID
    pub fn tenant_by_id(tenant_id: &str) -> String {
        format!("tenant:{}", tenant_id)
    }

    /// Logs recentes de um tenant
    pub fn tenant_logs(tenant_id: &str, page: u32) -> String {
        format!("logs:{}:page:{}", tenant_id, page)
    }

    /// Logs de usuário específico por tenant
    pub fn user_tenant_logs(user_id: &str, tenant_id: &str) -> String {
        format!("logs:user:{}:tenant:{}", user_id, tenant_id)
    }

    /// Padrão para invalidar todos os caches de tenant
    pub fn tenant_pattern(tenant_id: &str) -> String {
        format!("tenant:{}*", tenant_id)
    }

    /// Padrão para invalidar cache de logs de tenant
    pub fn logs_pattern(tenant_id: &str) -> String {
        format!("logs:*tenant:{}", tenant_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_keys() {
        assert_eq!(cache_keys::tenant_by_id("123"), "tenant:123");
        assert_eq!(cache_keys::tenant_logs("123", 1), "logs:123:page:1");
        assert_eq!(cache_keys::tenant_pattern("123"), "tenant:123*");
    }
}
