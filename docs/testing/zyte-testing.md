# 🧪 Zyte Testing Strategy

<div align="center">

*Comprehensive testing approach for the Zyte scraping service*

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

Testing the Zyte integration requires a comprehensive approach that covers both the reliability of web scraping operations and the system's behavior under various network and target website conditions.

### Testing Approach

1. **Unit Testing with Mocks**
   - Mock HTTP responses
   - Test request building
   - Verify proxy configuration
   - Test error handling
   - Validate rate limiting

2. **Integration Testing**
   - Real Zyte API calls
   - End-to-end scraping flows
   - JavaScript rendering
   - Proxy rotation testing
   - Session management

3. **Performance Testing**
   - Request throughput
   - Response times
   - Concurrent scraping
   - Memory usage
   - Bandwidth utilization

## Test Implementation

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use wiremock::{Mock, ResponseTemplate};

    #[tokio::test]
    async fn test_request_building() {
        let config = ZyteConfig::new("test_key".to_string());
        let client = ZyteClient::new(config).await?;
        
        let request = client.build_request(
            "https://example.com",
            RequestOptions {
                js_render: true,
                country_code: Some("US"),
                ..Default::default()
            }
        )?;
        
        assert_eq!(request.headers()["X-Crawlera-Profile"], "desktop");
        assert_eq!(request.headers()["X-Crawlera-Region"], "US");
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(10); // 10 RPS
        let start = Instant::now();
        
        for _ in 0..20 {
            limiter.acquire().await?;
        }
        
        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_secs(2));
    }
}
```

### Integration Tests

```rust
#[cfg(test)]
mod integration_tests {
    use test_context::{test_context, TestContext};

    struct ZyteContext {
        client: ZyteClient,
        test_server: TestServer,
    }

    #[test_context(ZyteContext)]
    #[tokio::test]
    async fn test_js_rendering(ctx: &mut ZyteContext) {
        let response = ctx.client
            .scrape("https://example.com", &ScrapeOptions {
                js_render: true,
                wait_for: Some("#dynamic-content"),
                timeout: Duration::from_secs(10),
            })
            .await?;
            
        assert!(response.body.contains("dynamic-content"));
        assert_eq!(response.status, 200);
    }

    #[test_context(ZyteContext)]
    #[tokio::test]
    async fn test_proxy_rotation() {
        let mut ips = HashSet::new();
        
        for _ in 0..10 {
            let response = ctx.client
                .scrape("https://httpbin.org/ip", &Default::default())
                .await?;
                
            let ip = extract_ip(&response.body);
            ips.insert(ip);
        }
        
        assert!(ips.len() > 1, "Proxy rotation not working");
    }
}
```

### Performance Tests

```rust
#[cfg(test)]
mod performance_tests {
    use criterion::{criterion_group, criterion_main, Criterion};

    fn benchmark_concurrent_scraping(c: &mut Criterion) {
        c.bench_function("concurrent_100_requests", |b| {
            b.iter(|| {
                let client = setup_test_client();
                let futures: Vec<_> = (0..100)
                    .map(|_| client.scrape("https://example.com"))
                    .collect();
                    
                runtime.block_on(futures::future::join_all(futures))
            })
        });
    }

    fn benchmark_js_rendering(c: &mut Criterion) {
        c.bench_function("js_render_heavy_page", |b| {
            b.iter(|| {
                let client = setup_test_client();
                runtime.block_on(client.scrape(
                    "https://example.com/js-heavy",
                    &ScrapeOptions { js_render: true }
                ))
            })
        });
    }
}
```

## Test Infrastructure

### Mock Server Setup

```rust
pub struct MockWebServer {
    server: WireMock,
    responses: HashMap<String, ResponseTemplate>,
}

impl MockWebServer {
    pub async fn new() -> Self {
        let server = WireMock::start().await;
        
        Mock::given(method("GET"))
            .and(path("/test"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_string("test response"))
            .mount(&server)
            .await;
            
        Self {
            server,
            responses: HashMap::new(),
        }
    }
}
```

### Test Data Generation

```rust
pub struct TestDataGenerator {
    templates: Vec<String>,
    current_index: AtomicUsize,
}

impl TestDataGenerator {
    pub fn generate_html_page(&self) -> String {
        let template = include_str!("../templates/test_page.html");
        template.replace(
            "{{content}}",
            &format!("Test Content {}", self.next_index())
        )
    }
    
    pub fn generate_js_page(&self) -> String {
        let template = include_str!("../templates/js_page.html");
        template.replace(
            "{{dynamic_content}}",
            &format!("Dynamic Content {}", self.next_index())
        )
    }
}
```

## Failure Testing

### Network Failures

```rust
#[tokio::test]
async fn test_network_failures() {
    let mock_server = MockWebServer::new().await;
    
    // Test timeout
    mock_server.simulate_delay(Duration::from_secs(5)).await;
    let result = client.scrape(mock_server.url("/slow")).await;
    assert!(matches!(result, Err(Error::Timeout)));
    
    // Test connection reset
    mock_server.simulate_connection_reset().await;
    let result = client.scrape(mock_server.url("/reset")).await;
    assert!(matches!(result, Err(Error::ConnectionReset)));
}
```

### Rate Limit Handling

```rust
#[tokio::test]
async fn test_rate_limit_handling() {
    let client = setup_test_client();
    let limiter = RateLimiter::new(1); // 1 RPS
    
    let results = futures::future::join_all((0..5).map(|_| {
        client.scrape_with_limiter("https://example.com", &limiter)
    }))
    .await;
    
    assert!(results.iter().all(|r| r.is_ok()));
    assert!(results.len() == 5);
}
```

## Best Practices

1. **Test Isolation**
   - Use fresh client for each test
   - Clean up test data
   - Avoid test interdependencies
   - Reset rate limiters

2. **Failure Testing**
   - Test network errors
   - Test malformed responses
   - Test rate limiting
   - Test proxy failures

3. **Performance Testing**
   - Establish baselines
   - Test with realistic load
   - Monitor resource usage
   - Test concurrent scraping

4. **Integration Testing**
   - Test real websites
   - Verify JavaScript rendering
   - Test proxy rotation
   - Test session handling

## CI/CD Integration

```yaml
zyte-tests:
  runs-on: ubuntu-latest
  env:
    ZYTE_API_KEY: ${{ secrets.ZYTE_API_KEY }}
  steps:
    - uses: actions/checkout@v2
    - name: Run tests
      run: |
        cargo test --package zyte-client
        cargo test --package zyte-client --test '*' --features integration-tests
``` 