pub mod user;

use bson::{Document, Bson};
use serde::{Serialize, de::DeserializeOwned};

pub trait Model: Serialize + DeserializeOwned {
    /// Get the collection name for this model
    fn collection_name() -> &'static str;

    /// Convert the model to a BSON document
    fn to_document(&self) -> Result<Document, bson::ser::Error> {
        bson::to_document(self)
    }

    /// Create a model from a BSON document
    fn from_document(doc: Document) -> Result<Self, bson::de::Error> {
        bson::from_document(doc)
    }
}

pub trait Timestamps {
    fn created_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn updated_at(&self) -> chrono::DateTime<chrono::Utc>;
    fn set_created_at(&mut self, time: chrono::DateTime<chrono::Utc>);
    fn set_updated_at(&mut self, time: chrono::DateTime<chrono::Utc>);
}

pub trait WithId {
    fn id(&self) -> Option<&bson::oid::ObjectId>;
    fn set_id(&mut self, id: bson::oid::ObjectId);
}

pub trait SoftDelete {
    fn deleted_at(&self) -> Option<chrono::DateTime<chrono::Utc>>;
    fn set_deleted_at(&mut self, time: Option<chrono::DateTime<chrono::Utc>>);
    fn is_deleted(&self) -> bool {
        self.deleted_at().is_some()
    }
} 