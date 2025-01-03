# ⚠️ Error Handling

<div align="center">

*Error handling documentation for the Rust Web Scraper API*

</div>

## 📑 Table of Contents

- [Overview](#overview)
- [Error Response Format](#error-response-format)
- [Error Categories](#error-categories)
- [Error Codes](#error-codes)
- [Handling Errors](#handling-errors)
- [Best Practices](#best-practices)

## Overview

The API uses a consistent error handling approach across all endpoints. Errors are returned with appropriate HTTP status codes and detailed error messages to help clients handle errors effectively.

## Error Response Format

All API errors follow this standard format:

```json
{
    "status": "error",
    "error": {
        "code": "ERROR_CODE",
        "message": "Human-readable error message",
        "details": {
            "field": "specific_field",
            "reason": "Detailed explanation",
            "suggestion": "How to fix"
        },
        "request_id": "req_123abc",
        "timestamp": "2024-01-20T10:00:00Z"
    }
}
```

## Error Categories

### 1. Client Errors (4xx)

Errors caused by client requests:

#### 400 Bad Request

```json
{
    "status": "error",
    "error": {
        "code": "INVALID_REQUEST",
        "message": "Invalid request parameters",
        "details": {
            "validation_errors": [
                {
                    "field": "url",
                    "error": "Invalid URL format",
                    "value": "not-a-url"
                },
                {
                    "field": "rate_limit",
                    "error": "Must be greater than 0",
                    "value": -1
                }
            ]
        }
    }
}
```

#### 401 Unauthorized

```json
{
    "status": "error",
    "error": {
        "code": "UNAUTHORIZED",
        "message": "Authentication required",
        "details": {
            "reason": "Missing or invalid token",
            "authenticate": "Bearer"
        }
    }
}
```

#### 403 Forbidden

```json
{
    "status": "error",
    "error": {
        "code": "FORBIDDEN",
        "message": "Insufficient permissions",
        "details": {
            "required_role": "admin",
            "current_role": "user",
            "resource": "store_config"
        }
    }
}
```

#### 404 Not Found

```json
{
    "status": "error",
    "error": {
        "code": "NOT_FOUND",
        "message": "Resource not found",
        "details": {
            "resource_type": "Product",
            "resource_id": "prod_123"
        }
    }
}
```

#### 429 Too Many Requests

```json
{
    "status": "error",
    "error": {
        "code": "RATE_LIMIT_EXCEEDED",
        "message": "Too many requests",
        "details": {
            "retry_after": 60,
            "limit": 100,
            "reset_at": "2024-01-20T10:30:00Z"
        }
    }
}
```

### 2. Server Errors (5xx)

Errors occurring on the server side:

#### 500 Internal Server Error

```json
{
    "status": "error",
    "error": {
        "code": "INTERNAL_ERROR",
        "message": "An unexpected error occurred",
        "details": {
            "error_id": "err_123abc",
            "support_contact": "support@example.com"
        }
    }
}
```

#### 503 Service Unavailable

```json
{
    "status": "error",
    "error": {
        "code": "SERVICE_UNAVAILABLE",
        "message": "Service temporarily unavailable",
        "details": {
            "retry_after": 300,
            "maintenance_window": {
                "start": "2024-01-20T10:00:00Z",
                "end": "2024-01-20T11:00:00Z"
            }
        }
    }
}
```

## Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| `INVALID_REQUEST` | 400 | Invalid request parameters |
| `VALIDATION_ERROR` | 400 | Request validation failed |
| `UNAUTHORIZED` | 401 | Authentication required |
| `FORBIDDEN` | 403 | Insufficient permissions |
| `NOT_FOUND` | 404 | Resource not found |
| `CONFLICT` | 409 | Resource conflict |
| `RATE_LIMIT_EXCEEDED` | 429 | Too many requests |
| `INTERNAL_ERROR` | 500 | Server error |
| `SERVICE_UNAVAILABLE` | 503 | Service temporarily unavailable |

## Domain-Specific Errors

### Scraping Errors

```json
{
    "status": "error",
    "error": {
        "code": "SCRAPING_ERROR",
        "message": "Failed to scrape URL",
        "details": {
            "url": "https://example.com",
            "reason": "Site blocking requests",
            "selector_errors": [
                {
                    "selector": ".price",
                    "error": "Selector not found"
                }
            ]
        }
    }
}
```

### Store Configuration Errors

```json
{
    "status": "error",
    "error": {
        "code": "STORE_CONFIG_ERROR",
        "message": "Invalid store configuration",
        "details": {
            "store_id": "store_123",
            "validation_errors": [
                {
                    "field": "selectors.price",
                    "error": "Invalid CSS selector"
                }
            ]
        }
    }
}
```

## Handling Errors

### Client-Side Error Handling

```typescript
async function handleApiError(error: any) {
    if (error.response) {
        const { status, data } = error.response;
        
        switch (status) {
            case 401:
                // Handle authentication errors
                await refreshToken();
                break;
            
            case 429:
                // Handle rate limiting
                const retryAfter = data.error.details.retry_after;
                await delay(retryAfter * 1000);
                break;
            
            case 500:
                // Log server errors
                logger.error({
                    error_id: data.error.details.error_id,
                    message: data.error.message
                });
                break;
        }
    }
}
```

### Retry Strategy

```typescript
async function withRetry<T>(
    operation: () => Promise<T>,
    options: RetryOptions
): Promise<T> {
    const maxAttempts = options.maxAttempts || 3;
    const backoff = options.backoff || 1000;
    
    for (let attempt = 1; attempt <= maxAttempts; attempt++) {
        try {
            return await operation();
        } catch (error) {
            if (attempt === maxAttempts) throw error;
            if (!isRetryable(error)) throw error;
            
            await delay(backoff * Math.pow(2, attempt - 1));
        }
    }
}
```

## Best Practices

### 1. Error Logging

```typescript
function logError(error: ApiError) {
    logger.error({
        code: error.code,
        message: error.message,
        request_id: error.request_id,
        timestamp: error.timestamp,
        details: error.details,
        stack: error.stack
    });
}
```

### 2. Error Recovery

```typescript
async function handleTransientError(error: ApiError) {
    if (isTransient(error)) {
        // Implement exponential backoff
        const backoff = calculateBackoff(error);
        await delay(backoff);
        return retry(operation);
    }
    throw error;
}
```

### 3. User Communication

```typescript
function translateErrorForUser(error: ApiError): UserFriendlyError {
    return {
        message: getLocalizedMessage(error.code),
        action: getSuggestedAction(error.code),
        support_info: error.details.support_contact
    };
}
```

### 4. Monitoring and Alerts

```typescript
function monitorErrors(error: ApiError) {
    // Track error rates
    metrics.increment(`errors.${error.code}`);
    
    // Alert on critical errors
    if (isCritical(error)) {
        alerts.notify({
            level: 'critical',
            error: error,
            service: 'api'
        });
    }
}
``` 