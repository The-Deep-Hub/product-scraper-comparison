# Authentication System Documentation

## Overview
The authentication system provides secure user authentication and authorization using JWT tokens. It includes features such as:
- User registration and login
- Password reset functionality
- Rate limiting for login attempts
- Input validation for user data
- Role-based access control

## Configuration
The authentication system requires the following environment variables:

```env
# JWT Configuration
JWT_SECRET=your_jwt_secret_here
JWT_EXPIRATION=3600  # Token expiration in seconds

# Redis Configuration (for rate limiting and password reset)
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_redis_password_here

# MongoDB Configuration
MONGO_HOST=localhost
MONGO_PORT=27017
MONGO_DATABASE=rust_scraper
MONGO_APP_USERNAME=your_app_username
MONGO_APP_PASSWORD=your_app_password
```

## API Endpoints

### Registration
```http
POST /auth/register
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "StrongPassword123!",
    "confirm_password": "StrongPassword123!",
    "name": "John Doe",
    "role": "Basic"  // Optional, defaults to "Basic"
}
```

Response:
```json
{
    "id": "user_id",
    "email": "user@example.com",
    "name": "John Doe",
    "role": "Basic"
}
```

### Login
```http
POST /auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "StrongPassword123!"
}
```

Response:
```json
{
    "token": "jwt_token_here",
    "user": {
        "id": "user_id",
        "email": "user@example.com",
        "name": "John Doe",
        "role": "Basic"
    }
}
```

### Password Reset Request
```http
POST /password/reset
Content-Type: application/json

{
    "email": "user@example.com"
}
```

Response:
```json
{
    "message": "If the email exists, a password reset link has been sent"
}
```

### Password Reset Confirmation
```http
POST /password/reset/confirm
Content-Type: application/json

{
    "reset_token": "token_from_email",
    "password": "NewPassword123!",
    "confirm_password": "NewPassword123!"
}
```

Response:
```json
{
    "message": "Password has been successfully reset"
}
```

## Security Features

### Password Requirements
- Minimum length: 8 characters
- Must contain at least one uppercase letter
- Must contain at least one lowercase letter
- Must contain at least one number
- Must contain at least one special character

### Rate Limiting
- Login attempts are limited to 5 per minute per IP address
- Password reset requests are limited to 3 per hour per email address
- Rate limit information is stored in Redis with appropriate expiration

### JWT Token
- Tokens include user ID, email, and role
- Default expiration time is 1 hour
- Tokens are signed using HS256 algorithm
- Token validation includes expiration check and signature verification

### Input Validation
- Email format validation
- Password strength validation
- Password confirmation matching
- Request body validation for required fields

## Error Handling

### HTTP Status Codes
- 200: Success
- 400: Bad Request (invalid input)
- 401: Unauthorized (invalid credentials)
- 403: Forbidden (insufficient permissions)
- 429: Too Many Requests (rate limit exceeded)
- 500: Internal Server Error

### Error Response Format
```json
{
    "error": {
        "code": "ERROR_CODE",
        "message": "Human readable error message"
    }
}
```

## Testing
The authentication system includes comprehensive tests:
- Unit tests for validation functions
- Integration tests for authentication flows
- Rate limiting tests
- Password reset flow tests

Run tests using:
```bash
cargo test auth
cargo test password
cargo test validation
cargo test rate_limit
```

## Security Best Practices
1. Passwords are hashed using bcrypt with appropriate cost factor
2. Rate limiting prevents brute force attacks
3. JWT tokens are signed and have short expiration times
4. Input validation prevents injection attacks
5. Error messages don't leak sensitive information
6. Redis keys have appropriate TTL values
7. MongoDB connections use TLS and authentication

## Monitoring and Logging
- Failed login attempts are logged with IP address (but not passwords)
- Password reset requests are logged
- Rate limit exceeded events are logged
- JWT token validation failures are logged

## Dependencies
```toml
[dependencies]
jsonwebtoken = "8.1"
bcrypt = "0.10"
redis = "0.21"
mongodb = "2.3"
actix-web = "4.0"
serde = { version = "1.0", features = ["derive"] }
validator = "0.14"
``` 