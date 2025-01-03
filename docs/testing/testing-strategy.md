# 🧪 Testing Strategy

<div align="center">

*Testing strategy documentation for the Rust Web Scraper*

</div>

## 📑 Table of Contents

- [Overview](#overview)
- [Test Categories](#test-categories)
- [Component Testing](#component-testing)
- [Integration Testing](#integration-testing)
- [Performance Testing](#performance-testing)
- [Test Infrastructure](#test-infrastructure)

## Overview

Our testing strategy follows a comprehensive approach to ensure the reliability and correctness of all system components. We employ multiple testing levels and methodologies to cover different aspects of the system.

## Test Categories

### 1. Unit Tests

- Test individual components in isolation
- Mock external dependencies
- Focus on business logic and edge cases
- Quick feedback loop for developers

### 2. Integration Tests

- Test component interactions
- Use test containers for external services
- End-to-end workflows
- API contract validation

### 3. Performance Tests

- Load testing
- Stress testing
- Scalability validation
- Resource utilization monitoring

## Component Testing

### Domain Layer Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_product_validation() {
        let product = Product {
            name: "Test Product".to_string(),
            price: Price::new(100, "USD"),
            // ... other fields
        };
        
        assert!(product.validate().is_ok());
    }

    #[test]
    fn test_product_price_validation() {
        let invalid_product = Product {
            name: "Test Product".to_string(),
            price: Price::new(-100, "USD"),
            // ... other fields
        };
        
        assert!(matches!(
            invalid_product.validate(),
            Err(ValidationError::InvalidPrice)
        ));
    }
}
```

### Port Testing

```rust
#[cfg(test)]
mod cache_port_tests {
    use super::*;
    use mockall::automock;

    #[automock]
    trait CachePort {
        async fn get(&self, key: String) -> Result<Option<Value>, Error>;
        async fn set(&self, key: String, value: Value, ttl: Duration) -> Result<(), Error>;
    }

    #[tokio::test]
    async fn test_cache_operations() {
        let mut mock_cache = MockCachePort::new();
        
        mock_cache
            .expect_get()
            .with(eq("test_key"))
            .returning(|_| Ok(Some("test_value".into())));

        let result = mock_cache.get("test_key".to_string()).await;
        assert!(result.is_ok());
    }
}
```

### Adapter Testing

```rust
#[cfg(test)]
mod redis_adapter_tests {
    use super::*;
    use testcontainers::*;

    #[tokio::test]
    async fn test_redis_adapter() {
        let docker = clients::Cli::default();
        let redis_container = docker.run(images::redis::Redis::default());
        let port = redis_container.get_host_port_ipv4(6379);
        
        let adapter = RedisAdapter::new(format!("redis://localhost:{}", port));
        
        let result = adapter
            .set("test_key", "test_value", Duration::from_secs(60))
            .await;
        assert!(result.is_ok());
        
        let value = adapter.get("test_key").await.unwrap();
        assert_eq!(value, Some("test_value".to_string()));
    }
}
```

## Integration Testing

### API Integration Tests

```rust
#[cfg(test)]
mod api_tests {
    use super::*;
    use actix_web::test;

    #[actix_web::test]
    async fn test_product_creation_flow() {
        let app = test::init_service(
            App::new()
                .service(web::resource("/products").route(web::post().to(create_product)))
        ).await;

        let req = test::TestRequest::post()
            .uri("/products")
            .set_json(&product_data)
            .to_request();
            
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }
}
```

### Event System Integration

```rust
#[cfg(test)]
mod event_system_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_flow() {
        let event_bus = setup_test_event_bus().await;
        let handler = setup_test_handler().await;
        
        event_bus.subscribe(Box::new(handler)).await?;
        
        let event = ProductCreated(test_product());
        event_bus.publish(event).await?;
        
        // Wait for event processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Verify handler effects
        assert!(verify_handler_effects().await);
    }
}
```

## Performance Testing

### Load Testing

```rust
#[cfg(test)]
mod load_tests {
    use super::*;
    use goose::prelude::*;

    #[tokio::test]
    async fn test_api_load() {
        let test = Transaction::new(GooseAttack::initialize()?
            .register_scenario(scenario!("LoadTest")
                .register_transaction(transaction!(api_call))
            ))
            .set_users(GooseUsers::new(100, Duration::from_secs(10)))
            .execute()
            .await?;
            
        assert!(test.duration() < Duration::from_secs(30));
        assert!(test.success_count > test.fail_count * 10); // 90% success rate
    }
}
```

### Stress Testing

```rust
#[cfg(test)]
mod stress_tests {
    use super::*;

    #[tokio::test]
    async fn test_system_under_stress() {
        let metrics = run_stress_test(StressConfig {
            duration: Duration::from_minutes(10),
            concurrent_users: 1000,
            ramp_up_time: Duration::from_minutes(2),
        }).await?;
        
        assert!(metrics.response_time_p95 < Duration::from_secs(1));
        assert!(metrics.error_rate < 0.01); // Less than 1% errors
    }
}
```

## Test Infrastructure

### Test Containers

```rust
pub struct TestContainers {
    pub redis: Container<Redis>,
    pub mongodb: Container<Mongodb>,
    pub rabbitmq: Container<RabbitMq>,
}

impl TestContainers {
    pub async fn new() -> Self {
        let docker = clients::Cli::default();
        
        let redis = docker.run(images::redis::Redis::default());
        let mongodb = docker.run(images::mongo::Mongo::default());
        let rabbitmq = docker.run(images::rabbitmq::RabbitMq::default());
        
        Self {
            redis,
            mongodb,
            rabbitmq,
        }
    }
}
```

### Test Data Factories

```rust
pub struct TestDataFactory {
    pub product_factory: ProductFactory,
    pub store_factory: StoreFactory,
}

impl TestDataFactory {
    pub fn create_test_product(&self) -> Product {
        self.product_factory
            .with_name("Test Product")
            .with_price(Price::new(100, "USD"))
            .with_store(self.store_factory.create_test_store())
            .build()
    }
}
```

### Test Utilities

```rust
pub mod test_utils {
    pub async fn wait_for_condition<F>(f: F, timeout: Duration) -> Result<(), Error>
    where
        F: Fn() -> bool,
    {
        let start = Instant::now();
        while !f() {
            if start.elapsed() > timeout {
                return Err(Error::Timeout);
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Ok(())
    }
    
    pub fn generate_test_id() -> String {
        format!("test_{}", Uuid::new_v4())
    }
}
```

## Test Coverage

We aim for the following coverage targets:

1. **Domain Layer**: 95% code coverage
2. **Ports**: 90% code coverage
3. **Adapters**: 85% code coverage
4. **Integration Tests**: Cover all critical paths
5. **Performance Tests**: Cover high-traffic endpoints

## Continuous Integration

Tests are run in CI/CD pipeline:

1. Unit tests on every commit
2. Integration tests on PR
3. Performance tests nightly
4. Coverage reports generated and tracked

## Best Practices

1. **Test Independence**
   - Each test should be self-contained
   - Clean up test data after each test
   - Avoid test interdependencies

2. **Test Data Management**
   - Use factories for test data creation
   - Avoid hardcoded test data
   - Clean up test data in teardown

3. **Mock Usage**
   - Mock external dependencies
   - Use realistic mock responses
   - Document mock behavior

4. **Performance Testing**
   - Run tests in isolation
   - Monitor system resources
   - Use realistic load patterns 