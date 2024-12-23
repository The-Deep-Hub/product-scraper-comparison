# Redis Configuration

This document describes the Redis setup and configuration for the Rust Scraper project.

## Overview

The project uses Redis 7.2-alpine as a caching layer, running in a Docker container. The setup includes:
- Password authentication
- Persistence configuration
- Memory management
- Integration tests
- Environment-based configuration

## Configuration

### Environment Variables

The Redis configuration uses environment variables for sensitive data. Add to your `.env` file:

```env
# Redis Configuration
REDIS_PASSWORD=your_redis_password_here
REDIS_HOST=localhost
REDIS_PORT=6379
```

> ⚠️ Never commit the `.env` file. Use `.env.example` as a template.

### Docker Configuration

Redis runs in a Docker container defined in `docker-compose.yml`:

```yaml
services:
  redis:
    image: redis:7.2-alpine
    container_name: rust_scraper_redis
    command: redis-server --requirepass ${REDIS_PASSWORD}
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
      - ./docker/redis/redis.conf:/usr/local/etc/redis/redis.conf:ro
    environment:
      - REDIS_PASSWORD=${REDIS_PASSWORD}
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 5s
      timeout: 5s
      retries: 5
```

### Redis Configuration File

The Redis configuration (`redis.conf`) includes:

```conf
# Memory Management
maxmemory 256mb
maxmemory-policy allkeys-lru

# Persistence
save 900 1
save 300 10
save 60 10000

# Security
rename-command FLUSHDB ""
rename-command FLUSHALL ""
rename-command DEBUG ""
```

## Cache Structure

### API Response Caching
- Key Format: `api:endpoint:params`
- Value: JSON string
- Default TTL: Configurable per endpoint
- Example:
  ```
  Key: api:products:category=electronics
  Value: {"products": [...]}
  ```

### Cache Management
- LRU (Least Recently Used) eviction policy
- 256MB memory limit
- Automatic persistence
- Key expiration monitoring

## Testing

Integration tests are organized under `tests/infrastructure/redis/`:

```
tests/infrastructure/redis/
├── helpers.rs       # Shared test utilities
├── connection_test.rs # Connection tests
└── operations_test.rs # Redis operations tests
```

Run tests with:
```bash
cargo test
```

### Test Coverage
- Connection testing (sync and async)
- Basic operations (String, List, Hash, Set)
- Expiration testing
- Data cleanup

## Monitoring

### Redis Commander (Web UI)
To access Redis through a web interface:

```yaml
# Add to docker-compose.yml
services:
  redis-commander:
    image: rediscommander/redis-commander:latest
    container_name: rust_scraper_redis_commander
    ports:
      - "8081:8081"
    environment:
      - REDIS_HOSTS=local:redis:6379:0:${REDIS_PASSWORD}
    depends_on:
      - redis
```

Access the UI at: http://localhost:8081

### Redis CLI
Connect to Redis using CLI:
```bash
docker exec -it rust_scraper_redis redis-cli -a ${REDIS_PASSWORD}
```

## Development Setup

1. Copy environment template:
   ```bash
   cp .env.example .env
   ```

2. Update Redis password in `.env`

3. Start Redis:
   ```bash
   docker-compose up -d
   ```

4. Verify setup:
   ```bash
   docker exec -it rust_scraper_redis redis-cli -a ${REDIS_PASSWORD} ping
   ```

## Best Practices

### Caching Strategies
1. **TTL Setting**:
   - Short-lived data: 5-15 minutes
   - Semi-static data: 1-6 hours
   - Static data: 24 hours

2. **Key Naming**:
   - Use colon `:` as namespace separator
   - Include version in key if needed
   - Example: `v1:api:products:electronics`

3. **Error Handling**:
   - Cache misses should fallback to source
   - Implement circuit breaker for Redis failures
   - Log cache hit/miss ratios

### Memory Management
- Monitor memory usage
- Set appropriate maxmemory limit
- Use LRU eviction policy
- Implement key expiration

## Future Improvements

1. Add Redis Cluster configuration
2. Implement cache warming strategies
3. Add Prometheus metrics
4. Configure backup automation
5. Add cache analytics 