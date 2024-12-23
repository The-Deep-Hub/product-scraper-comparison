use futures_util::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection,
};
use uuid::Uuid;

use crate::{
    db::models::scraping::{ScrapingTask, ScrapingResult},
    error::AppResult,
};

pub struct ScrapingRepository {
    tasks: Collection<ScrapingTask>,
    results: Collection<ScrapingResult>,
}

impl ScrapingRepository {
    pub fn new(db: mongodb::Database) -> Self {
        Self {
            tasks: db.collection("scraping_tasks"),
            results: db.collection("scraping_results"),
        }
    }

    pub async fn find_active_tasks(&self) -> AppResult<Vec<ScrapingTask>> {
        let filter = doc! { "status": "pending" };
        let cursor = self.tasks.find(filter, None).await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    pub async fn find_task_by_uuid(&self, task_id: Uuid) -> AppResult<Option<ScrapingTask>> {
        let filter = doc! { "task_id": task_id.to_string() };
        Ok(self.tasks.find_one(filter, None).await?)
    }

    pub async fn find_results_by_task(&self, task_id: Uuid) -> AppResult<Vec<ScrapingResult>> {
        let filter = doc! { "task_id": task_id.to_string() };
        let cursor = self.results.find(filter, None).await?;
        cursor.try_collect().await.map_err(Into::into)
    }

    pub async fn create_task(&self, task: &ScrapingTask) -> AppResult<ObjectId> {
        let result = self.tasks.insert_one(task, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn update_task_status(&self, task_id: Uuid, status: &str) -> AppResult<()> {
        let filter = doc! { "task_id": task_id.to_string() };
        let update = doc! {
            "$set": {
                "status": status,
                "updated_at": chrono::Utc::now()
            }
        };
        self.tasks.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn create_result(&self, result: &ScrapingResult) -> AppResult<ObjectId> {
        let result = self.results.insert_one(result, None).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }
} 