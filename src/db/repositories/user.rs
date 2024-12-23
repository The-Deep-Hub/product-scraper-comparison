use async_trait::async_trait;
use bson::{doc, Document, oid::ObjectId};
use futures::stream::TryStream;
use mongodb::{Collection, options::FindOptions};

use crate::db::{
    error::DbError,
    models::{user::User, Model},
    MongoDb,
};

use super::Repository;

pub struct UserRepository {
    collection: Collection<User>,
}

impl UserRepository {
    pub fn new(db: &MongoDb) -> Self {
        Self {
            collection: db.get_database().collection(User::collection_name()),
        }
    }

    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>, DbError> {
        let filter = doc! { "email": email };
        self.find_one(filter).await
    }

    pub async fn exists_by_email(&self, email: &str) -> Result<bool, DbError> {
        let filter = doc! { "email": email };
        let count = self.count(filter).await?;
        Ok(count > 0)
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<User>, DbError> {
        let filter = doc! { "_id": id };
        self.find_one(filter).await
    }

    async fn find(
        &self,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<impl TryStream<Ok = User, Error = DbError>, DbError> {
        let cursor = self.collection.find(filter, options).await?;
        Ok(cursor.map_err(DbError::from))
    }

    async fn find_one(&self, filter: Document) -> Result<Option<User>, DbError> {
        Ok(self.collection.find_one(filter, None).await?)
    }

    async fn insert(&self, user: &User) -> Result<ObjectId, DbError> {
        let result = self.collection.insert_one(user, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    async fn update_by_id(&self, id: ObjectId, update: Document) -> Result<bool, DbError> {
        let filter = doc! { "_id": id };
        let result = self.collection.update_one(filter, update, None).await?;
        Ok(result.modified_count > 0)
    }

    async fn update_many(&self, filter: Document, update: Document) -> Result<u64, DbError> {
        let result = self.collection.update_many(filter, update, None).await?;
        Ok(result.modified_count)
    }

    async fn delete_by_id(&self, id: ObjectId) -> Result<bool, DbError> {
        let filter = doc! { "_id": id };
        let result = self.collection.delete_one(filter, None).await?;
        Ok(result.deleted_count > 0)
    }

    async fn delete_many(&self, filter: Document) -> Result<u64, DbError> {
        let result = self.collection.delete_many(filter, None).await?;
        Ok(result.deleted_count)
    }

    async fn count(&self, filter: Document) -> Result<u64, DbError> {
        Ok(self.collection.count_documents(filter, None).await?)
    }
} 