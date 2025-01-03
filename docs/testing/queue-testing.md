# 🧪 RabbitMQ Testing Strategy

<div align="center">

*Comprehensive testing approach for the RabbitMQ message queue system*

</div>

## 📑 Table of Contents

<details open>
<summary><strong>Strategy & Approach</strong></summary>

- [Overview](#overview)
- [Testing Approach](#testing-approach)
- [Test Environment](#test-environment)
</details>

<details open>
<summary><strong>Test Categories</strong></summary>

- [Unit Tests](#unit-tests)
- [Integration Tests](#integration-tests)
- [Performance Tests](#performance-tests)
- [Failure Tests](#failure-tests)
</details>

<details open>
<summary><strong>Implementation</strong></summary>

- [Test Infrastructure](#test-infrastructure)
- [Test Data Generation](#test-data-generation)
- [Mocking Strategy](#mocking-strategy)
</details>

## Overview

Testing the RabbitMQ implementation requires a comprehensive approach that covers both the reliability of message delivery and the system's behavior under various failure conditions.

### Testing Approach

1. **Unit Testing with Mocks**:
   - Mock RabbitMQ connections and channels
   - Test message serialization/deserialization
   - Verify retry logic and error handling
   - Test queue configuration management

2. **Integration Testing with Test Containers**:
   - Real RabbitMQ instance in Docker
   - End-to-end message flow testing
   - Dead letter queue testing
   - Consumer group behavior

3. **Performance Testing**:
   - Message throughput
   - Consumer scaling
   - Backpressure handling
   - Memory usage under load

## Test Implementation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_message_publishing() {
        let mock_channel = MockChannel::new();
        let adapter = RabbitMQAdapter::with_channel(mock_channel);
        
        let task = ScrapingTask::new("test_query");
        assert!(adapter.publish_task(task).await.is_ok());
    }

    #[tokio::test]
    async fn test_retry_policy() {
        let policy = RetryPolicy {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            backoff_factor: 2.0,
            max_delay: Duration::from_secs(30),
        };

        assert_eq!(policy.calculate_delay(0), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(2));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(4));
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_message_flow() {
    let container = RabbitMQTestContainer::new().await;
    let config = QueueConfig::from_container(&container);
    
    // Set up publisher and consumer
    let publisher = RabbitMQAdapter::new(config.clone()).await?;
    let consumer = RabbitMQAdapter::new(config).await?;
    
    // Publish test message
    let task = generate_test_task();
    publisher.publish_task(task.clone()).await?;
    
    // Consume and verify
    let received = consumer.receive_task().await?;
    assert_eq!(received, task);
}
```

### Performance Tests

```rust
#[tokio::test]
async fn test_throughput() {
    let adapter = setup_test_queue().await;
    let start = Instant::now();
    
    // Publish messages in parallel
    let handles: Vec<_> = (0..1000).map(|i| {
        let adapter = adapter.clone();
        tokio::spawn(async move {
            adapter.publish_task(
                generate_test_task(i)
            ).await
        })
    }).collect();
    
    // Wait for all publishes
    for handle in handles {
        handle.await.unwrap()?;
    }
    
    // Verify throughput
    let elapsed = start.elapsed();
    assert!(elapsed < Duration::from_secs(5));
}
```

## Test Infrastructure

### RabbitMQ Test Container

```rust
pub struct RabbitMQTestContainer {
    container: Container<RabbitMq>,
    connection_string: String,
}

impl RabbitMQTestContainer {
    pub async fn new() -> Self {
        let container = Container::new("rabbitmq:3.12-alpine")
            .with_env("RABBITMQ_DEFAULT_USER", "test")
            .with_env("RABBITMQ_DEFAULT_PASS", "test")
            .await?;
        
        Self {
            connection_string: container.connection_string(),
            container,
        }
    }
}
```

### Test Data Generation

```rust
pub fn generate_test_task(id: u32) -> ScrapingTask {
    ScrapingTask {
        id: format!("test_{}", id),
        query: "test_query".into(),
        priority: 1,
        ttl: Duration::from_secs(300),
        retry_count: 0,
    }
}
```

## Failure Testing

### Connection Failures

```rust
#[tokio::test]
async fn test_connection_recovery() {
    let container = RabbitMQTestContainer::new().await;
    let adapter = setup_test_adapter(&container).await;
    
    // Simulate network partition
    container.pause().await?;
    tokio::time::sleep(Duration::from_secs(5)).await;
    container.unpause().await?;
    
    // Verify recovery
    let task = generate_test_task(1);
    assert!(adapter.publish_task(task).await.is_ok());
}
```

### Consumer Failures

```rust
#[tokio::test]
async fn test_consumer_recovery() {
    let adapter = setup_test_adapter().await;
    let consumer = setup_test_consumer().await;
    
    // Kill consumer
    consumer.abort();
    
    // Verify messages are requeued
    let new_consumer = setup_test_consumer().await;
    assert!(new_consumer.receive_task().await.is_ok());
}
```

## Monitoring Tests

```rust
#[tokio::test]
async fn test_metrics_collection() {
    let adapter = setup_test_adapter().await;
    let metrics = setup_test_metrics();
    
    // Publish and consume messages
    publish_test_messages(&adapter, 100).await;
    
    // Verify metrics
    assert_eq!(
        metrics.get_counter("messages_published_total"),
        100
    );
}
```

## Best Practices

1. **Test Isolation**
   - Use fresh RabbitMQ instance for each test
   - Clean up queues between tests
   - Avoid test interdependencies

2. **Failure Testing**
   - Test network partitions
   - Test consumer failures
   - Test queue overflow
   - Test message expiration

3. **Performance Testing**
   - Establish baselines
   - Test with realistic load
   - Monitor resource usage
   - Test backpressure handling

4. **Integration Testing**
   - Test complete message flow
   - Verify message ordering
   - Test dead letter handling
   - Test retry mechanisms

## CI/CD Integration

```yaml
queue-tests:
  runs-on: ubuntu-latest
  services:
    rabbitmq:
      image: rabbitmq:3.12-alpine
      ports:
        - 5672:5672
      env:
        RABBITMQ_DEFAULT_USER: test
        RABBITMQ_DEFAULT_PASS: test
  steps:
    - uses: actions/checkout@v2
    - name: Run queue tests
      run: cargo test --package queue
``` 