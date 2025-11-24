use crate::models::{Tenant, CreateTenantRequest, UpdateTenantRequest};
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};
use futures_util::TryStreamExt;
use std::time::SystemTime;

pub struct TenantService {
    collection: Collection<Tenant>,
}

impl TenantService {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection::<Tenant>("tenants"),
        }
    }

    /// Verifica se um tenant existe e está ativo
    pub async fn validate_tenant(&self, tenant_id: &str) -> Result<bool, String> {
        match self
            .collection
            .find_one(doc! { "tenant_id": tenant_id })
            .await
        {
            Ok(Some(tenant)) => {
                if tenant.active {
                    Ok(true)
                } else {
                    Err("Tenant is inactive".to_string())
                }
            }
            Ok(None) => Err("Tenant not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Busca um tenant pelo tenant_id
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<Option<Tenant>, String> {
        self.collection
            .find_one(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Database error: {}", e))
    }

    /// Cria um novo tenant
    pub async fn create_tenant(&self, request: CreateTenantRequest) -> Result<Tenant, String> {
        // Verificar se já existe tenant com o mesmo document (CNPJ/CPF único)
        let existing_document = self
            .collection
            .find_one(doc! {
                "document": &request.document
            })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if existing_document.is_some() {
            return Err("Tenant with this document already exists".to_string());
        }

        // Verificar se já existe tenant com o mesmo nome e document (redundante, mas mantém compatibilidade)
        let existing_name_doc = self
            .collection
            .find_one(doc! {
                "name": &request.name,
                "document": &request.document
            })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if existing_name_doc.is_some() {
            return Err("Tenant with this name and document already exists".to_string());
        }

        let tenant = Tenant::new(request.name, request.document, request.description);

        self.collection
            .insert_one(&tenant)
            .await
            .map_err(|e| format!("Failed to create tenant: {}", e))?;

        Ok(tenant)
    }

    /// Lista todos os tenants
    pub async fn list_tenants(&self, active_only: bool) -> Result<Vec<Tenant>, String> {
        let filter = if active_only {
            doc! { "active": true }
        } else {
            doc! {}
        };

        let cursor = self
            .collection
            .find(filter)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect tenants: {}", e))
    }

    /// Atualiza um tenant
    pub async fn update_tenant(
        &self,
        tenant_id: &str,
        request: UpdateTenantRequest,
    ) -> Result<Tenant, String> {
        let mut update_doc = doc! {};

        if let Some(name) = &request.name {
            update_doc.insert("name", name);
        }

        if let Some(document) = &request.document {
            update_doc.insert("document", document);
        }

        if let Some(description) = request.description {
            update_doc.insert("description", description);
        }

        if let Some(active) = request.active {
            update_doc.insert("active", active);
        }

        update_doc.insert("updated_at", DateTime::from_system_time(SystemTime::now()));

        if update_doc.is_empty() {
            return Err("No fields to update".to_string());
        }

        // Se name ou document estão sendo atualizados, verificar duplicidade
        if request.document.is_some() {
            let current_tenant = self
                .get_tenant(tenant_id)
                .await?
                .ok_or("Tenant not found".to_string())?;

            let check_document = request.document.as_ref().unwrap_or(&current_tenant.document);

            // Verificar se o document já existe em outro tenant (document deve ser único)
            let existing_document = self
                .collection
                .find_one(doc! {
                    "document": check_document,
                    "tenant_id": { "$ne": tenant_id }
                })
                .await
                .map_err(|e| format!("Database error: {}", e))?;

            if existing_document.is_some() {
                return Err("Tenant with this document already exists".to_string());
            }

            // Também verificar combinação name + document
            let check_name = request.name.as_ref().unwrap_or(&current_tenant.name);

            let existing_name_doc = self
                .collection
                .find_one(doc! {
                    "name": check_name,
                    "document": check_document,
                    "tenant_id": { "$ne": tenant_id }
                })
                .await
                .map_err(|e| format!("Database error: {}", e))?;

            if existing_name_doc.is_some() {
                return Err("Tenant with this name and document already exists".to_string());
            }
        }

        let result = self
            .collection
            .find_one_and_update(
                doc! { "tenant_id": tenant_id },
                doc! { "$set": update_doc },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        match result {
            Some(_) => self
                .get_tenant(tenant_id)
                .await?
                .ok_or_else(|| "Failed to retrieve updated tenant".to_string()),
            None => Err("Tenant not found".to_string()),
        }
    }

    /// Desativa um tenant (soft delete)
    pub async fn deactivate_tenant(&self, tenant_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .update_one(
                doc! { "tenant_id": tenant_id },
                doc! {
                    "$set": {
                        "active": false,
                        "updated_at": DateTime::from_system_time(SystemTime::now())
                    }
                },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.matched_count == 0 {
            Err("Tenant not found".to_string())
        } else {
            Ok(())
        }
    }

    /// Ativa um tenant
    pub async fn activate_tenant(&self, tenant_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .update_one(
                doc! { "tenant_id": tenant_id },
                doc! {
                    "$set": {
                        "active": true,
                        "updated_at": DateTime::from_system_time(SystemTime::now())
                    }
                },
            )
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.matched_count == 0 {
            Err("Tenant not found".to_string())
        } else {
            Ok(())
        }
    }

    /// Deleta permanentemente um tenant (use com cuidado!)
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .delete_one(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.deleted_count == 0 {
            Err("Tenant not found".to_string())
        } else {
            Ok(())
        }
    }
}
