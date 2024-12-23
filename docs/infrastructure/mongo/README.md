# MongoDB Configuration

This document describes the MongoDB setup and configuration for the Rust Scraper project.

## Overview

The project uses MongoDB 6.0 as its primary database, running in a Docker container. The setup includes:
- Secure user authentication
- Collection indexes
- Integration tests
- Environment-based configuration

## Configuration

### Environment Variables

The MongoDB configuration uses environment variables for sensitive data. Create a `.env` file with:

```env
# MongoDB Credentials
MONGO_ROOT_USERNAME=admin
MONGO_ROOT_PASSWORD=your_root_password_here
MONGO_APP_USERNAME=rust_scraper
MONGO_APP_PASSWORD=your_app_password_here
MONGO_DATABASE=rust_scraper
```

> ⚠️ Never commit the `.env` file. Use `.env.example` as a template.

### Docker Configuration

MongoDB runs in a Docker container defined in `docker-compose.yml`:

```yaml
services:
  mongodb:
    image: mongo:6.0
    container_name: rust_scraper_mongodb
    ports:
      - "27017:27017"
    environment:
      MONGO_INITDB_ROOT_USERNAME: ${MONGO_ROOT_USERNAME}
      MONGO_INITDB_ROOT_PASSWORD: ${MONGO_ROOT_PASSWORD}
      MONGO_INITDB_DATABASE: ${MONGO_DATABASE}
    volumes:
      - mongodb_data:/data/db
      - ./docker/mongodb/init.js:/docker-entrypoint-initdb.d/init.js:ro
```

### Database Initialization

The database is initialized using `init.js` which:
1. Creates the application database
2. Sets up the application user with appropriate permissions
3. Creates collections and indexes

## Collections

### Users Collection
- Primary collection for user data
- Indexes:
  - `email`: Unique index
  - `deleted_at`: Index for soft deletion queries

## Testing

Integration tests are organized under `tests/infrastructure/mongo/`:

```
tests/infrastructure/mongo/
├── helpers.rs       # Shared test utilities
├── connection_test.rs # Connection tests
└── crud_test.rs     # CRUD operation tests
```

Run tests with:
```bash
cargo test
```

### Test Coverage
- Connection testing (admin and application users)
- CRUD operations
- Unique constraint validation
- Data cleanup after tests

## Security

### Authentication
- Root user: Full administrative access
- Application user: Limited to specific database with read/write permissions
- Credentials stored in environment variables
- Connection strings include proper authentication source

### Network
- MongoDB runs on default port 27017
- Access restricted to container network
- Port exposed to host for development

## Maintenance

### Backup
The MongoDB data is persisted using Docker volumes:
```yaml
volumes:
  mongodb_data:
    name: rust_scraper_mongodb_data
```

### Monitoring
TODO: Add monitoring configuration (e.g., MongoDB Compass, Prometheus metrics)

## Development Setup

1. Copy environment template:
   ```bash
   cp .env.example .env
   ```

2. Update credentials in `.env`

3. Start MongoDB:
   ```bash
   docker-compose up -d
   ```

4. Verify setup:
   ```bash
   docker-compose logs mongodb
   ```

## Future Improvements

1. Add connection pooling configuration
2. Implement monitoring and alerting
3. Add backup automation
4. Configure replica set for high availability
5. Add performance optimization guidelines 