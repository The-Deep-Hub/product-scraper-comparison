use async_trait::async_trait;
use bson::{doc, Document};
use futures_util::TryStreamExt;
use mongodb::{Collection, Database};

use crate::db::{
    error::DbError,
    models::{Model, user::User},
    repositories::Repository,
};

pub struct UserRepository {
    db: Database,
}

impl UserRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
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
    fn collection(&self) -> Collection<User> {
        self.db.collection(User::collection_name())
    }

    async fn find_one(&self, filter: Document) -> Result<Option<User>, DbError> {
        self.collection()
            .find_one(filter, None)
            .await
            .map_err(DbError::from)
    }

    async fn find(
        &self,
        filter: Document,
        sort: Option<Document>,
        limit: Option<i64>,
        skip: Option<u64>,
    ) -> Result<Box<dyn futures_util::TryStream<Ok = User, Error = DbError, Item = Result<User, DbError>> + Send + Unpin + 'static>, DbError> {
        let mut options = mongodb::options::FindOptions::default();
        if let Some(sort) = sort {
            options.sort = Some(sort);
        }
        if let Some(limit) = limit {
            options.limit = Some(limit);
        }
        if let Some(skip) = skip {
            options.skip = Some(skip);
        }

        let cursor = self.collection().find(filter, options).await?;
        Ok(Box::new(cursor.map_err(DbError::from)))
    }

    async fn create(&self, document: User) -> Result<User, DbError> {
        let result = self.collection().insert_one(document, None).await?;
        let filter = doc! { "_id": result.inserted_id };
        self.find_one(filter)
            .await?
            .ok_or_else(|| DbError::NotFound("Created document not found".to_string()))
    }

    async fn update_by_id(
        &self,
        id: bson::oid::ObjectId,
        update: Document,
    ) -> Result<Option<User>, DbError> {
        let filter = doc! { "_id": id };
        let result = self.collection()
            .update_one(filter.clone(), update, None)
            .await?;

        if result.modified_count == 0 {
            return Ok(None);
        }

        self.find_one(filter).await
    }

    async fn delete_by_id(&self, id: bson::oid::ObjectId) -> Result<bool, DbError> {
        let filter = doc! { "_id": id };
        let result = self.collection().delete_one(filter, None).await?;
        Ok(result.deleted_count > 0)
    }

    async fn count(&self, filter: Document) -> Result<u64, DbError> {
        self.collection()
            .count_documents(filter, None)
            .await
            .map_err(DbError::from)
    }
} 