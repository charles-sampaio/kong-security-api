use mongodb::{Database, Collection, bson::doc};
use futures_util::stream::StreamExt;
use std::error::Error;
use crate::models::{LoginLog, LoginStats};

pub struct LogService {
    db: Database,
}

impl LogService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn logs_collection(&self) -> Collection<LoginLog> {
        self.db.collection("logs")
    }

    pub async fn save_login_log(&self, login_log: &LoginLog) -> Result<mongodb::bson::oid::ObjectId, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        
        match collection.insert_one(login_log).await {
            Ok(result) => {
                if let Some(id) = result.inserted_id.as_object_id() {
                    println!("✅ Login log saved successfully for email: {}", login_log.email);
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

    pub async fn get_user_logs(&self, user_id: &str, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! { "user_id": user_id };

        let mut cursor = collection.find(filter).await?;
        let mut logs = Vec::new();
        let mut count = 0;

        while let Some(log) = cursor.next().await {
            match log {
                Ok(l) => {
                    logs.push(l);
                    count += 1;
                    if let Some(limit_val) = limit {
                        if count >= limit_val {
                            break;
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        // Sort by timestamp descending (most recent first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

    pub async fn get_user_logs_by_tenant(&self, user_id: &str, tenant_id: &str, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! { 
            "user_id": user_id,
            "tenant_id": tenant_id 
        };

        let mut cursor = collection.find(filter).await?;
        let mut logs = Vec::new();
        let mut count = 0;

        while let Some(log) = cursor.next().await {
            match log {
                Ok(l) => {
                    logs.push(l);
                    count += 1;
                    if let Some(limit_val) = limit {
                        if count >= limit_val {
                            break;
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        // Sort by timestamp descending (most recent first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

    pub async fn get_all_logs(&self, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! {};

        let mut cursor = collection.find(filter).await?;
        let mut logs = Vec::new();
        let mut count = 0;

        while let Some(log) = cursor.next().await {
            match log {
                Ok(l) => {
                    logs.push(l);
                    count += 1;
                    if let Some(limit_val) = limit {
                        if count >= limit_val {
                            break;
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        // Sort by timestamp descending (most recent first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }

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

    pub async fn get_logs_by_email(&self, email: &str, limit: Option<i64>) -> Result<Vec<LoginLog>, Box<dyn Error + Send + Sync>> {
        let collection = self.logs_collection();
        let filter = doc! { "email": email };

        let mut cursor = collection.find(filter).await?;
        let mut logs = Vec::new();
        let mut count = 0;

        while let Some(log) = cursor.next().await {
            match log {
                Ok(l) => {
                    logs.push(l);
                    count += 1;
                    if let Some(limit_val) = limit {
                        if count >= limit_val {
                            break;
                        }
                    }
                }
                Err(e) => return Err(Box::new(e)),
            }
        }

        // Sort by timestamp descending (most recent first)
        logs.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(logs)
    }
}