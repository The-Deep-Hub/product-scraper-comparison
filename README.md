# Product Scraping API

A Rust-based API for scraping product information from various home improvement stores.

## Features

- Product search across multiple stores
- Detailed product information retrieval
- Caching system for improved performance
- Background worker for asynchronous scraping
- JWT-based authentication
- Rate limiting and request throttling
- Support for multiple stores (Leroy Merlin, with more to come)

## Architecture

The application is built using a microservices architecture with the following components:

- **API Server**: Handles HTTP requests and responses
- **Cache Service**: Redis-based caching for search results and product details
- **Queue Service**: RabbitMQ-based task queue for background processing
- **Worker Service**: Processes queued tasks for detailed product information
- **Scraper Service**: Manages the scraping logic for different stores
- **Database**: MongoDB for storing user and product information

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- MongoDB
- Redis
- RabbitMQ
- Zyte API Key (for proxy service)

## Setup

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/rust_scraper.git
   cd rust_scraper
   ```

2. Copy the example environment file and update it with your configuration:
   ```bash
   cp .env.example .env
   ```

3. Start the required services using Docker Compose:
   ```bash
   docker-compose up -d
   ```

4. Build and run the application:
   ```bash
   cargo build
   cargo run
   ```

## API Endpoints

### Authentication

- `POST /api/auth/register`: Register a new user
- `POST /api/auth/login`: Login and receive JWT token

### Product Search

- `POST /api/scraper`: Search for products
  ```json
  {
    "query": "hammer",
    "store": "leroy_merlin"
  }
  ```

### Product Details

- `GET /api/scraper/{product_url}`: Get detailed product information

## Worker Service

The worker service processes queued tasks in the background:

1. Polls the queue for pending tasks
2. Processes tasks concurrently (configurable limit)
3. Updates product details in the cache
4. Handles retries and error reporting

## Caching Strategy

- Search results are cached for 1 hour
- Product details are cached for 24 hours
- Cache is automatically invalidated when updates occur

## Configuration

Key environment variables:

- `ZYTE_API_KEY`: Your Zyte API key for proxy service
- `ZYTE_CONCURRENT_REQUESTS`: Maximum concurrent requests to Zyte
- `WORKER_POLLING_INTERVAL`: Worker polling interval in seconds
- `WORKER_MAX_CONCURRENT_TASKS`: Maximum concurrent worker tasks

## Development

### Running Tests

```bash
cargo test
```

### Code Style

```bash
cargo fmt
cargo clippy
```

## TODO

- [ ] Implement additional store scrapers (Bricodepot, Bauhaus, Obramat)
- [ ] Add price history tracking
- [ ] Implement product availability notifications
- [ ] Add product comparison feature
- [ ] Improve error handling and retry mechanisms
- [ ] Add metrics and monitoring
- [ ] Implement rate limiting per user
- [ ] Add API documentation using OpenAPI/Swagger

## Contributing

1. Fork the repository
2. Create your feature branch
3. Commit your changes
4. Push to the branch
5. Create a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.