# Rate Limiting System Documentation

## Overview
The rate limiting system protects the API from abuse by limiting the number of requests that can be made within a specified time window. It uses Redis to track request counts per IP address and provides configurable limits for different endpoints.

## Features
- IP-based rate limiting
- Configurable time windows and request limits
- Redis-backed storage for distributed deployments
- Automatic cleanup of expired rate limit data
- Custom response headers for rate limit information
- Different rate limits for various endpoints

## Configuration

### Environment Variables
```env
# Redis Configuration
REDIS_HOST=localhost
REDIS_PORT=6379
REDIS_PASSWORD=your_redis_password_here

# Rate Limit Configuration
RATE_LIMIT_WINDOW=60  # Time window in seconds
RATE_LIMIT_MAX_REQUESTS=100  # Maximum requests per window
```

### Rate Limit Settings
Default rate limits for different endpoints:

```rust
// In src/api/middleware/rate_limit.rs
pub const DEFAULT_RATE_LIMIT: RateLimitConfig = RateLimitConfig {
    window_secs: 60,
    max_requests: 100,
};

pub const AUTH_RATE_LIMIT: RateLimitConfig = RateLimitConfig {
    window_secs: 60,
    max_requests: 5,
};

pub const PASSWORD_RESET_RATE_LIMIT: RateLimitConfig = RateLimitConfig {
    window_secs: 3600,  // 1 hour
    max_requests: 3,
};
```

## Implementation Details

### Rate Limit Key Format
```
rate_limit:{ip_address}:{endpoint}
```

Example:
```
rate_limit:192.168.1.1:/api/auth/login
```

### Redis Data Structure
- Key: Rate limit key (as shown above)
- Value: Request count
- TTL: Set to the rate limit window duration

### Response Headers
The system adds the following headers to responses:
```
X-RateLimit-Limit: Maximum requests allowed in the window
X-RateLimit-Remaining: Remaining requests in the current window
X-RateLimit-Reset: Timestamp when the current window expires
```

## Usage

### Basic Implementation
```rust
use actix_web::{web, App, HttpServer};
use crate::api::middleware::rate_limit::{RateLimiter, RateLimitConfig};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let rate_limiter = RateLimiter::new(
        redis_client,
        RateLimitConfig {
            window_secs: 60,
            max_requests: 100,
        },
    );

    HttpServer::new(move || {
        App::new()
            .wrap(rate_limiter.clone())
            // ... other middleware and routes
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

### Custom Rate Limits for Specific Routes
```rust
use actix_web::{web, Scope};
use crate::api::middleware::rate_limit::{RateLimiter, AUTH_RATE_LIMIT};

pub fn auth_routes() -> Scope {
    let auth_rate_limiter = RateLimiter::new(
        redis_client,
        AUTH_RATE_LIMIT,
    );

    web::scope("/auth")
        .wrap(auth_rate_limiter)
        .route("/login", web::post().to(login_handler))
        .route("/register", web::post().to(register_handler))
}
```

## Error Handling

### Rate Limit Exceeded Response
When a rate limit is exceeded, the system returns:

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1633027200

{
    "error": {
        "code": "RATE_LIMIT_EXCEEDED",
        "message": "Too many requests. Please try again later.",
        "retry_after": 3600
    }
}
```

### Error Scenarios
1. Redis Connection Failure
   - Falls back to allowing requests
   - Logs error for monitoring
   - Sets specific error header

2. Invalid IP Address
   - Returns 400 Bad Request
   - Logs attempt for security monitoring

3. Configuration Errors
   - Logs error during startup
   - Uses default configuration as fallback

## Monitoring and Logging

### Logged Events
- Rate limit exceeded events
- Redis connection failures
- Configuration issues
- Suspicious patterns (multiple IPs hitting limits)

### Metrics
The system tracks the following metrics:
- Request counts per endpoint
- Rate limit exceeded counts
- Redis connection status
- Average request rate per IP

### Log Format
```json
{
    "timestamp": "2023-10-01T12:00:00Z",
    "level": "INFO",
    "event": "rate_limit_exceeded",
    "ip": "192.168.1.1",
    "endpoint": "/api/auth/login",
    "request_count": 6,
    "limit": 5,
    "window": 60
}
```

## Testing

### Unit Tests
```rust
#[test]
fn test_rate_limit_basic() {
    // Test basic rate limiting functionality
}

#[test]
fn test_rate_limit_window_reset() {
    // Test rate limit window reset
}

#[test]
fn test_rate_limit_multiple_ips() {
    // Test rate limiting for multiple IPs
}
```

### Integration Tests
```rust
#[actix_rt::test]
async fn test_rate_limit_integration() {
    // Test rate limiting in a running application
}
```

## Best Practices

1. **Configuration**
   - Use environment variables for configuration
   - Set reasonable defaults
   - Document rate limit values

2. **Security**
   - Validate IP addresses
   - Use secure Redis connection
   - Monitor for abuse patterns

3. **Performance**
   - Use Redis pipeline for multiple operations
   - Set appropriate key expiration
   - Monitor Redis memory usage

4. **Maintenance**
   - Regular monitoring of rate limit events
   - Periodic review of rate limit values
   - Cleanup of expired data

## Dependencies
```toml
[dependencies]
redis = "0.21"
actix-web = "4.0"
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }
``` 