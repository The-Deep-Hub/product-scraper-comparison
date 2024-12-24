# Product Scraping API

A Rust-based API for scraping product information from various home improvement stores, featuring parallel scraping, caching, and asynchronous task processing.

## Features

- Multi-store product search (Leroy Merlin, Bauhaus, Bricodepot)
- Parallel scraping across all supported stores
- Redis-based caching system
- RabbitMQ task queue for asynchronous processing
- Background worker with concurrent task processing
- Zyte (formerly ScrapingHub) integration for reliable scraping

## Architecture

The application consists of several components:

- **API Server**: Handles HTTP requests and manages the scraping workflow
- **Task Worker**: Processes scraping tasks in parallel across multiple stores
- **Cache Service**: Redis-based caching for search results
- **Queue Service**: RabbitMQ-based task queue for asynchronous processing
- **Scraper Services**: Store-specific scrapers with Zyte integration

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- Redis
- RabbitMQ
- Zyte API Key

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

3. Update the following environment variables in `.env`:
   - `ZYTE_API_KEY`: Your Zyte API key
   - `REDIS_PASSWORD`: Your Redis password
   - `AMQP_ADDR`: RabbitMQ connection string

4. Start the required services:
   ```bash
   docker-compose up -d
   ```

5. Run the task worker:
   ```bash
   cargo run --bin task_worker
   ```

6. In a separate terminal, run the API server:
   ```bash
   cargo run
   ```

## Usage

### Publishing Tasks

```bash
# Publish a scraping task
cargo run --bin publish_task
```

### API Endpoints

- `POST /api/scraper`: Start a new scraping task
- `GET /api/task/{task_id}`: Get task status and results
- `GET /api/product`: Get product details by URL

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

## Roadmap

- [ ] Graceful shutdown for task worker
- [ ] Task retry mechanism with exponential backoff
- [ ] Health check endpoints
- [ ] Monitoring and metrics
- [ ] Rate limiting
- [ ] API documentation (OpenAPI/Swagger)

## License

This project is licensed under the MIT License - see the LICENSE file for details.