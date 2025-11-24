use mongodb::{
    bson::doc,
    options::IndexOptions,
    IndexModel,
    Database,
};

/// Inicializa todos os índices únicos do banco de dados
pub async fn initialize_indexes(db: &Database) -> Result<(), String> {
    // Migrar dados existentes primeiro
    migrate_existing_data(db).await?;
    
    // Índices para a collection 'tenants'
    create_tenant_indexes(db).await?;
    
    // Índices para a collection 'users'
    create_user_indexes(db).await?;
    
    println!("✅ Database indexes initialized successfully");
    Ok(())
}

/// Migra dados existentes antes de criar índices
async fn migrate_existing_data(db: &Database) -> Result<(), String> {
    let tenants_collection = db.collection::<mongodb::bson::Document>("tenants");
    
    // Adicionar campo document aos tenants que não o possuem
    let result = tenants_collection
        .update_many(
            doc! { "document": { "$exists": false } },
            doc! { "$set": { "document": "00000000000000" } }, // Placeholder
        )
        .await
        .map_err(|e| format!("Failed to migrate tenants: {}", e))?;
    
    if result.modified_count > 0 {
        println!("  ⚠️  Migrated {} tenants with placeholder document", result.modified_count);
        println!("  ℹ️  Please update these tenants with real CNPJ/CPF values");
    }
    
    Ok(())
}

/// Cria índices para a collection 'tenants'
async fn create_tenant_indexes(db: &Database) -> Result<(), String> {
    let collection = db.collection::<mongodb::bson::Document>("tenants");
    
    // Índice único para tenant_id (UUID)
    let tenant_id_index = IndexOptions::builder()
        .unique(true)
        .name("tenant_id_unique".to_string())
        .build();
    
    let tenant_id_model = IndexModel::builder()
        .keys(doc! { "tenant_id": 1 })
        .options(tenant_id_index)
        .build();
    
    // Índice único para document (CNPJ/CPF deve ser único no sistema)
    let document_index = IndexOptions::builder()
        .unique(true)
        .name("document_unique".to_string())
        .build();
    
    let document_model = IndexModel::builder()
        .keys(doc! { "document": 1 })
        .options(document_index)
        .build();
    
    // Índice único composto para name + document (evita duplicação de CNPJ/CPF com mesmo nome)
    let name_document_index = IndexOptions::builder()
        .unique(true)
        .name("name_document_unique".to_string())
        .build();
    
    let name_document_model = IndexModel::builder()
        .keys(doc! { "name": 1, "document": 1 })
        .options(name_document_index)
        .build();
    
    // Índice para busca por active status
    let active_index_model = IndexModel::builder()
        .keys(doc! { "active": 1 })
        .build();
    
    collection
        .create_indexes(vec![
            tenant_id_model,
            document_model,
            name_document_model,
            active_index_model,
        ])
        .await
        .map_err(|e| format!("Failed to create tenant indexes: {}", e))?;
    
    println!("  ✓ Tenant indexes created");
    Ok(())
}

/// Cria índices para a collection 'users'
async fn create_user_indexes(db: &Database) -> Result<(), String> {
    let collection = db.collection::<mongodb::bson::Document>("users");
    
    // Índice único composto para email + tenant_id
    // Permite que o mesmo email exista em tenants diferentes
    let email_tenant_index = IndexOptions::builder()
        .unique(true)
        .name("email_tenant_unique".to_string())
        .build();
    
    let email_tenant_model = IndexModel::builder()
        .keys(doc! { "email": 1, "tenant_id": 1 })
        .options(email_tenant_index)
        .build();
    
    // Índice para busca por tenant_id
    let tenant_index_model = IndexModel::builder()
        .keys(doc! { "tenant_id": 1 })
        .build();
    
    // Índice para timestamps (útil para ordenação)
    let created_at_index_model = IndexModel::builder()
        .keys(doc! { "created_at": -1 })
        .build();
    
    collection
        .create_indexes(vec![
            email_tenant_model,
            tenant_index_model,
            created_at_index_model,
        ])
        .await
        .map_err(|e| format!("Failed to create user indexes: {}", e))?;
    
    println!("  ✓ User indexes created");
    Ok(())
}

/// Cria índices para a collection 'logs' (login logs)
pub async fn create_log_indexes(db: &Database) -> Result<(), String> {
    let collection = db.collection::<mongodb::bson::Document>("logs");
    
    // Índice composto para busca por tenant_id + created_at
    let tenant_date_index_model = IndexModel::builder()
        .keys(doc! { "tenant_id": 1, "created_at": -1 })
        .build();
    
    // Índice para busca por email + tenant_id
    let email_tenant_index_model = IndexModel::builder()
        .keys(doc! { "email": 1, "tenant_id": 1, "created_at": -1 })
        .build();
    
    // Índice para busca por user_id
    let user_id_index_model = IndexModel::builder()
        .keys(doc! { "user_id": 1, "created_at": -1 })
        .build();
    
    collection
        .create_indexes(vec![
            tenant_date_index_model,
            email_tenant_index_model,
            user_id_index_model,
        ])
        .await
        .map_err(|e| format!("Failed to create log indexes: {}", e))?;
    
    println!("  ✓ Log indexes created");
    Ok(())
}

/// Cria índices para a collection 'password_reset_tokens'
pub async fn create_password_reset_indexes(db: &Database) -> Result<(), String> {
    let collection = db.collection::<mongodb::bson::Document>("password_reset_tokens");
    
    // Índice composto para busca por email + tenant_id + token
    let email_tenant_token_index = IndexOptions::builder()
        .unique(false)
        .build();
    
    let email_tenant_token_model = IndexModel::builder()
        .keys(doc! { "email": 1, "tenant_id": 1, "token": 1 })
        .options(email_tenant_token_index)
        .build();
    
    // Índice com TTL (Time To Live) para expirar documentos automaticamente
    let expires_at_index = IndexOptions::builder()
        .expire_after(std::time::Duration::from_secs(0))
        .name("expires_at_ttl".to_string())
        .build();
    
    let expires_at_model = IndexModel::builder()
        .keys(doc! { "expires_at": 1 })
        .options(expires_at_index)
        .build();
    
    collection
        .create_indexes(vec![
            email_tenant_token_model,
            expires_at_model,
        ])
        .await
        .map_err(|e| format!("Failed to create password reset indexes: {}", e))?;
    
    println!("  ✓ Password reset token indexes created");
    Ok(())
}
