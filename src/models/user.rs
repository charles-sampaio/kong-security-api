use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use std::time::SystemTime;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub tenant_id: String,
    pub email: String,
    pub password: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_login: Option<DateTime>,

    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub email_verified: bool,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password_reset_expiry: Option<DateTime>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_tokens: Option<Vec<String>>, 
}

impl User {
    pub fn new(tenant_id: String, email: String, password_hash: String) -> Self {
        Self {
            _id: Some(ObjectId::new()),
            tenant_id,
            email,
            password: password_hash,
            roles: Some(vec!["user".to_string()]),
            created_at: Some(DateTime::from_system_time(SystemTime::now())),
            updated_at: Some(DateTime::from_system_time(SystemTime::now())),
            last_login: None,
            is_active: true,
            email_verified: false,
            password_reset_token: None,
            password_reset_expiry: None,
            refresh_tokens: Some(vec![]),
        }
    }

    pub fn is_admin(&self) -> bool {
        self.roles
            .as_ref()
            .map_or(false, |roles| roles.contains(&"admin".to_string()))
    }

    pub fn has_role(&self, role: &str) -> bool {
        self.roles
            .as_ref()
            .map_or(false, |roles| roles.contains(&role.to_string()))
    }

    pub fn add_role(&mut self, role: String) {
        if let Some(roles) = &mut self.roles {
            if !roles.contains(&role) {
                roles.push(role);
            }
        } else {
            self.roles = Some(vec![role]);
        }
    }

    pub fn remove_role(&mut self, role: &str) {
        if let Some(roles) = &mut self.roles {
            roles.retain(|r| r != role);
        }
    }
}