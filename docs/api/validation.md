# Input Validation System Documentation

## Overview
The input validation system ensures that all data entering the application meets the required format and security standards. It provides comprehensive validation for user input, particularly focusing on authentication-related data such as emails and passwords.

## Features
- Email format validation
- Password strength requirements
- Password confirmation matching
- Request body validation
- Custom validation rules
- Reusable validation components
- Detailed error messages

## Validation Rules

### Email Validation
```rust
pub fn validate_email(email: &str) -> bool {
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    email_regex.is_match(email)
}
```

Requirements:
- Must contain a single `@` symbol
- Local part can contain letters, numbers, and common special characters
- Domain part must be valid with at least one period
- TLD must be at least 2 characters

### Password Validation
```rust
pub fn validate_password(password: &str) -> bool {
    let length_regex = Regex::new(r".{8,}").unwrap();
    let uppercase_regex = Regex::new(r"[A-Z]").unwrap();
    let lowercase_regex = Regex::new(r"[a-z]").unwrap();
    let digit_regex = Regex::new(r"[0-9]").unwrap();
    let special_regex = Regex::new(r"[!@#$%^&*(),.?\":{}|<>]").unwrap();

    length_regex.is_match(password) &&
    uppercase_regex.is_match(password) &&
    lowercase_regex.is_match(password) &&
    digit_regex.is_match(password) &&
    special_regex.is_match(password)
}
```

Requirements:
- Minimum length: 8 characters
- Must contain at least one uppercase letter
- Must contain at least one lowercase letter
- Must contain at least one number
- Must contain at least one special character

### Password Confirmation
```rust
pub fn validate_passwords_match(password: &str, confirm_password: &str) -> bool {
    password == confirm_password
}
```

## Request Validation Structures

### Registration Request
```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegistrationRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(custom = "validate_password")]
    pub password: String,
    
    pub confirm_password: String,
    
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    
    #[validate(custom = "validate_role")]
    pub role: Option<String>,
}
```

### Login Request
```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(custom = "validate_password")]
    pub password: String,
}
```

### Password Reset Request
```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordResetRequest {
    #[validate(email)]
    pub email: String,
}
```

### Password Update Request
```rust
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct PasswordUpdateRequest {
    #[validate(custom = "validate_password")]
    pub password: String,
    
    pub confirm_password: String,
    
    #[validate(length(equal = 64))]
    pub reset_token: String,
}
```

## Usage

### Middleware Implementation
```rust
use actix_web::{dev::ServiceRequest, Error};
use validator::Validate;

pub async fn validate_request<T>(req: &ServiceRequest) -> Result<(), Error>
where
    T: DeserializeOwned + Validate,
{
    let body = req.get_body().await?;
    let data: T = serde_json::from_slice(&body)?;
    
    if let Err(errors) = data.validate() {
        return Err(ValidationError::new(errors).into());
    }
    
    Ok(())
}
```

### Using Validation in Routes
```rust
use actix_web::{web, post, HttpResponse};
use crate::api::middleware::validation::validate_request;

#[post("/register")]
async fn register(
    data: web::Json<RegistrationRequest>,
) -> HttpResponse {
    if let Err(err) = validate_request::<RegistrationRequest>(&data).await {
        return HttpResponse::BadRequest().json(err);
    }
    
    // Process registration
}
```

## Error Handling

### Validation Error Response
```json
{
    "error": {
        "code": "VALIDATION_ERROR",
        "message": "Invalid input data",
        "details": {
            "email": ["Invalid email format"],
            "password": [
                "Password must be at least 8 characters long",
                "Password must contain at least one uppercase letter"
            ]
        }
    }
}
```

### Error Types
1. Format Validation Errors
   - Invalid email format
   - Weak password
   - Mismatched passwords

2. Length Validation Errors
   - Input too short/long
   - Missing required fields

3. Custom Validation Errors
   - Invalid role values
   - Business logic violations

## Testing

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_email_validation() {
        assert!(validate_email("user@example.com"));
        assert!(!validate_email("invalid-email"));
    }

    #[test]
    fn test_password_validation() {
        assert!(validate_password("StrongP@ss1"));
        assert!(!validate_password("weak"));
    }

    #[test]
    fn test_password_match() {
        assert!(validate_passwords_match("Pass123!", "Pass123!"));
        assert!(!validate_passwords_match("Pass123!", "Different123!"));
    }
}
```

### Integration Tests
```rust
#[actix_rt::test]
async fn test_registration_validation() {
    let app = test::init_service(
        App::new()
            .service(register)
    ).await;

    let req = test::TestRequest::post()
        .uri("/register")
        .set_json(json!({
            "email": "invalid-email",
            "password": "weak",
            "confirm_password": "different",
            "name": ""
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), http::StatusCode::BAD_REQUEST);
}
```

## Best Practices

1. **Input Sanitization**
   - Trim whitespace
   - Normalize email addresses
   - Remove control characters

2. **Security**
   - Validate input length limits
   - Prevent common injection patterns
   - Use secure regex patterns

3. **User Experience**
   - Provide clear error messages
   - Validate in real-time where possible
   - Return all validation errors at once

4. **Performance**
   - Cache compiled regex patterns
   - Use efficient validation methods
   - Validate before expensive operations

## Dependencies
```toml
[dependencies]
validator = "0.14"
regex = "1.5"
serde = { version = "1.0", features = ["derive"] }
actix-web = "4.0"
``` 