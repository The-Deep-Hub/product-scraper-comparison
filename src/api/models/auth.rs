use serde::{Deserialize, Serialize};
use crate::api::middleware::auth::Role;
use std::collections::HashMap;
use std::sync::RwLock;
use lazy_static::lazy_static;
use config::Config;

#[derive(Debug, Clone)]
pub struct User {
    pub email: String,
    pub password: String, // In production, this would be hashed
    pub name: String,
    pub role: Role,
}

lazy_static! {
    static ref USERS: RwLock<HashMap<String, User>> = {
        let mut users = HashMap::new();
        
        // Load config
        let config = Config::builder()
            .add_source(config::File::with_name("config/default"))
            .build()
            .expect("Failed to load config");

        // Add development admin user
        if config.get_string("environment").unwrap_or_default() == "development" {
            users.insert(
                config.get_string("development.admin_email").unwrap(),
                User {
                    email: config.get_string("development.admin_email").unwrap(),
                    password: config.get_string("development.admin_password").unwrap(),
                    name: config.get_string("development.admin_name").unwrap(),
                    role: Role::Admin,
                },
            );
        }
        
        RwLock::new(users)
    };
}

pub fn add_user(user: User) -> Result<(), String> {
    let mut users = USERS.write().unwrap();
    if users.contains_key(&user.email) {
        return Err("User already exists".to_string());
    }
    users.insert(user.email.clone(), user);
    Ok(())
}

pub fn get_user(email: &str) -> Option<User> {
    let users = USERS.read().unwrap();
    users.get(email).cloned()
}

pub fn verify_credentials(email: &str, password: &str) -> Option<User> {
    let users = USERS.read().unwrap();
    users.get(email)
        .filter(|user| user.password == password)
        .cloned()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub name: String,
    pub role: Option<Role>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub email: String,
    pub name: String,
    pub role: Role,
} 