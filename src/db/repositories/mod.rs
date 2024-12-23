pub mod user;

use async_trait::async_trait;
use bson::Document;
use futures_util::TryStream;
use mongodb::Collection;

use crate::db::{
    error::DbError,
    models::Model,
};

#[async_trait]
pub trait Repository<T: Model + Send + Sync + 'static> {
    fn collection(&self) -> Collection<T>;

    async fn find_one(&self, filter: Document) -> Result<Option<T>, DbError>;

    async fn find(
        &self,
        filter: Document,
        sort: Option<Document>,
        limit: Option<i64>,
        skip: Option<u64>,
    ) -> Result<Box<dyn TryStream<Ok = T, Error = DbError, Item = Result<T, DbError>> + Send + Unpin + 'static>, DbError>;

    async fn create(&self, document: T) -> Result<T, DbError>;

    async fn update_by_id(
        &self,
        id: bson::oid::ObjectId,
        update: Document,
    ) -> Result<Option<T>, DbError>;

    async fn delete_by_id(&self, id: bson::oid::ObjectId) -> Result<bool, DbError>;

    async fn count(&self, filter: Document) -> Result<u64, DbError>;
} 