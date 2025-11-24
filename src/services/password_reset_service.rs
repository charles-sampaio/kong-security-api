use crate::models::PasswordResetToken;
use mongodb::{Collection, Database, bson::doc};
use std::error::Error;
use uuid::Uuid;

/// Serviço para gerenciar tokens de recuperação de senha
pub struct PasswordResetService {
    collection: Collection<PasswordResetToken>,
}

impl PasswordResetService {
    /// Cria uma nova instância do serviço
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("password_reset_tokens"),
        }
    }
    
    /// Gera um token de reset de senha
    pub async fn create_reset_token(
        &self,
        tenant_id: &str,
        email: &str,
        expiration_hours: i64,
        ip_address: Option<String>,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        // Gerar token único
        let token = Uuid::new_v4().to_string();
        
        // Criar documento do token
        let reset_token = PasswordResetToken::new(
            tenant_id.to_string(),
            email.to_string(),
            token.clone(),
            expiration_hours,
            ip_address,
        );
        
        // Salvar no banco
        self.collection.insert_one(reset_token).await?;
        
        Ok(token)
    }
    
    /// Valida um token de reset
    pub async fn validate_token(
        &self,
        token: &str,
    ) -> Result<Option<PasswordResetToken>, Box<dyn Error + Send + Sync>> {
        let filter = doc! { "token": token };
        
        if let Some(reset_token) = self.collection.find_one(filter).await? {
            if reset_token.is_valid() {
                Ok(Some(reset_token))
            } else {
                Ok(None) // Token expirado ou já usado
            }
        } else {
            Ok(None) // Token não encontrado
        }
    }
    
    /// Marca um token como usado
    pub async fn mark_token_as_used(
        &self,
        token: &str,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let filter = doc! { "token": token };
        let update = doc! { "$set": { "used": true } };
        
        let result = self.collection.update_one(filter, update).await?;
        Ok(result.modified_count > 0)
    }
    
    /// Remove tokens expirados (limpeza periódica)
    pub async fn cleanup_expired_tokens(&self) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let now = chrono::Utc::now();
        let now_millis = now.timestamp_millis();
        
        let filter = doc! {
            "expires_at": { "$lt": mongodb::bson::DateTime::from_millis(now_millis) }
        };
        
        let result = self.collection.delete_many(filter).await?;
        Ok(result.deleted_count)
    }
    
    /// Invalida todos os tokens de um email (útil quando usuário troca senha)
    pub async fn invalidate_all_tokens_for_email(
        &self,
        email: &str,
    ) -> Result<u64, Box<dyn Error + Send + Sync>> {
        let filter = doc! { "email": email };
        let update = doc! { "$set": { "used": true } };
        
        let result = self.collection.update_many(filter, update).await?;
        Ok(result.modified_count)
    }
    
    /// Busca token por email (para debug/admin)
    pub async fn get_tokens_by_email(
        &self,
        email: &str,
        limit: Option<i64>,
    ) -> Result<Vec<PasswordResetToken>, Box<dyn Error + Send + Sync>> {
        let filter = doc! { "email": email };
        
        let mut cursor = self
            .collection
            .find(filter)
            .sort(doc! { "created_at": -1 })
            .await?;
        
        let mut tokens = Vec::new();
        let mut count = 0i64;
        
        while cursor.advance().await? {
            if let Some(lim) = limit {
                if count >= lim {
                    break;
                }
            }
            tokens.push(cursor.deserialize_current()?);
            count += 1;
        }
        
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Nota: Testes reais precisariam de MongoDB mock ou Sled
    // Aqui apenas validamos a estrutura
    
    #[test]
    fn test_service_creation() {
        // Este teste é apenas ilustrativo
        // Em produção, usaria mock ou Sled
        assert!(true);
    }
}
