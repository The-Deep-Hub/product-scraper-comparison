pub mod user;

use async_trait::async_trait;
use bson::{Document, oid::ObjectId};
use futures::stream::TryStream;
use mongodb::options::FindOptions;

use crate::db::{
    error::DbError,
    models::{Model, WithId},
};

#[async_trait]
pub trait Repository<T>
where
    T: Model + WithId + Send + Sync,
{
    /// Find a document by its ID
    async fn find_by_id(&self, id: ObjectId) -> Result<Option<T>, DbError>;

    /// Find documents matching a filter
    async fn find(
        &self,
        filter: Document,
        options: Option<FindOptions>,
    ) -> Result<impl TryStream<Ok = T, Error = DbError>, DbError>;

    /// Find one document matching a filter
    async fn find_one(&self, filter: Document) -> Result<Option<T>, DbError>;

    /// Insert a new document
    async fn insert(&self, model: &T) -> Result<ObjectId, DbError>;

    /// Update a document by its ID
    async fn update_by_id(&self, id: ObjectId, update: Document) -> Result<bool, DbError>;

    /// Update documents matching a filter
    async fn update_many(&self, filter: Document, update: Document) -> Result<u64, DbError>;

    /// Delete a document by its ID
    async fn delete_by_id(&self, id: ObjectId) -> Result<bool, DbError>;

    /// Delete documents matching a filter
    async fn delete_many(&self, filter: Document) -> Result<u64, DbError>;

    /// Count documents matching a filter
    async fn count(&self, filter: Document) -> Result<u64, DbError>;
} 