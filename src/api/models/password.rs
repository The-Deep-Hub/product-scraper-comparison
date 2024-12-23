use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordReset {
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordUpdate {
    pub token: String,
    #[validate(length(min = 8))]
    pub new_password: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordChange {
    #[validate(length(min = 8))]
    pub current_password: String,
    #[validate(length(min = 8))]
    pub new_password: String,
} 