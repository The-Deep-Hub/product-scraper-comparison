pub mod user;
pub mod scraping;

use async_trait::async_trait;
use bson::{doc, Document};
use futures_util::TryStreamExt;
use mongodb::{
    Collection,
    options::FindOptions,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::error::AppResult;

#[async_trait]
pub trait Repository<T>
where
    T: DeserializeOwned + Serialize + Unpin + Send + Sync,
{
    fn collection(&self) -> &Collection<T>;

    async fn find_one(&self, filter: Document) -> AppResult<Option<T>> {
        Ok(self.collection().find_one(filter, None).await?)
    }

    async fn find_by_id(&self, id: &str) -> AppResult<Option<T>> {
        let filter = doc! { "_id": id };
        Ok(self.collection().find_one(filter, None).await?)
    }

    async fn find(&self, filter: Document, options: Option<FindOptions>) -> AppResult<Vec<T>> {
        let mut cursor = self.collection().find(filter, options).await?;
        let mut results = Vec::new();
        while let Some(doc) = cursor.try_next().await? {
            results.push(doc);
        }
        Ok(results)
    }

    async fn insert_one(&self, document: &T) -> AppResult<()> {
        self.collection().insert_one(document, None).await?;
        Ok(())
    }

    async fn update_one(&self, filter: Document, update: Document) -> AppResult<bool> {
        let result = self.collection().update_one(filter, update, None).await?;
        Ok(result.modified_count > 0)
    }

    async fn delete_one(&self, filter: Document) -> AppResult<bool> {
        let result = self.collection().delete_one(filter, None).await?;
        Ok(result.deleted_count > 0)
    }
} 