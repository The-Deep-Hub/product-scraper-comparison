# 🧪 Cache Testing Strategy

<div align="center">

*A comprehensive testing approach for the Redis caching system*

</div>

## 📑 Table of Contents

<details open>
<summary><strong>Strategy & Approach</strong></summary>

- [Overview](#overview)
- [Testing Approach](#testing-approach)
- [Why This Approach?](#why-this-approach)
</details>

<details open>
<summary><strong>Implementation</strong></summary>

- [Test Levels](#test-levels)
  - [Unit Tests](#1-unit-tests)
  - [Integration Tests](#2-integration-tests)
  - [Performance Tests](#3-performance-tests)
- [Test Infrastructure](#test-infrastructure)
  - [Test Containers](#1-test-containers)
  - [Test Data Generators](#2-test-data-generators)
</details>

<details open>
<summary><strong>Test Categories & Execution</strong></summary>

- [Test Categories](#test-categories)
  - [Functional Tests](#1-functional-tests)
  - [Error Handling Tests](#2-error-handling-tests)
  - [Concurrency Tests](#3-concurrency-tests)
- [Monitoring & Metrics](#monitoring--metrics-in-tests)
- [Test Environment Setup](#test-environment-setup)
</details>

<details open>
<summary><strong>Guidelines & Integration</strong></summary>

- [Best Practices](#best-practices)
- [CI/CD Integration](#cicd-integration)
</details>

---

## Overview

The testing strategy for the Redis caching system follows a comprehensive approach that combines mocking, integration testing, and performance testing. This multi-layered strategy ensures both the correctness of the caching logic and its real-world performance characteristics.

### Testing Approach

1. **Unit Testing with Mocks**:
   - We use `mockall` to mock the Redis connection and configuration
   - This allows testing the cache logic in isolation
   - Ensures correct handling of cache keys, serialization, and error cases
   - Fast and reliable tests that don't require external dependencies

2. **Integration Testing with Test Containers**:
   - Uses real Redis instances in Docker containers
   - Tests actual Redis interactions and data persistence
   - Verifies connection pooling and concurrent access
   - Ensures compatibility with the Redis version used in production

3. **Performance Testing with Metrics**:
   - Measures real-world performance characteristics
   - Tests cache under load with concurrent access
   - Verifies memory usage and connection pool efficiency
   - Establishes performance baselines and SLAs

### Why This Approach?

1. **Mocking for Unit Tests**:
   - Fast test execution
   - Predictable behavior
   - Easy to test error scenarios
   - No external dependencies

2. **Real Redis for Integration**:
   - Catches real-world issues
   - Tests actual Redis commands
   - Verifies configuration
   - Tests data persistence

3. **Performance Metrics**:
   - Ensures scalability
   - Identifies bottlenecks
   - Validates production readiness
   - Maintains performance standards

## Test Levels

### 1. Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_cache_products() {
        let mock_config = MockCacheConfig::new();
        let adapter = RedisAdapter::new(Box::new(mock_config)).await.unwrap();
        
        let products = vec![
            Product::new("Test Product", 10.0, Store::Bricodepot),
            Product::new("Test Product 2", 20.0, Store::Bauhaus),
        ];
        
        assert!(adapter.cache_products("test_query", &products).await.is_ok());
    }
}
```

Key Areas:
- Cache key generation
- Serialization/deserialization
- Error handling
- TTL management

### 2. Integration Tests

```rust
#[tokio::test]
async fn test_cache_integration() {
    let test_container = TestContainer::new("redis:7.2-alpine");
    let config = RedisCacheConfig::new(test_container.connection_string());
    let adapter = RedisAdapter::new(Box::new(config)).await.unwrap();
    
    // Test full cache flow
    let products = generate_test_products();
    adapter.cache_products("test", &products).await.unwrap();
    
    let cached = adapter.get_products("test").await.unwrap();
    assert_eq!(products, cached);
}
```

Key Scenarios:
- Cache hit/miss scenarios
- Concurrent access
- Connection pool behavior
- Data consistency

### 3. Performance Tests

```rust
#[tokio::test]
async fn test_cache_performance() {
    let adapter = setup_test_cache().await;
    let start = Instant::now();
    
    for i in 0..1000 {
        adapter.get_products(&format!("query_{}", i)).await.unwrap();
    }
    
    assert!(start.elapsed() < Duration::from_secs(1));
}
```

Metrics to Test:
- Response times
- Throughput
- Memory usage
- Connection pool efficiency

## Test Infrastructure

### 1. Test Containers

```rust
pub struct RedisCacheTestContainer {
    container: Container<Redis>,
    connection_string: String,
}

impl RedisCacheTestContainer {
    pub async fn new() -> Self {
        let container = Container::new("redis:7.2-alpine")
            .with_env("REDIS_PASSWORD", "test_password")
            .await?;
        
        Self {
            connection_string: container.connection_string(),
            container,
        }
    }
}
```

### 2. Test Data Generators

```rust
pub fn generate_test_products() -> Vec<Product> {
    vec![
        Product::builder()
            .name("Test Product")
            .price(10.0)
            .store(Store::Bricodepot)
            .build(),
        Product::builder()
            .name("Another Product")
            .price(20.0)
            .store(Store::Bauhaus)
            .build(),
    ]
}
```

## Test Categories

### 1. Functional Tests

```rust
#[tokio::test]
async fn test_cache_operations() {
    let suite = vec![
        TestCase::new("Basic Cache Hit")
            .with_input(products.clone())
            .expect_output(products.clone()),
        TestCase::new("Cache Miss")
            .with_input(vec![])
            .expect_output(vec![]),
        TestCase::new("Cache Update")
            .with_input(updated_products)
            .expect_output(updated_products),
    ];
    
    run_test_suite(suite).await;
}
```

### 2. Error Handling Tests

```rust
#[tokio::test]
async fn test_error_scenarios() {
    let cases = vec![
        ("connection_failure", expect_connection_error()),
        ("invalid_data", expect_serialization_error()),
        ("timeout", expect_timeout_error()),
    ];
    
    for (scenario, expected) in cases {
        assert_matches!(
            test_error_scenario(scenario).await,
            expected
        );
    }
}
```

### 3. Concurrency Tests

```rust
#[tokio::test]
async fn test_concurrent_access() {
    let adapter = setup_test_cache().await;
    let handles: Vec<_> = (0..100)
        .map(|i| {
            let adapter = adapter.clone();
            tokio::spawn(async move {
                adapter.cache_products(
                    &format!("key_{}", i),
                    &generate_test_products()
                ).await
            })
        })
        .collect();
    
    for handle in handles {
        handle.await.unwrap().unwrap();
    }
}
```

## Monitoring & Metrics in Tests

```rust
#[derive(Default)]
struct CacheTestMetrics {
    hits: AtomicUsize,
    misses: AtomicUsize,
    errors: AtomicUsize,
    latencies: Vec<Duration>,
}

impl CacheTestMetrics {
    fn record_operation(&self, start: Instant, result: Result<(), Error>) {
        let duration = start.elapsed();
        match result {
            Ok(_) => self.hits.fetch_add(1, Ordering::SeqCst),
            Err(_) => self.errors.fetch_add(1, Ordering::SeqCst),
        };
        self.latencies.push(duration);
    }
}
```

## Test Environment Setup

```rust
async fn setup_test_environment() -> TestEnvironment {
    TestEnvironment {
        redis: RedisCacheTestContainer::new().await,
        metrics: Arc::new(CacheTestMetrics::default()),
        logger: setup_test_logger(),
    }
}
```

## Best Practices

1. **Test Isolation**
   - Use fresh Redis instance for each test
   - Clear cache between tests
   - Avoid test interdependencies

2. **Test Data Management**
   - Use realistic test data
   - Cover edge cases
   - Test with various data sizes

3. **Error Simulation**
   - Network failures
   - Redis server crashes
   - Memory limits
   - Connection pool exhaustion

4. **Performance Testing**
   - Establish baselines
   - Monitor resource usage
   - Test under load
   - Measure latencies

## CI/CD Integration

```yaml
cache-tests:
  runs-on: ubuntu-latest
  services:
    redis:
      image: redis:7.2-alpine
      ports:
        - 6379:6379
  steps:
    - uses: actions/checkout@v2
    - name: Run cache tests
      run: cargo test --package cache
``` 