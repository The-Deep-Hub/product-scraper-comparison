# Rust Scraper API Documentation

## Base URL
```
http://localhost:8080/api
```

## Authentication
The API uses JWT (JSON Web Token) authentication. Protected endpoints require a valid JWT token to be included in the request header:

```
Authorization: Bearer <token>
```

## Endpoints

### Public Routes

#### Register User
Creates a new user account.

- **URL**: `/auth/register`
- **Method**: `POST`
- **Authentication**: None
- **Request Body**:
  ```json
  {
    "email": "string",
    "password": "string",
    "password_confirmation": "string"
  }
  ```
- **Success Response (200 OK)**:
  ```json
  {
    "message": "User registered successfully",
    "email": "string",
    "role": "string"
  }
  ```
- **Error Responses**:
  - `400 Bad Request`:
    ```json
    {
      "error": "User with this email already exists"
    }
    ```
  - `500 Internal Server Error`:
    ```json
    {
      "error": "Failed to create user"
    }
    ```
    or
    ```json
    {
      "error": "Failed to hash password"
    }
    ```

#### Login
Authenticates a user and returns a JWT token.

- **URL**: `/auth/login`
- **Method**: `POST`
- **Authentication**: None
- **Request Body**:
  ```json
  {
    "email": "string",
    "password": "string"
  }
  ```
- **Success Response (200 OK)**:
  ```json
  {
    "message": "Login successful",
    "token": "string",
    "email": "string",
    "role": "string"
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`:
    ```json
    {
      "error": "Invalid email or password"
    }
    ```
  - `500 Internal Server Error`:
    ```json
    {
      "error": "Failed to find user"
    }
    ```

### Protected Routes
All protected routes require authentication via JWT token in the Authorization header.

#### Health Check
Checks the health status of the API.

- **URL**: `/protected/health`
- **Method**: `GET`
- **Authentication**: Required
- **Success Response (200 OK)**:
  ```json
  {
    "status": "ok",
    "timestamp": "string"
  }
  ```
- **Error Responses**:
  - `401 Unauthorized`:
    ```json
    {
      "error": "No authorization header"
    }
    ```
    or
    ```json
    {
      "error": "Invalid token"
    }
    ```

## Data Models

### User
```json
{
  "id": "ObjectId",
  "email": "string",
  "password_hash": "string",
  "role": "User | Admin",
  "created_at": "DateTime<Utc>",
  "updated_at": "DateTime<Utc>"
}
```

### Role
Available user roles:
- `User`: Standard user role
- `Admin`: Administrative user role

## Error Handling
The API uses standard HTTP status codes:
- `200`: Success
- `400`: Bad Request (client error)
- `401`: Unauthorized (missing or invalid authentication)
- `403`: Forbidden (insufficient permissions)
- `404`: Not Found
- `500`: Internal Server Error

All error responses follow the format:
```json
{
  "error": "string"
}
```

## Rate Limiting
The API implements rate limiting on protected routes to prevent abuse. Rate limits are configured as follows:
- Maximum 100 requests per minute per IP address
- Rate limit status is tracked using Redis

## Dependencies
- MongoDB: User data storage
- Redis: Rate limiting and session management
- JWT: Authentication tokens
- bcrypt: Password hashing 