use serde::{Deserialize, Serialize};

/// Core user entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Option<String>,
    pub email: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Value object for user creation
#[derive(Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub email: String,
    pub password: String,
} 