# 🔐 Authentication

<div align="center">

*Authentication documentation for the Rust Web Scraper API*

</div>

## 📑 Table of Contents

- [Overview](#overview)
- [Authentication Methods](#authentication-methods)
- [Token Management](#token-management)
- [Security Best Practices](#security-best-practices)
- [Examples](#examples)

## Overview

The Rust Web Scraper API uses a robust authentication system to ensure secure access to resources. We support multiple authentication methods to accommodate different use cases and security requirements.

## Authentication Methods

### 1. Bearer Token Authentication

The primary authentication method using JWT tokens:

```http
GET /v1/products
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

Token Structure:
```json
{
    "sub": "user_123",
    "exp": 1706659200,
    "iat": 1706572800,
    "scope": ["read:products", "write:scrapes"]
}
```

### 2. API Key Authentication

For service-to-service communication:

```http
GET /v1/products
X-API-Key: sk_live_123456789abcdef
```

### 3. OAuth2 Authentication

For third-party integrations:

```http
POST /oauth/token
Content-Type: application/x-www-form-urlencoded

grant_type=authorization_code
&code=AUTH_CODE
&client_id=CLIENT_ID
&client_secret=CLIENT_SECRET
&redirect_uri=https://your-app.com/callback
```

## Token Management

### Obtaining Tokens

1. **Login Endpoint**
```http
POST /v1/auth/login
Content-Type: application/json

{
    "email": "user@example.com",
    "password": "secure_password"
}
```

Response:
```json
{
    "status": "success",
    "data": {
        "access_token": "eyJhbGciOiJIUzI1NiI...",
        "refresh_token": "eyJhbGciOiJIUzI1NiI...",
        "token_type": "Bearer",
        "expires_in": 3600
    }
}
```

### Refreshing Tokens

```http
POST /v1/auth/refresh
Content-Type: application/json
Authorization: Bearer <refresh_token>

{
    "refresh_token": "eyJhbGciOiJIUzI1NiI..."
}
```

### Revoking Tokens

```http
POST /v1/auth/revoke
Content-Type: application/json
Authorization: Bearer <access_token>

{
    "token": "eyJhbGciOiJIUzI1NiI..."
}
```

## Access Control

### Role-Based Access Control (RBAC)

Available roles:
- `admin`: Full system access
- `operator`: Manage scraping operations
- `reader`: Read-only access
- `writer`: Read and write access

Example role claim in JWT:
```json
{
    "sub": "user_123",
    "role": "operator",
    "permissions": [
        "scrapes:create",
        "scrapes:read",
        "products:read"
    ]
}
```

### Permission Scopes

| Scope | Description |
|-------|-------------|
| `scrapes:create` | Create scraping tasks |
| `scrapes:read` | View scraping tasks |
| `products:read` | View product data |
| `products:write` | Modify product data |
| `stores:manage` | Manage store configurations |

## Security Best Practices

### 1. Token Security

- Store tokens securely
- Never expose tokens in URLs
- Use HTTPS for all requests
- Implement token rotation

### 2. API Key Management

- Rotate API keys regularly
- Use environment-specific keys
- Monitor API key usage
- Implement key revocation

### 3. Error Handling

Authentication error responses:

```json
{
    "status": "error",
    "error": {
        "code": "UNAUTHORIZED",
        "message": "Invalid or expired token",
        "details": {
            "reason": "Token expired",
            "expired_at": "2024-01-20T10:00:00Z"
        }
    }
}
```

## Examples

### 1. Client Authentication Flow

```typescript
async function authenticate() {
    const response = await fetch('https://api.scraper.example.com/v1/auth/login', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({
            email: 'user@example.com',
            password: 'secure_password'
        })
    });

    const { access_token, refresh_token } = await response.json();
    return { access_token, refresh_token };
}
```

### 2. Token Refresh Flow

```typescript
async function refreshToken(refresh_token: string) {
    const response = await fetch('https://api.scraper.example.com/v1/auth/refresh', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': `Bearer ${refresh_token}`
        }
    });

    const { access_token, refresh_token: new_refresh_token } = await response.json();
    return { access_token, refresh_token: new_refresh_token };
}
```

### 3. API Key Usage

```python
import requests

def get_products(api_key: str):
    response = requests.get(
        'https://api.scraper.example.com/v1/products',
        headers={
            'X-API-Key': api_key
        }
    )
    return response.json()
```

## Rate Limiting

Authentication-specific rate limits:

| Auth Method | Rate Limit |
|-------------|------------|
| JWT Token | 1000 requests/hour |
| API Key | 5000 requests/hour |
| OAuth2 | 2000 requests/hour |

Rate limit headers:
```http
X-Rate-Limit-Limit: 1000
X-Rate-Limit-Remaining: 999
X-Rate-Limit-Reset: 1706659200
```

## Monitoring and Security

### 1. Audit Logging

All authentication events are logged:

```json
{
    "event": "token_issued",
    "user_id": "user_123",
    "timestamp": "2024-01-20T10:00:00Z",
    "ip_address": "192.168.1.1",
    "user_agent": "Mozilla/5.0...",
    "success": true
}
```

### 2. Security Alerts

Suspicious activity triggers alerts:
- Multiple failed login attempts
- Token usage from new IP addresses
- Unusual request patterns
- API key misuse

### 3. Compliance

- GDPR compliance for EU users
- Data encryption in transit and at rest
- Regular security audits
- Incident response procedures 