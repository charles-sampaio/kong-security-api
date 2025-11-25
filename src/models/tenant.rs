use serde::{Serialize, Deserialize};
use mongodb::bson::{oid::ObjectId, DateTime};
use std::time::SystemTime;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, ToSchema)]
pub struct Tenant {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    
    #[schema(example = "550e8400-e29b-41d4-a716-446655440000")]
    pub tenant_id: String,
    
    #[schema(example = "ACME Corporation")]
    pub name: String,
    
    #[schema(example = "12345678000190")]
    pub document: String, // CNPJ or CPF
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Main tenant for ACME Corporation")]
    pub description: Option<String>,
    
    #[serde(default = "default_active")]
    #[schema(example = true)]
    pub active: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime>,
}

fn default_active() -> bool {
    true
}

impl Tenant {
    pub fn new(name: String, document: String, description: Option<String>) -> Self {
        Self {
            id: Some(ObjectId::new()),
            tenant_id: Uuid::new_v4().to_string(), // Gera UUID automaticamente
            name,
            document,
            description,
            active: true,
            created_at: Some(DateTime::from_system_time(SystemTime::now())),
            updated_at: Some(DateTime::from_system_time(SystemTime::now())),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    #[schema(example = "ACME Corporation")]
    pub name: String,
    
    #[schema(example = "12345678000190")]
    pub document: String, // CNPJ or CPF
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Main tenant for ACME Corporation")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "ACME Corporation Updated")]
    pub name: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "12345678000190")]
    pub document: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = "Updated description")]
    pub description: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    #[schema(example = true)]
    pub active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TenantResponse {
    pub tenant_id: String,
    pub name: String,
    pub document: String,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    
    pub active: bool,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<String>,
    
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

impl From<Tenant> for TenantResponse {
    fn from(tenant: Tenant) -> Self {
        Self {
            tenant_id: tenant.tenant_id,
            name: tenant.name,
            document: tenant.document,
            description: tenant.description,
            active: tenant.active,
            created_at: tenant.created_at.map(|dt| dt.to_string()),
            updated_at: tenant.updated_at.map(|dt| dt.to_string()),
        }
    }
}
