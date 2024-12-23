use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use bcrypt::{hash, DEFAULT_COST};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub email: String,
    pub password: String,
    pub role: String,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(email: String, password: String) -> Self {
        let hashed_password = hash(password.as_bytes(), DEFAULT_COST)
            .expect("Failed to hash password");
        
        Self {
            id: None,
            email,
            password: hashed_password,
            role: "user".to_string(),
            created_at: chrono::Utc::now(),
        }
    }
} 