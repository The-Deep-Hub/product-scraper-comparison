pub mod user;
pub mod scraping;

use async_trait::async_trait;

#[async_trait]
pub trait Model {
    fn collection_name() -> &'static str;
}

pub trait WithId {
    fn id(&self) -> Option<&bson::oid::ObjectId>;
    fn set_id(&mut self, id: bson::oid::ObjectId);
} 