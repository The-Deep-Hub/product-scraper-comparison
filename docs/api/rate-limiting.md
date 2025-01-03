# 🚦 Rate Limiting

<div align="center">

*Rate limiting documentation for the Rust Web Scraper API*

</div>

## 📑 Table of Contents

- [Overview](#overview)
- [Rate Limit Rules](#rate-limit-rules)
- [Headers](#headers)
- [Handling Rate Limits](#handling-rate-limits)
- [Best Practices](#best-practices)

## Overview

The API implements rate limiting to ensure fair usage and system stability. Rate limits are applied based on authentication method, endpoint, and resource type.

## Rate Limit Rules

### Authentication-Based Limits

| Authentication Method | Rate Limit | Window |
|----------------------|------------|---------|
| No Authentication | 30 | per minute |
| API Key | 1000 | per hour |
| OAuth2 Token | 2000 | per hour |
| Enterprise Token | 5000 | per hour |

### Endpoint-Specific Limits

| Endpoint | Method | Rate Limit | Window |
|----------|---------|------------|---------|
| `/scrapes` | POST | 60 | per hour |
| `/products` | GET | 1000 | per hour |
| `/stores` | GET | 100 | per hour |
| `/stores` | POST | 10 | per hour |

### Resource-Based Limits

| Resource | Limit | Window |
|----------|-------|--------|
| Scraping Tasks | 100 | per day |
| Store Configurations | 50 | per day |
| Bulk Operations | 10 | per hour |

## Headers

### Request Headers

```http
X-Rate-Limit-Strategy: gradual
X-Rate-Limit-Priority: high
```

### Response Headers

```http
X-Rate-Limit-Limit: 1000
X-Rate-Limit-Remaining: 999
X-Rate-Limit-Reset: 1706659200
X-Rate-Limit-Used: 1
```

## Rate Limit Response

When rate limit is exceeded:

```json
{
    "status": "error",
    "error": {
        "code": "RATE_LIMIT_EXCEEDED",
        "message": "Rate limit exceeded",
        "details": {
            "limit": 1000,
            "remaining": 0,
            "reset_at": "2024-01-20T11:00:00Z",
            "retry_after": 3600,
            "scope": "hourly"
        }
    }
}
```

## Rate Limit Algorithms

### 1. Token Bucket Algorithm

```rust
pub struct TokenBucket {
    capacity: u32,
    tokens: AtomicU32,
    refill_rate: f64,
    last_refill: AtomicI64,
}

impl TokenBucket {
    pub fn try_acquire(&self, tokens: u32) -> bool {
        let now = Utc::now().timestamp();
        self.refill(now);
        
        let current = self.tokens.load(Ordering::Relaxed);
        if current >= tokens {
            self.tokens.fetch_sub(tokens, Ordering::Relaxed);
            true
        } else {
            false
        }
    }
    
    fn refill(&self, now: i64) {
        let last = self.last_refill.load(Ordering::Relaxed);
        let elapsed = (now - last) as f64;
        let new_tokens = (elapsed * self.refill_rate) as u32;
        
        if new_tokens > 0 {
            self.tokens.fetch_add(new_tokens, Ordering::Relaxed);
            self.last_refill.store(now, Ordering::Relaxed);
        }
    }
}
```

### 2. Sliding Window Algorithm

```rust
pub struct SlidingWindow {
    window_size: Duration,
    max_requests: u32,
    requests: Arc<RwLock<VecDeque<DateTime<Utc>>>>,
}

impl SlidingWindow {
    pub async fn check_rate_limit(&self) -> Result<(), RateLimitError> {
        let now = Utc::now();
        let window_start = now - self.window_size;
        
        let mut requests = self.requests.write().await;
        
        // Remove old requests
        while let Some(timestamp) = requests.front() {
            if *timestamp < window_start {
                requests.pop_front();
            } else {
                break;
            }
        }
        
        // Check if limit is exceeded
        if requests.len() >= self.max_requests as usize {
            return Err(RateLimitError::new(
                self.window_size,
                requests.len() as u32,
                self.max_requests,
            ));
        }
        
        // Add new request
        requests.push_back(now);
        Ok(())
    }
}
```

## Handling Rate Limits

### Client-Side Implementation

```typescript
class RateLimitHandler {
    private buckets: Map<string, TokenBucket>;
    
    async makeRequest(endpoint: string): Promise<Response> {
        const bucket = this.getBucket(endpoint);
        
        if (!bucket.tryAcquire()) {
            const retryAfter = bucket.getRetryAfter();
            await this.delay(retryAfter);
        }
        
        try {
            return await this.executeRequest(endpoint);
        } catch (error) {
            if (error.status === 429) {
                await this.handleRateLimit(error);
                return this.makeRequest(endpoint);
            }
            throw error;
        }
    }
    
    private async handleRateLimit(error: RateLimitError) {
        const retryAfter = error.details.retry_after;
        await this.delay(retryAfter * 1000);
    }
}
```

### Retry Strategy

```typescript
const retryStrategy = {
    maxAttempts: 3,
    backoff: {
        initial: 1000,
        multiplier: 2,
        maxDelay: 60000
    },
    shouldRetry: (error: any) => {
        return error.status === 429 || 
               (error.status >= 500 && error.status <= 599);
    }
};

async function withRetry<T>(
    operation: () => Promise<T>
): Promise<T> {
    let attempt = 0;
    let delay = retryStrategy.backoff.initial;
    
    while (attempt < retryStrategy.maxAttempts) {
        try {
            return await operation();
        } catch (error) {
            attempt++;
            
            if (!retryStrategy.shouldRetry(error) || 
                attempt === retryStrategy.maxAttempts) {
                throw error;
            }
            
            await sleep(Math.min(
                delay * Math.pow(retryStrategy.backoff.multiplier, attempt - 1),
                retryStrategy.backoff.maxDelay
            ));
        }
    }
}
```

## Best Practices

### 1. Rate Limit Monitoring

```typescript
class RateLimitMonitor {
    private metrics: MetricsClient;
    
    constructor(metrics: MetricsClient) {
        this.metrics = metrics;
    }
    
    trackRateLimit(response: Response) {
        const remaining = parseInt(
            response.headers.get('X-Rate-Limit-Remaining')
        );
        const limit = parseInt(
            response.headers.get('X-Rate-Limit-Limit')
        );
        
        this.metrics.gauge('rate_limit.remaining', remaining);
        this.metrics.gauge('rate_limit.usage_percent', 
            ((limit - remaining) / limit) * 100
        );
    }
    
    alertOnThreshold(usagePercent: number) {
        if (usagePercent > 80) {
            this.metrics.alert('rate_limit.high_usage', {
                usage: usagePercent,
                threshold: 80
            });
        }
    }
}
```

### 2. Request Optimization

```typescript
class RequestOptimizer {
    private cache: Cache;
    private batcher: RequestBatcher;
    
    async optimizeRequests(requests: Request[]): Promise<Response[]> {
        // Check cache first
        const cachedResponses = await this.checkCache(requests);
        const uncachedRequests = this.filterUncached(requests, cachedResponses);
        
        // Batch remaining requests
        if (uncachedRequests.length > 0) {
            const batchedResponses = await this.batcher.batch(uncachedRequests);
            await this.cache.store(batchedResponses);
            return [...cachedResponses, ...batchedResponses];
        }
        
        return cachedResponses;
    }
}
```

### 3. Rate Limit Budgeting

```typescript
class RateLimitBudget {
    private budget: number;
    private resetTime: Date;
    
    async allocateBudget(operation: Operation): Promise<boolean> {
        const cost = this.calculateCost(operation);
        
        if (this.budget >= cost) {
            this.budget -= cost;
            return true;
        }
        
        const timeToReset = this.resetTime.getTime() - Date.now();
        if (timeToReset <= 0) {
            await this.resetBudget();
            return this.allocateBudget(operation);
        }
        
        return false;
    }
}
```

### 4. Adaptive Rate Limiting

```typescript
class AdaptiveRateLimiter {
    private successRate: number;
    private currentLimit: number;
    
    adjustLimit(response: Response) {
        const latency = this.measureLatency(response);
        const success = response.status < 500;
        
        this.updateMetrics(latency, success);
        
        if (this.shouldAdjustLimit()) {
            this.currentLimit = this.calculateNewLimit();
        }
    }
    
    private calculateNewLimit(): number {
        if (this.successRate < 0.9) {
            return this.currentLimit * 0.8; // Reduce by 20%
        } else if (this.successRate > 0.95) {
            return this.currentLimit * 1.1; // Increase by 10%
        }
        return this.currentLimit;
    }
}
``` 