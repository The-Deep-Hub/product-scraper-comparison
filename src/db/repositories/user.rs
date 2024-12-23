use async_trait::async_trait;
use mongodb::{
    bson::{doc, Document, oid::ObjectId},
    Collection, Database,
};

use crate::{
    db::models::user::User,
    error::AppResult,
};

use super::Repository;

#[derive(Clone)]
pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self {
            collection: db.collection("users"),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<User>> {
        let filter = doc! { "email": email };
        self.find_one(filter).await
    }

    pub async fn create_user(&self, user: &User) -> AppResult<ObjectId> {
        let result = self.collection
            .insert_one(user, None)
            .await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn update_password(&self, id: ObjectId, password_hash: String) -> AppResult<bool> {
        let update = doc! {
            "$set": {
                "password": password_hash
            }
        };
        self.update_one(id, update).await?;
        Ok(true)
    }

    pub async fn update_one(&self, id: ObjectId, update: Document) -> AppResult<()> {
        let filter = doc! { "_id": id };
        self.collection
            .update_one(filter, update, None)
            .await?;
        Ok(())
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    fn collection(&self) -> &Collection<User> {
        &self.collection
    }
} 