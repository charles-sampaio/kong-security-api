use mongodb::{Database, Collection, bson::{doc, oid::ObjectId}};
use futures_util::stream::StreamExt;
use std::error::Error;
use crate::models::User;

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

    pub async fn create_user(&self, user: &User) -> Result<ObjectId, Box<dyn Error + Send + Sync>> {
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

    pub async fn update_user(&self, id: &ObjectId, user: &User) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let collection = self.users_collection();
        let filter = doc! { "_id": id };
        let update = doc! {
            "$set": {
                "email": &user.email,
                "password": &user.password,
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