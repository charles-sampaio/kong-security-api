use crate::models::{Tenant, CreateTenantRequest, UpdateTenantRequest};
use crate::cache::{RedisCache, cache_keys};
use mongodb::{
    bson::{doc, DateTime},
    Collection, Database,
};
use futures_util::TryStreamExt;
use std::time::SystemTime;

pub struct TenantService {
    collection: Collection<Tenant>,
    pub cache: Option<RedisCache>,
}

impl TenantService {
    pub fn new(db: &Database, cache: Option<RedisCache>) -> Self {
        Self {
            collection: db.collection::<Tenant>("tenants"),
            cache,
        }
    }

    /// Invalida cache de tenant específico e listas
    async fn invalidate_tenant_cache(&self, tenant_id: &str) {
        if let Some(cache) = &self.cache {
            // Invalidar tenant específico
            let _ = cache.delete(&cache_keys::tenant_by_id(tenant_id)).await;
            // Invalidar lista de tenants ativos
            let _ = cache.delete(cache_keys::TENANTS_LIST).await;
            // Invalidar lista de TODOS os tenants
            let _ = cache.delete(cache_keys::TENANTS_LIST_ALL).await;
        }
    }

    /// Verifica se um tenant existe e está ativo (com cache)
    /// Retorna (is_valid, from_cache)
    pub async fn validate_tenant(&self, tenant_id: &str) -> Result<(bool, bool), String> {
        // Tentar buscar do cache primeiro
        if let Some(cache) = &self.cache {
            let cache_key = cache_keys::tenant_by_id(tenant_id);
            if let Ok(Some(tenant)) = cache.get::<Tenant>(&cache_key).await {
                return if tenant.active {
                    Ok((true, true))
                } else {
                    Err("Tenant is inactive".to_string())
                };
            }
        }

        // Buscar do banco de dados
        match self
            .collection
            .find_one(doc! { "tenant_id": tenant_id })
            .await
        {
            Ok(Some(tenant)) => {
                // Cachear o resultado (TTL de 5 minutos)
                if let Some(cache) = &self.cache {
                    let cache_key = cache_keys::tenant_by_id(tenant_id);
                    let _ = cache.set_with_ttl(&cache_key, &tenant, 300).await;
                }

                if tenant.active {
                    Ok((true, false))
                } else {
                    Err("Tenant is inactive".to_string())
                }
            }
            Ok(None) => Err("Tenant not found".to_string()),
            Err(e) => Err(format!("Database error: {}", e)),
        }
    }

    /// Busca um tenant pelo tenant_id (com cache)
    /// Retorna (Option<Tenant>, from_cache)
    pub async fn get_tenant(&self, tenant_id: &str) -> Result<(Option<Tenant>, bool), String> {
        // Tentar buscar do cache primeiro
        if let Some(cache) = &self.cache {
            let cache_key = cache_keys::tenant_by_id(tenant_id);
            if let Ok(Some(tenant)) = cache.get::<Tenant>(&cache_key).await {
                return Ok((Some(tenant), true));
            }
        }

        // Buscar do banco de dados
        let tenant = self.collection
            .find_one(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        // Cachear se encontrado
        if let (Some(cache), Some(ref t)) = (&self.cache, &tenant) {
            let cache_key = cache_keys::tenant_by_id(tenant_id);
            let _ = cache.set_with_ttl(&cache_key, t, 300).await;
        }

        Ok((tenant, false))
    }

    /// Cria um novo tenant (invalida cache)
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

        // Invalidar cache de lista de tenants
        self.invalidate_tenant_cache(&tenant.tenant_id).await;

        Ok(tenant)
    }

    /// Lista todos os tenants (com cache para todas as listas)
    /// Retorna (Vec<Tenant>, from_cache)
    pub async fn list_tenants(&self, active_only: bool) -> Result<(Vec<Tenant>, bool), String> {
        // Determinar chave de cache baseado no filtro
        let cache_key = if active_only {
            cache_keys::TENANTS_LIST
        } else {
            cache_keys::TENANTS_LIST_ALL
        };

        // Tentar buscar do cache primeiro
        if let Some(cache) = &self.cache {
            if let Ok(Some(tenants)) = cache.get::<Vec<Tenant>>(cache_key).await {
                return Ok((tenants, true));
            }
        }

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

        let tenants: Vec<Tenant> = cursor
            .try_collect()
            .await
            .map_err(|e| format!("Failed to collect tenants: {}", e))?;

        // Cachear o resultado (TTL de 5 minutos)
        if let Some(cache) = &self.cache {
            let _ = cache.set_with_ttl(cache_key, &tenants, 300).await;
        }

        Ok((tenants, false))
    }

    /// Atualiza um tenant (invalida cache)
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
            let (current_tenant_opt, _) = self
                .get_tenant(tenant_id)
                .await?;
            
            let current_tenant = current_tenant_opt.ok_or("Tenant not found".to_string())?;

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

        // Invalidar cache
        self.invalidate_tenant_cache(tenant_id).await;

        match result {
            Some(_) => {
                let (tenant_opt, _) = self
                    .get_tenant(tenant_id)
                    .await?;
                tenant_opt.ok_or_else(|| "Failed to retrieve updated tenant".to_string())
            }
            None => Err("Tenant not found".to_string()),
        }
    }

    /// Desativa um tenant (soft delete) - invalida cache
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
            // Invalidar cache
            self.invalidate_tenant_cache(tenant_id).await;
            Ok(())
        }
    }

    /// Ativa um tenant - invalida cache
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
            // Invalidar cache
            self.invalidate_tenant_cache(tenant_id).await;
            Ok(())
        }
    }

    /// Deleta permanentemente um tenant (use com cuidado!) - invalida cache
    pub async fn delete_tenant(&self, tenant_id: &str) -> Result<(), String> {
        let result = self
            .collection
            .delete_one(doc! { "tenant_id": tenant_id })
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        if result.deleted_count == 0 {
            Err("Tenant not found".to_string())
        } else {
            // Invalidar cache
            self.invalidate_tenant_cache(tenant_id).await;
            Ok(())
        }
    }
}
