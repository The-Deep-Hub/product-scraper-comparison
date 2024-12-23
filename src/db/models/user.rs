use serde::{Serialize, Deserialize};
use bson::oid::ObjectId;
use chrono::{DateTime, Utc};
use crate::api::middleware::auth::Role;

use super::{Model, Timestamps, WithId, SoftDelete};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    id: Option<ObjectId>,
    pub email: String,
    pub password_hash: String,
    pub name: String,
    pub role: Role,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl User {
    pub fn new(email: String, password_hash: String, name: String, role: Role) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            email,
            password_hash,
            name,
            role,
            created_at: now,
            updated_at: now,
            deleted_at: None,
        }
    }
}

impl Model for User {
    fn collection_name() -> &'static str {
        "users"
    }
}

impl Timestamps for User {
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    fn set_created_at(&mut self, time: DateTime<Utc>) {
        self.created_at = time;
    }

    fn set_updated_at(&mut self, time: DateTime<Utc>) {
        self.updated_at = time;
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

impl SoftDelete for User {
    fn deleted_at(&self) -> Option<DateTime<Utc>> {
        self.deleted_at
    }

    fn set_deleted_at(&mut self, time: Option<DateTime<Utc>>) {
        self.deleted_at = time;
    }
} 