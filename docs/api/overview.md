# 🌐 API Overview

<div align="center">

*API documentation for the Rust Web Scraper*

</div>

## 📑 Table of Contents

- [Introduction](#introduction)
- [Base URL](#base-url)
- [Request Format](#request-format)
- [Response Format](#response-format)
- [Common Headers](#common-headers)
- [Status Codes](#status-codes)
- [API Groups](#api-groups)
- [Try It Out](#try-it-out)
- [Future Improvements](#future-improvements)

## Introduction

The Rust Web Scraper API provides a RESTful interface for managing web scraping operations and product data. The API follows REST principles and uses JSON for request and response payloads.

## Base URL

```
http://localhost:8080
```

All endpoints are relative to this base URL.

## Request Format

### Content Type

All requests should include:
```http
Content-Type: application/json
```

### Request Body

For POST requests, data should be sent as JSON:

```json
{
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
}
```

## Response Format

### Success Response

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

### Error Response

```json
{
    "error": "ErrorType",
    "message": "Detailed error message"
}
```

## Common Headers

| Header | Description |
|--------|-------------|
| `Content-Type` | Application/json |

## Status Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request |
| 404 | Not Found |
| 500 | Internal Server Error |

## API Groups

### Scraping Operations
- Submit scraping requests
- Check task status
- Cancel running tasks
- Retrieve results

## Try It Out

Here are examples of how to interact with the API:

### Using curl

#### Submit a Scraping Request
```bash
curl -X POST http://localhost:8080/search \
  -H "Content-Type: application/json" \
  -d '{
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
  }'
```

#### Check Task Status
```bash
curl -X GET http://localhost:8080/task/{task_id}
```

#### List Active Tasks
```bash
curl -X GET http://localhost:8080/tasks
```

#### Cancel a Task
```bash
curl -X POST http://localhost:8080/task/{task_id}/cancel
```

### Using Python

```python
import requests
import json

# Configuration
BASE_URL = "http://localhost:8080"

# Submit a scraping request
data = {
    "query": "hammer",
    "stores": ["Bricodepot", "Leroy"],
    "num_products": 50
}

response = requests.post(
    f"{BASE_URL}/search",
    headers={"Content-Type": "application/json"},
    json=data
)

print(json.dumps(response.json(), indent=2))
```

### Using JavaScript

```javascript
const BASE_URL = 'http://localhost:8080';

// Submit a scraping request
async function submitScrapeRequest() {
  const response = await fetch(`${BASE_URL}/search`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      query: 'hammer',
      stores: ['Bricodepot', 'Leroy'],
      num_products: 50
    })
  });

  const data = await response.json();
  console.log(data);
}

// Check task status
async function checkTaskStatus(taskId) {
  const response = await fetch(`${BASE_URL}/task/${taskId}`);
  const data = await response.json();
  console.log(data);
}
```

## Future Improvements

### Authentication (Coming Soon)
- User registration and login
- JWT token-based authentication
- Role-based access control
- API key support for service accounts
- Token refresh mechanism

### Additional Features Planned
- Rate limiting
- Request validation
- Response compression
- API versioning
- Detailed error reporting 