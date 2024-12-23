use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::{
    api::middleware::auth::Role,
    db::models::{Model, WithId},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Model for User {
    fn collection_name() -> &'static str {
        "users"
    }
}

impl WithId for User {
    fn id(&self) -> Option<&ObjectId> {
        self.id.as_ref()
    }

    fn set_id(&mut self, id: ObjectId) {
        self.id = Some(id);
    }
} 