# Rust Scraper API Documentation

## Overview
The Rust Scraper API provides a RESTful interface for web scraping operations, with support for distributed task processing, caching, and authentication.

## Base URL
```
http://localhost:8080/api
```

## Authentication
The API uses JWT (JSON Web Token) for authentication. Most endpoints require a valid JWT token in the Authorization header.

### Headers
```
Authorization: Bearer <your_jwt_token>
```

### Public Endpoints
The following endpoints do not require authentication:
- `POST /auth/register`
- `POST /auth/login`
- `GET /health`

## Endpoints

### Authentication

#### Register User
```http
POST /auth/register
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123",
    "password_confirmation": "password123"
}
```

Response:
```json
{
    "email": "user@example.com",
    "role": "user",
    "created_at": "2024-01-23T12:00:00Z"
}
```

#### Login
```http
POST /auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "password123"
}
```

Response:
```json
{
    "message": "Login successful",
    "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
    "email": "user@example.com",
    "role": "user"
}
```

### Scraping Operations

#### Submit Scraping Request
```http
POST /scraper
Authorization: Bearer <token>
Content-Type: application/json

{
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
}
```

Response (Cache Hit):
```json
{
    "results": [...],
    "cache_status": "hit",
    "stores": "bricodepot,leroy"
}
```

Response (Cache Miss):
```json
{
    "task_id": "123e4567-e89b-12d3-a456-426614174000",
    "message": "Search request accepted",
    "cache_status": "miss",
    "stores": "bricodepot,leroy"
}
```

#### Get Search Results
```http
GET /scraper/{task_id}
Authorization: Bearer <token>
```

Response:
```json
{
    "task": {
        "id": "123e4567-e89b-12d3-a456-426614174000",
        "status": "completed",
        "created_at": "2024-01-23T12:00:00Z",
        "updated_at": "2024-01-23T12:01:00Z"
    },
    "results": [
        {
            "title": "Professional Hammer",
            "price": 29.99,
            "store": "Bricodepot",
            "url": "https://..."
        }
    ]
}
```

#### List Active Tasks
```http
GET /scraper/tasks
Authorization: Bearer <token>
```

Response:
```json
[
    {
        "task_id": "123e4567-e89b-12d3-a456-426614174000",
        "query": "hammer",
        "store": "Bricodepot",
        "status": "pending",
        "created_at": "2024-01-23T12:00:00Z"
    }
]
```

#### Cancel Search
```http
POST /scraper/{task_id}/cancel
Authorization: Bearer <token>
```

Response:
```json
{
    "message": "Search cancelled successfully"
}
```

### Job Management

#### List All Jobs
```http
GET /jobs
Authorization: Bearer <token>
```

Response:
```json
[
    {
        "task_id": "123e4567-e89b-12d3-a456-426614174000",
        "status": "completed",
        "created_at": "2024-01-23T12:00:00Z",
        "updated_at": "2024-01-23T12:01:00Z"
    }
]
```

#### Retry Failed Job
```http
POST /jobs/{id}/retry
Authorization: Bearer <token>
```

Response:
```json
{
    "message": "Task retry initiated"
}
```

## Error Responses

The API uses standard HTTP status codes and returns error messages in a consistent format:

```json
{
    "error": "ErrorType",
    "message": "Detailed error message"
}
```

Common error codes:
- `400 Bad Request`: Invalid input parameters
- `401 Unauthorized`: Missing or invalid authentication token
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict (e.g., user already exists)
- `500 Internal Server Error`: Server-side error

## Caching Strategy

- Search results are cached in Redis with a TTL (Time To Live)
- Cache keys are formatted as `search:{query}:{stores}`
- Cache is checked before initiating new scraping tasks
- Cache invalidation occurs automatically after TTL expires

## Rate Limiting

- Rate limiting is applied per user and IP address
- Limits are enforced on a per-endpoint basis
- Rate limit headers are included in responses:
  ```
  X-RateLimit-Limit: 100
  X-RateLimit-Remaining: 99
  X-RateLimit-Reset: 1706016000
  ``` 