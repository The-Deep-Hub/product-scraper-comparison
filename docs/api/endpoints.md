# 🛣️ API Endpoints

<div align="center">

*API endpoints documentation for the Rust Web Scraper*

</div>

## 📑 Table of Contents

- [Overview](#overview)
- [Base URL](#base-url)
- [Authentication](#authentication)
- [Scraping Operations](#scraping-operations)
- [Future Improvements](#future-improvements)

## Overview

This document details all available API endpoints, their parameters, and response formats. All endpoints are relative to the base URL.

## Base URL

```
http://localhost:8080/api
```

## Authentication

Most endpoints require JWT authentication. Include the token in the Authorization header:

```http
Authorization: Bearer <your_jwt_token>
```

### Public Endpoints
The following endpoints do not require authentication:
- `POST /auth/register`
- `POST /auth/login`
- `GET /health`

## Scraping Operations

### Submit Scraping Request

```http
POST /scraper
```

Submit a new scraping request for one or more stores.

#### Request Body

```json
{
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
}
```

#### Response (Cache Hit)

```json
{
    "results": [
        {
            "name": "Professional Hammer",
            "description": "Heavy-duty hammer for professional use",
            "price": 29.99,
            "url": "https://example.com/hammer",
            "image_url": "https://example.com/hammer.jpg"
        }
    ],
    "cache_status": "hit",
    "stores": "bricodepot,leroy"
}
```

#### Response (Cache Miss)

```json
{
    "task_id": "123e4567-e89b-12d3-a456-426614174000",
    "message": "Search request accepted",
    "cache_status": "miss",
    "stores": "bricodepot,leroy"
}
```

### Get Search Results

```http
GET /scraper/{task_id}
```

Retrieve results for a specific scraping task.

#### Response

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
            "name": "Professional Hammer",
            "description": "Heavy-duty hammer for professional use",
            "price": 29.99,
            "url": "https://example.com/hammer",
            "image_url": "https://example.com/hammer.jpg"
        }
    ]
}
```

### List Active Tasks

```http
GET /scraper/tasks
```

List all active scraping tasks.

#### Response

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

### Cancel Search

```http
POST /scraper/{task_id}/cancel
```

Cancel an ongoing scraping task.

#### Response

```json
{
    "message": "Search cancelled successfully"
}
```

## Error Responses

All endpoints use a consistent error response format:

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
- `409 Conflict`: Resource conflict
- `500 Internal Server Error`: Server-side error 

## Future Improvements

### API Enhancements
- Add pagination support for search results
- Implement filtering by price range, store, and product category
- Add sorting options for search results (by price, relevance, date)
- Support bulk operations for multiple tasks
- Add rate limiting headers to API responses

### Authentication & Security
- Implement OAuth2 authentication flow
- Add API key rotation mechanism
- Implement role-based access control (RBAC)
- Add request signing for enhanced security
- Implement IP whitelisting for sensitive operations

### Performance Optimizations
- Implement response compression
- Add field selection to reduce response payload size
- Implement GraphQL endpoint for flexible querying
- Add batch processing endpoints
- Implement server-side caching headers

### Monitoring & Observability
- Add request tracing with correlation IDs
- Implement detailed error tracking
- Add performance metrics endpoints
- Implement health check with component status
- Add API usage analytics endpoints

### Documentation
- Add OpenAPI/Swagger specification
- Implement interactive API documentation
- Add code examples in multiple languages
- Create SDK libraries for common programming languages
- Add rate limit documentation per endpoint

### Additional Features
- Add webhook support for task completion notifications
- Implement real-time status updates via WebSocket
- Add export functionality for search results
- Implement comparison endpoints for products across stores
- Add historical price tracking endpoints 