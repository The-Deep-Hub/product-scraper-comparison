# Scraper API Documentation

## Overview

The Scraper API provides endpoints for searching products across multiple home improvement stores, managing scraping tasks, and retrieving results.

## Base URL

```
http://localhost:8080/api
```

## Authentication

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

## Endpoints

### Search

#### POST /protected/search
Start a new search across stores.

**Request**
```json
{
  "query": "hammer",
  "stores": ["Bricodepot", "Bauhaus", "Leroy", "Obramat"],
  "num_products": 10
}
```

**Response (202 Accepted)**
```json
{
  "task_id": "123e4567-e89b-12d3-a456-426614174000",
  "query": "hammer",
  "stores": ["Bricodepot", "Bauhaus", "Leroy", "Obramat"],
  "status": "pending",
  "created_at": "2023-12-23T12:34:56Z"
}
```

#### GET /protected/search/{id}
Get search results by task ID.

**Response (200 OK)**
```json
{
  "task_id": "123e4567-e89b-12d3-a456-426614174000",
  "query": "hammer",
  "stores": ["Bricodepot"],
  "total_products": 10,
  "execution_time": 1.5,
  "results": [
    {
      "store": "Bricodepot",
      "products": [
        {
          "name": "Stanley Hammer",
          "url": "https://...",
          "price": 19.99,
          "original_price": 24.99,
          "image_url": "https://..."
        }
      ],
      "metrics": {
        "execution_time": 1.2,
        "products_found": 10,
        "cache_hit": false,
        "retry_count": 0
      }
    }
  ],
  "created_at": "2023-12-23T12:34:56Z"
}
```

### Tasks

#### GET /protected/tasks
List all tasks with filtering.

**Query Parameters**
- store: Store name (optional)
- status: Task status (optional)
- from_date: Start date (optional)
- to_date: End date (optional)
- page: Page number (default: 1)
- per_page: Items per page (default: 10)

**Response (200 OK)**
```json
{
  "tasks": [
    {
      "id": "123e4567-e89b-12d3-a456-426614174000",
      "store": "Bricodepot",
      "query": "hammer",
      "num_products": 10,
      "priority": 0,
      "retry_count": 0,
      "max_retries": 3,
      "status": "completed",
      "created_at": "2023-12-23T12:34:56Z",
      "updated_at": "2023-12-23T12:35:56Z"
    }
  ],
  "total": 1,
  "page": 1,
  "per_page": 10
}
```

#### POST /protected/tasks/{id}/retry
Retry a failed task.

**Response (202 Accepted)**
```json
{
  "task_id": "123e4567-e89b-12d3-a456-426614174000",
  "status": "pending",
  "message": "Task requeued successfully",
  "retry_count": 1
}
```

### Results

#### GET /protected/results/cached
Get cached results with filtering.

**Query Parameters**
- query: Search query (optional)
- store: Store name (optional)
- min_price: Minimum price (optional)
- max_price: Maximum price (optional)
- sort_by: Sort order (price_asc, price_desc, date)
- page: Page number (default: 1)
- per_page: Items per page (default: 10)

**Response (200 OK)**
```json
{
  "query": "hammer",
  "stores": ["Bricodepot"],
  "total_products": 10,
  "execution_time": 0.1,
  "results": [
    {
      "store": "Bricodepot",
      "products": [
        {
          "name": "Stanley Hammer",
          "url": "https://...",
          "price": 19.99,
          "original_price": 24.99,
          "image_url": "https://..."
        }
      ],
      "metrics": {
        "execution_time": 0.1,
        "products_found": 10,
        "cache_hit": true,
        "retry_count": 0
      }
    }
  ],
  "created_at": "2023-12-23T12:34:56Z"
}
```

#### GET /protected/results/popular
Get popular searches.

**Response (200 OK)**
```json
{
  "searches": [
    {
      "query": "hammer",
      "total_searches": 100,
      "avg_results": 15.5,
      "last_searched_at": "2023-12-23T12:34:56Z"
    }
  ]
}
```

#### GET /protected/results/export/{query}
Export results to CSV.

**Query Parameters**
Same as /results/cached

**Response (200 OK)**
Content-Type: text/csv
Content-Disposition: attachment; filename="search_results_hammer.csv"

## Error Responses

### 400 Bad Request
```json
{
  "error": "Validation failed",
  "details": {
    "query": ["Query must be between 1 and 100 characters"]
  }
}
```

### 401 Unauthorized
```json
{
  "error": "Unauthorized",
  "message": "Invalid or missing token"
}
```

### 404 Not Found
```json
{
  "error": "Not found",
  "message": "Task not found: 123e4567-e89b-12d3-a456-426614174000"
}
```

### 500 Internal Server Error
```json
{
  "error": "Internal server error",
  "message": "An unexpected error occurred"
}
```

## Rate Limiting

The API is rate limited to:
- 100 requests per minute for search endpoints
- 1000 requests per minute for other endpoints

Rate limit headers are included in responses:
```
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 99
X-RateLimit-Reset: 1640217296
``` 