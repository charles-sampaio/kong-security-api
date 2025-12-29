use mongodb::{Database, Collection, bson::{doc, oid::ObjectId}};
use futures_util::stream::StreamExt;
use std::error::Error;
use crate::models::{User, user::OAuthProvider};

pub struct UserService {
    db: Database,
}

impl UserService {
    pub fn new(db: Database) -> Self {
        Self { db }
    }

    pub fn users_collection(&self) -> Collection<User> {
        self.db.collection("users")
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { "email": email };
        
        match collection.find_one(filter).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn find_by_email_and_tenant(&self, email: &str, tenant_id: &str) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { 
            "email": email,
            "tenant_id": tenant_id 
        };
        
        match collection.find_one(filter).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Buscar usuário por ID e tenant
    pub async fn find_by_id(
        &self, 
        tenant_id: &str, 
        user_id: &str
    ) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        
        let object_id = match ObjectId::parse_str(user_id) {
            Ok(id) => id,
            Err(e) => return Err(Box::new(e)),
        };
        
        let filter = doc! { 
            "_id": object_id,
            "tenant_id": tenant_id,
        };
        
        match collection.find_one(filter).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Buscar usuário por OAuth provider e OAuth ID
    pub async fn find_by_oauth(
        &self, 
        tenant_id: &str, 
        provider: OAuthProvider, 
        oauth_id: &str
    ) -> Result<Option<User>, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { 
            "tenant_id": tenant_id,
            "oauth_provider": provider.as_str(),
            "oauth_id": oauth_id
        };
        
        match collection.find_one(filter).await {
            Ok(user) => Ok(user),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn create_user(&self, user: User) -> Result<ObjectId, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        
        match collection.insert_one(user).await {
            Ok(result) => {
                if let Some(id) = result.inserted_id.as_object_id() {
                    Ok(id)
                } else {
                    Err("Failed to get inserted ID".into())
                }
            },
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn update_user(&self, user: &User) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { "_id": user._id };
        let update = doc! {
            "$set": {
                "tenant_id": &user.tenant_id,
                "email": &user.email,
                "password": &user.password,
                "oauth_provider": user.oauth_provider.as_ref().map(|p| p.as_str()),
                "oauth_id": &user.oauth_id,
                "name": &user.name,
                "picture": &user.picture,
                "roles": &user.roles,
                "created_at": &user.created_at,
                "updated_at": &user.updated_at,
                "is_active": &user.is_active,
                "last_login": &user.last_login,
            }
        };

        match collection.update_one(filter, update).await {
            Ok(result) => Ok(result.modified_count > 0),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn delete_user(&self, id: &ObjectId) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { "_id": id };

        match collection.delete_one(filter).await {
            Ok(result) => Ok(result.deleted_count > 0),
            Err(e) => Err(Box::new(e)),
        }
    }

    pub async fn list_users(&self, limit: Option<i64>) -> Result<Vec<User>, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! {};
        
        let mut cursor = collection.find(filter).await?;
        let mut users = Vec::new();
        let mut count = 0;

        while let Some(user) = cursor.next().await {
            match user {
                Ok(u) => {
                    users.push(u);
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

        Ok(users)
    }
}