use mongodb::{Database, Collection, bson::doc, options::FindOptions};
use futures_util::TryStreamExt;
use std::error::Error;
use crate::models::{LoginLog, LoginStats};
use crate::cache::{RedisCache, cache_keys};

pub struct LogService {
    db: Database,
    cache: Option<RedisCache>,
}

impl LogService {
    pub fn new(db: Database, cache: Option<RedisCache>) -> Self {
        Self { db, cache }
    }

    pub fn logs_collection(&self) -> Collection<LoginLog> {
        self.db.collection("logs")
    }

    /// Salvar log e invalidar cache
    pub async fn save_login_log(&self, login_log: &LoginLog) -> Result<mongodb::bson::oid::ObjectId, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        
        match collection.insert_one(login_log).await {
            Ok(result) => {
                if let Some(id) = result.inserted_id.as_object_id() {
                    println!("✅ Login log saved successfully for email: {}", login_log.email);
                    
                    // Invalidar cache de logs do tenant
                    if let Some(cache) = &self.cache {
                        let pattern = cache_keys::logs_pattern(&login_log.tenant_id);
                        let _ = cache.delete_pattern(&pattern).await;
                    }
                    
                    Ok(id)
                } else {
                    Err("Failed to get inserted ID".into())
                }
            },
            Err(e) => {
                eprintln!("❌ Error saving login log: {}", e);
                Err(Box::new(e))
            }
        }
    }

    /// Buscar logs de usuário (otimizado com sort e limit no MongoDB)
    pub async fn get_user_logs(&self, user_id: &str, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! { "user_id": user_id };

        let find_options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let cursor = collection.find(filter).with_options(find_options).await?;
        let logs: Vec<LoginLog> = cursor.try_collect().await?;

        Ok(logs)
    }

    /// Buscar logs de usuário por tenant (com cache)
    /// Retorna (Vec<LoginLog>, from_cache)
    pub async fn get_user_logs_by_tenant(&self, user_id: &str, tenant_id: &str, limit: Option<i64>) -> Result<(Vec<LoginLog>, bool), Box<dyn Error + Send + Sync>> {
        // Define limite padrão de 50 se não especificado
        let effective_limit = limit.unwrap_or(50);
        
        // Cachear primeira página com até 50 registros
        let should_cache = effective_limit <= 50;
        
        // Tentar buscar do cache
        if should_cache {
            if let Some(cache) = &self.cache {
                let cache_key = cache_keys::user_tenant_logs(user_id, tenant_id);
                if let Ok(Some(logs)) = cache.get::<Vec<LoginLog>>(&cache_key).await {
                    return Ok((logs, true));
                }
            }
        }

        let collection = self.logs_collection();
        let filter = doc! { 
            "user_id": user_id,
            "tenant_id": tenant_id 
        };

        let find_options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(Some(effective_limit))
            .build();

        let cursor = collection.find(filter).with_options(find_options).await?;
        let logs: Vec<LoginLog> = cursor.try_collect().await?;

        // Cachear primeira página (TTL de 2 minutos - logs mudam frequentemente)
        if should_cache {
            if let Some(cache) = &self.cache {
                let cache_key = cache_keys::user_tenant_logs(user_id, tenant_id);
                let _ = cache.set_with_ttl(&cache_key, &logs, 120).await;
            }
        }

        Ok((logs, false))
    }

    /// Buscar todos os logs (otimizado)
    pub async fn get_all_logs(&self, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! {};

        let find_options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let cursor = collection.find(filter).with_options(find_options).await?;
        let logs: Vec<LoginLog> = cursor.try_collect().await?;

        Ok(logs)
    }

    /// Buscar estatísticas de login (sem cache - dados em tempo real)
    pub async fn get_login_stats(&self, days: i32) -> Result<LoginStats, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        
        // Calculate date threshold
        let date_threshold = chrono::Utc::now() - chrono::Duration::days(days as i64);
        let system_time = date_threshold.timestamp_millis();
        let date_filter = mongodb::bson::DateTime::from_millis(system_time);
        
        // Count total attempts
        let total_filter = doc! { "timestamp": { "$gte": date_filter } };
        let total_attempts = collection.count_documents(total_filter.clone()).await?;
        
        // Count successful logins
        let success_filter = doc! { 
            "timestamp": { "$gte": date_filter },
            "success": true 
        };
        let successful_logins = collection.count_documents(success_filter).await?;
        
        // Count failed logins
        let failed_logins = total_attempts - successful_logins;
        
        // Calculate success rate
        let success_rate = if total_attempts > 0 {
            (successful_logins as f64 / total_attempts as f64) * 100.0
        } else {
            0.0
        };

        Ok(LoginStats {
            total_attempts,
            successful_logins,
            failed_logins,
            success_rate,
            period_days: days,
        })
    }

    /// Buscar logs por email (otimizado)
    pub async fn get_logs_by_email(&self, email: &str, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! { "email": email };

        let find_options = FindOptions::builder()
            .sort(doc! { "timestamp": -1 })
            .limit(limit)
            .build();

        let cursor = collection.find(filter).with_options(find_options).await?;
        let logs: Vec<LoginLog> = cursor.try_collect().await?;

        Ok(logs)
    }
}