use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use std::time::SystemTime;
use chrono;
use crate::utils::user_agent_parser::UserAgentInfo;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginLog {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    
    // User information
    pub user_id: Option<String>, // Some if login successful, None if failed
    pub email: String,
    
    // Login attempt details
    pub success: bool,
    pub failure_reason: Option<String>,
    
    // Request information
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_method: String,
    pub request_path: String,
    
    // Timestamps
    pub timestamp: DateTime,
    pub login_date: String, // Format: YYYY-MM-DD for easier querying
    pub login_time: String, // Format: HH:MM:SS
    
    // Security information
    pub token_generated: bool,
    pub refresh_token_generated: bool,
    
    // Additional metadata
    pub session_id: Option<String>,
    pub device_type: Option<String>, // Mobile, Desktop, Tablet, etc.
    pub browser: Option<String>,
    pub os: Option<String>,
    pub country: Option<String>,
    pub city: Option<String>,
}

impl LoginLog {
    pub fn new(
        email: String,
        success: bool,
        ip_address: Option<String>,
        user_agent: Option<String>,
    ) -> Self {
        let now = SystemTime::now();
        let datetime = DateTime::from_system_time(now);
        let utc_now = chrono::Utc::now();
        
        let ua_info = UserAgentInfo::parse(&user_agent);
        
        Self {
            _id: Some(ObjectId::new()),
            user_id: None,
            email,
            success,
            failure_reason: None,
            ip_address,
            user_agent: user_agent.clone(),
            request_method: "POST".to_string(),
            request_path: "/login".to_string(),
            timestamp: datetime,
            login_date: utc_now.format("%Y-%m-%d").to_string(),
            login_time: utc_now.format("%H:%M:%S").to_string(),
            token_generated: false,
            refresh_token_generated: false,
            session_id: Some(uuid::Uuid::new_v4().to_string()),
            device_type: ua_info.device_type,
            browser: ua_info.browser,
            os: ua_info.os,
            country: None,
            city: None,
        }
    }
    
    pub fn set_success(&mut self, user_id: String, token_generated: bool, refresh_token_generated: bool) {
        self.user_id = Some(user_id);
        self.success = true;
        self.token_generated = token_generated;
        self.refresh_token_generated = refresh_token_generated;
        self.failure_reason = None;
    }
    
    pub fn set_failure(&mut self, reason: String) {
        self.success = false;
        self.failure_reason = Some(reason);
        self.token_generated = false;
        self.refresh_token_generated = false;
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginStats {
    pub total_attempts: u64,
    pub successful_logins: u64,
    pub failed_logins: u64,
    pub success_rate: f64, // Percentage
    pub period_days: i32,
}