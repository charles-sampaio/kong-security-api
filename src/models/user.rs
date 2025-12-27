use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use std::time::SystemTime;

/// Provider de autenticação OAuth
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Apple,
}

impl OAuthProvider {
    pub fn as_str(&self) -> &str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Apple => "apple",
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub _id: Option<ObjectId>,
    pub tenant_id: String,
    pub email: String,
    
    // Password é opcional agora (OAuth não precisa)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    // Campos OAuth
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_provider: Option<OAuthProvider>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub oauth_id: Option<String>, // ID do usuário no provider (Google ID, Apple ID)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>, // URL da foto de perfil

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
    /// Criar usuário tradicional com senha (DEPRECATED - usar OAuth)
    pub fn new(tenant_id: String, email: String, password_hash: String) -> Self {
        Self {
            _id: Some(ObjectId::new()),
            tenant_id,
            email,
            password: Some(password_hash),
            oauth_provider: None,
            oauth_id: None,
            name: None,
            picture: None,
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

    /// Criar usuário via OAuth (Google/Apple)
    pub fn from_oauth(
        tenant_id: String,
        email: String,
        oauth_provider: OAuthProvider,
        oauth_id: String,
        name: Option<String>,
        picture: Option<String>,
    ) -> Self {
        Self {
            _id: Some(ObjectId::new()),
            tenant_id,
            email,
            password: None, // OAuth não usa senha
            oauth_provider: Some(oauth_provider),
            oauth_id: Some(oauth_id),
            name,
            picture,
            roles: Some(vec!["user".to_string()]),
            created_at: Some(DateTime::from_system_time(SystemTime::now())),
            updated_at: Some(DateTime::from_system_time(SystemTime::now())),
            last_login: None,
            is_active: true,
            email_verified: true, // OAuth já verificou o email
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