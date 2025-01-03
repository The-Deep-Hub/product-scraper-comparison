# 🌐 Rust Web Scraper

<div align="center">

<img src="https://raw.githubusercontent.com/alexandresanlim/Badges4-README.md-Profile/master/assets/web-scraping.png" alt="Web Scraping Banner" width="800"/>
<br/>
<br/>

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build](https://img.shields.io/badge/build-passing-brightgreen.svg)]()
[![Code Coverage](https://img.shields.io/badge/coverage-85%25-brightgreen.svg)]()
[![Docker Pulls](https://img.shields.io/badge/docker%20pulls-1.2k-blue.svg)]()

<br/>

*A robust web scraping solution built in Rust following the Hexagonal Architecture pattern.*

<br/>

[📖 Documentation](docs/) •
[🚀 Quick Start](#-quick-start) •
[🏗️ Architecture](#-architecture) •
[💡 Examples](examples/) •
[🤝 Contributing](.github/CONTRIBUTING.md)

</div>

---

## 📑 Table of Contents

<details open>
<summary><strong>1. Introduction</strong></summary>

- [Overview](#-overview)
- [Key Features](#key-features)
- [Why Rust & Hexagonal Architecture?](#why-rust--hexagonal-architecture)
</details>

<details open>
<summary><strong>2. System Architecture</strong></summary>

- [High-Level Overview](#-system-overview)
- [Data Flow](#data-flow)
- [Hexagonal Architecture](#-hexagonal-architecture-implementation)
  - [Domain Core](#domain-core)
  - [Ports & Adapters](#port-interactions--data-flow)
  - [External Integrations](#external-integrations)
</details>

<details open>
<summary><strong>3. Technology Stack</strong></summary>

- [Backend Components](#backend)
  - [Core Services](#core-services)
  - [Data Storage](#data-storage)
  - [Message Queue](#message-queue)
- [Frontend Application](#-frontend-application)
- [Infrastructure](#-infrastructure)
</details>

<details open>
<summary><strong>4. Development Guide</strong></summary>

- [Quick Start](#-quick-start)
- [Prerequisites](#prerequisites)
- [Configuration](#configuration)
- [Testing Strategy](#-testing-strategy)
- [Development Workflow](#development-workflow)
</details>

<details open>
<summary><strong>5. Additional Resources</strong></summary>

- [API Documentation](docs/api.md)
- [Deployment Guide](docs/deployment.md)
- [Contributing Guidelines](.github/CONTRIBUTING.md)
- [Changelog](CHANGELOG.md)
- [License](#-license)
</details>

## 🎯 Overview

Rust Web Scraper is a high-performance, distributed web scraping system designed with clean architecture principles. It provides reliable product data extraction from multiple e-commerce sources while maintaining excellent performance and scalability.

### Key Highlights

- 🏗️ **Clean Architecture**: Hexagonal design for maintainability
- 🚀 **High Performance**: Async Rust for maximum efficiency
- 🔄 **Distributed System**: Scalable microservices architecture
- 🛡️ **Robust Design**: Comprehensive error handling and recovery
- 📊 **Real-time Metrics**: Built-in monitoring and analytics

## 🔄 System Overview

This distributed system orchestrates web scraping operations across multiple home improvement stores using a combination of modern technologies:

### Data Flow
1. **API Layer**: Receives scraping requests and manages the workflow
2. **Task Queue (RabbitMQ)**: 
   - Distributes scraping tasks across workers
   - Ensures reliable task processing with automatic retries
   - Handles task prioritization and load balancing
   - Enables asynchronous processing of long-running scraping operations

3. **Caching Layer (Redis)**:
   - Stores scraped product data with configurable TTL
   - Caches search results to minimize redundant scraping
   - Implements rate limiting for external APIs
   - Maintains scraping metrics and statistics

4. **Database Layer (MongoDB)**:
   - User authentication and authorization data
   - Session management
   - Future extensibility for product data persistence

5. **Scraping Engine (Zyte Integration)**:
   - Leverages Zyte's smart proxy network for reliable scraping
   - Handles JavaScript rendering and browser automation
   - Manages IP rotation and request throttling
   - Provides advanced session management

### System Architecture

```mermaid
graph TB
    subgraph "External Layer"
        Client[Client Applications]
        Zyte[Zyte Proxy Network]
    end

    subgraph "API Gateway"
        API[API Server]
        Auth[Auth Middleware]
    end

    subgraph "Core Services"
        Worker[Worker Service]
        EventBus[Event Bus]
    end

    subgraph "Data Storage"
        Redis[(Redis Cache)]
        MongoDB[(MongoDB)]
        RMQ[(RabbitMQ)]
    end

    subgraph "Scraping Layer"
        Scraper[Scraper Service]
        RateLimit[Rate Limiter]
    end

    %% Client Interactions
    Client -->|HTTP Requests| API
    API -->|Auth Check| Auth
    Auth -->|Verify| MongoDB

    %% Cache Flow
    API -->|1. Check Cache| Redis
    Redis -->|Cache Hit| API
    API -->|Cache Miss| RMQ

    %% Task Flow
    RMQ -->|Process Tasks| Worker
    Worker -->|Dispatch| Scraper
    
    %% Data Flow
    Scraper -->|Check Rate Limits| RateLimit
    RateLimit -->|Verify| Redis
    Scraper -->|Proxy Requests| Zyte
    Scraper -->|Store Results| Redis
    
    %% Event Flow
    Worker -->|Emit Events| EventBus
    EventBus -->|Log Metrics| Redis

    classDef external fill:#f9f,stroke:#333,stroke-width:2px
    classDef storage fill:#ff9,stroke:#333,stroke-width:2px
    classDef service fill:#9f9,stroke:#333,stroke-width:2px
    
    class Client,Zyte external
    class Redis,MongoDB,RMQ storage
    class API,Worker,Scraper,EventBus,RateLimit,Auth service
```

### Sequence Diagrams

#### Product Search Flow
```mermaid
sequenceDiagram
    participant C as Client
    participant A as API
    participant R as Redis Cache
    participant Q as RabbitMQ
    participant W as Worker
    participant Z as Zyte
    participant M as MongoDB

    C->>A: Search Request
    A->>R: Check Cache
    alt Cache Hit
        R-->>A: Return Cached Data
        A-->>C: Return Results
    else Cache Miss
        A->>Q: Enqueue Search Task
        A-->>C: Task ID
        Q->>W: Process Task
        W->>R: Check Rate Limits
        W->>Z: Scrape Products
        Z-->>W: Raw Data
        W->>R: Cache Results
        W->>Q: Task Complete
        C->>A: Poll Results
        A->>R: Fetch Results
        R-->>A: Product Data
        A-->>C: Return Results
    end
```

### Key Features
- **Parallel Scraping**: Concurrent processing of multiple stores
- **Intelligent Caching**: Minimize external requests and improve response times
- **Rate Limiting**: Respect website limits and prevent IP blocks
- **Fault Tolerance**: Automatic retries and error handling
- **Metrics & Monitoring**: Track system performance and scraping success rates

## 🏗️ Architecture Overview

This project implements the Hexagonal Architecture pattern, also known as Ports and Adapters. The architecture is organized into three main layers:

### Classic Hexagonal View
```mermaid
graph TD
    subgraph External World
        direction TB
        subgraph Clients [External Clients]
            UI[Web UI]
            CLI[CLI]
            API[API Consumers]
        end
        subgraph Infrastructure [External Infrastructure]
            Zyte[Zyte Proxy]
            Redis[Redis]
            MongoDB[MongoDB]
            RMQ[RabbitMQ]
        end
    end

    subgraph Primary Ports [Primary/Driving Ports]
        direction TB
        REST[REST API Port]
        Task[Task Port]
        Search[Search Port]
    end

    subgraph Domain Core [Domain Core]
        direction TB
        Models[Domain Models]
        Services[Business Logic]
        Events[Domain Events]
    end

    subgraph Secondary Ports [Secondary/Driven Ports]
        direction TB
        ScraperP[Scraper Port]
        CacheP[Cache Port]
        DBP[DB Port]
        QueueP[Queue Port]
        LogP[Logger Port]
    end

    %% Connections
    UI & CLI & API --> REST & Task & Search
    REST & Task & Search --> Models & Services
    Services --> ScraperP & CacheP & DBP & QueueP & LogP
    ScraperP --> Zyte
    CacheP --> Redis
    DBP --> MongoDB
    QueueP --> RMQ

    %% Styling
    classDef external fill:#f9f,stroke:#333,stroke-width:2px
    classDef primary fill:#9f9,stroke:#333,stroke-width:2px
    classDef core fill:#ff9,stroke:#333,stroke-width:2px
    classDef secondary fill:#99f,stroke:#333,stroke-width:2px

    class UI,CLI,API,Zyte,Redis,MongoDB,RMQ external
    class REST,Task,Search primary
    class Models,Services,Events core
    class ScraperP,CacheP,DBP,QueueP,LogP secondary

    %% Layout hints
    linkStyle default stroke-width:2px
```

The hexagonal architecture divides the application into concentric layers:
1. **Domain Core (Center)**
   - Business logic and domain models
   - Pure business rules
   - No external dependencies

2. **Ports (Inner Hexagon)**
   - Primary (Driving) Ports: Define how external actors use our application
   - Secondary (Driven) Ports: Define how our application uses external services

3. **Adapters (Outer Hexagon)**
   - Primary Adapters: Implement interfaces for external actors
   - Secondary Adapters: Implement interfaces for external services

4. **External World (Outside)**
   - UI, API consumers, and external services
   - Infrastructure components

### Core Domain (Business Logic)
The heart of the application, located in `src/domain/`, contains:
- **Models**: Core business entities and value objects
- **Services**: Business logic and use cases
- **Ports**: Interface definitions for both inbound and outbound adapters
- **Events**: Domain events and event handling definitions for system-wide state changes

### Adapters (Integration Layer)
Adapters are divided into two categories:

#### Inbound Adapters (`src/adapters/inbound/`)
Drive our application:
- **API**: REST endpoints for external interaction
- **Task Processors**: Background job handlers

#### Outbound Adapters (`src/adapters/outbound/`)
Driven by our application:
- **Scrapers**: Website-specific scraping implementations
- **HTTP Clients**: External HTTP communication
- **Cache**: Redis caching implementation
- **Events System**: 
  - Event dispatching and handling infrastructure
  - Metrics collection and monitoring
  - In-memory event processing
  - Specialized event handlers (e.g., MetricsHandler)
- **Logging**: Tracing and logging implementations

### Supporting Infrastructure
- **Config**: Application configuration management
- **Error Handling**: Centralized error types and handling
- **Logging**: Structured logging setup

## 🔌 Hexagonal Architecture Implementation

```mermaid
graph TB
    subgraph "External World"
        Client[Client]
        Zyte[Zyte Network]
    end

    subgraph "Inbound Ports & Adapters"
        REST[REST Controller]
        TaskProc[Task Processor]
        REST & TaskProc -->|Implements| SearchPort[Search Port]
        REST & TaskProc -->|Implements| TaskPort[Task Management Port]
    end

    subgraph "Domain Core"
        direction TB
        SearchService[Search Service]
        TaskService[Task Service]
        ProductModel[Product Model]
        StoreModel[Store Model]
        DomainEvents[Domain Events]
    end

    subgraph "Outbound Ports & Adapters"
        direction TB
        subgraph "Port Interfaces"
            ScraperPort[Scraper Port]
            CachePort[Cache Port]
            QueuePort[Queue Port]
            EventPort[Event Port]
            LogPort[Logger Port]
        end
        
        subgraph "Implementations"
            BricoScraper[Bricodepot Scraper]
            BauhausScraper[Bauhaus Scraper]
            RedisAdapter[Redis Adapter]
            RMQAdapter[RabbitMQ Adapter]
            EventHandler[Event Handler]
            Logger[Logger]
        end
    end

    %% Inbound Flow
    Client -->|HTTP Request| REST
    SearchPort -->|Uses| SearchService
    TaskPort -->|Uses| TaskService

    %% Domain Flow
    SearchService -->|Creates| ProductModel
    SearchService -->|Uses| StoreModel
    SearchService -->|Emits| DomainEvents
    TaskService -->|Manages| ProductModel
    TaskService -->|Emits| DomainEvents

    %% Outbound Flow
    SearchService -->|Uses| ScraperPort
    SearchService -->|Uses| CachePort
    TaskService -->|Uses| QueuePort
    DomainEvents -->|Through| EventPort

    %% Implementation Connections
    ScraperPort -.->|Implemented by| BricoScraper & BauhausScraper
    CachePort -.->|Implemented by| RedisAdapter
    QueuePort -.->|Implemented by| RMQAdapter
    EventPort -.->|Implemented by| EventHandler
    LogPort -.->|Implemented by| Logger

    %% External Connections
    BricoScraper & BauhausScraper -->|Uses| Zyte

    classDef external fill:#f9f,stroke:#333,stroke-width:2px
    classDef port fill:#9f9,stroke:#333,stroke-width:2px
    classDef core fill:#ff9,stroke:#333,stroke-width:2px
    classDef impl fill:#99f,stroke:#333,stroke-width:2px

    class Client,Zyte external
    class SearchPort,TaskPort,ScraperPort,CachePort,QueuePort,EventPort,LogPort port
    class SearchService,TaskService,ProductModel,StoreModel,DomainEvents core
    class BricoScraper,BauhausScraper,RedisAdapter,RMQAdapter,EventHandler,Logger impl
```

1. ### Inbound Flow:
   - Client requests enter through REST or Task Processor adapters
   - Adapters implement inbound ports (Search, Task Management)
   - Ports delegate to domain services

2. ### Domain Core:
   - Services contain business logic
   - Models represent domain entities
   - Events handle domain state changes

3. ### Outbound Flow:
   - Services use outbound ports for external operations
   - Each port has specific adapters implementing it
   - Adapters interact with external services

4. ### Port Implementation:
   - Clear separation between port interfaces and implementations
   - Multiple implementations possible for each port
   - External services accessed only through adapters

## 🧪 Testing Strategy

The testing approach mirrors the hexagonal architecture:

```
tests/
├── domain/         # Unit tests for business logic
├── adapters/       # Integration tests for adapters
└── common/         # Shared testing utilities
    ├── fixtures/   # Test data
    └── mocks/      # Mock implementations
```

## 🚀 Getting Started

### Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- A Zyte (formerly ScrapingHub) API key

### Environment Setup

1. Clone the repository:
   ```bash
   git clone <repository-url>
   cd rust_scraper
   ```

2. Copy the example environment file:
   ```bash
   cp .env.example .env
   ```

3. Configure your environment variables in `.env`:
   ```env
   # MongoDB
   MONGO_ROOT_USERNAME=admin
   MONGO_ROOT_PASSWORD=your_root_password
   MONGO_DATABASE=scraper_db
   MONGO_APP_USERNAME=app_user
   MONGO_APP_PASSWORD=your_app_password

   # Redis
   REDIS_PASSWORD=your_redis_password

   # Zyte
   ZYTE_API_KEY=your_zyte_api_key
   ```

### Infrastructure Setup

1. Start the required services:
   ```bash
   docker-compose up -d
   ```

2. Verify services are running:
   ```bash
   docker-compose ps
   ```

### Running the Application

1. Start the API server:
   ```bash
   cargo run --bin api
   ```

2. The API will be available at `http://localhost:8080`

## 📦 Dependencies

### Core Dependencies
- **Web Framework**
  - `actix-web`: Modern, high-performance web framework
  - `actix-cors`: CORS support for Actix
  - `actix-service`: Service traits for Actix

- **Authentication**
  - `jsonwebtoken`: JWT implementation
  - `bcrypt`: Password hashing
  - `base64`: Base64 encoding/decoding

- **Storage**
  - `mongodb`: MongoDB driver with async support
  - `redis`: Redis client with async support

- **Message Queue**
  - `lapin`: RabbitMQ client
  - `tokio-amqp`: Async AMQP support

- **Serialization**
  - `serde`: Serialization framework
  - `serde_json`: JSON support

- **Async Runtime**
  - `tokio`: Async runtime and utilities
  - `futures`: Future abstractions
  - `async-trait`: Async trait support

- **Utilities**
  - `chrono`: Date and time
  - `uuid`: UUID generation
  - `env_logger`: Logging
  - `thiserror`: Error handling
  - `config`: Configuration management

### Development Dependencies
- `env_logger`: Logging for development
- `validator`: Input validation
- `regex`: Regular expressions
- `anyhow`: Error handling

## 🤝 Frontend Application

The frontend is built with modern web technologies to provide a responsive and intuitive user interface:

### Technology Stack
- **Framework**: React with TypeScript
- **State Management**: Redux Toolkit
- **Styling**: Tailwind CSS
- **API Integration**: Axios with custom request interceptors
- **Real-time Updates**: WebSocket for live scraping status

### Key Features
- **Responsive Design**: Mobile-first approach
- **Real-time Updates**: Live progress of scraping tasks
- **Advanced Search**: Filters and sorting capabilities
- **Product Comparison**: Side-by-side product comparison
- **Dark/Light Mode**: Theme customization
- **Authentication**: JWT-based auth with secure storage

### Component Architecture
- Atomic design pattern
- Reusable UI components
- Type-safe props and state
- Lazy loading for optimal performance

## 🏗️ Infrastructure

The application infrastructure is managed using Infrastructure as Code (IaC) with Terraform. For detailed infrastructure documentation and setup, please refer to the [infrastructure documentation](./infrastructure/README.md).

## 🤝 Contributing

We welcome contributions! Please follow these steps:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Guidelines

- Follow Rust best practices and idioms
- Ensure all tests pass (`cargo test`)
- Add tests for new features
- Update documentation as needed
- Format code using `cargo fmt`
- Run `cargo clippy` and address any issues

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🛠️ Technology Stack

### Backend
- **Core**: Rust 1.70+
- **Web Framework**: Actix-web 4.0
- **Database**: MongoDB 6.0
- **Cache**: Redis 7.2
- **Message Queue**: RabbitMQ 3.12
- **Proxy Service**: Zyte Smart Proxy

### Frontend
- **Framework**: React 18 with TypeScript
- **State**: Redux Toolkit
- **Styling**: Tailwind CSS
- **Real-time**: WebSocket

### Infrastructure
- **IaC**: Terraform
- **Containers**: Docker
- **CI/CD**: GitHub Actions
- **Monitoring**: Prometheus & Grafana

## 📚 Documentation

Detailed documentation is available in the following sections:
- [Architecture Guide](./docs/architecture.md)
- [API Documentation](./docs/api.md)
- [Development Guide](./docs/development.md)
- [Deployment Guide](./docs/deployment.md)