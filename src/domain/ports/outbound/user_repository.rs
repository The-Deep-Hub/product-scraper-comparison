use async_trait::async_trait;
use crate::domain::{
    models::user::{User, NewUser},
    models::DomainResult,
};

/// Port for user persistence operations
#[async_trait]
pub trait UserRepository: Send + Sync {
    /// Find a user by their email
    async fn find_by_email(&self, email: &str) -> DomainResult<Option<User>>;
    
    /// Create a new user
    async fn create(&self, user: NewUser) -> DomainResult<User>;
    
    /// Update a user
    async fn update(&self, user: User) -> DomainResult<User>;
    
    /// Delete a user by ID
    async fn delete(&self, id: &str) -> DomainResult<bool>;
} 