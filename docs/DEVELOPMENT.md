# Local Development Guide

## Prerequisites

- Rust (latest stable version)
- Docker and Docker Compose
- Git

## Setting Up the Development Environment

1. Clone the repository:
```bash
git clone https://github.com/yourusername/rust_scraper.git
cd rust_scraper
```

2. Start the local MongoDB instance:
```bash
docker-compose up -d mongodb
```

3. Install Rust dependencies:
```bash
cargo build
```

4. Run the tests:
```bash
cargo test
```

5. Start the development server:
```bash
cargo run
```

The API will be available at `http://localhost:8080`.

## Development Database

The local MongoDB instance is configured with the following credentials:

- **Host**: localhost
- **Port**: 27017
- **Database**: rust_scraper
- **Username**: rust_scraper
- **Password**: rust_scraper_password

You can connect to the database using MongoDB Compass or the mongo shell:
```bash
mongosh mongodb://rust_scraper:rust_scraper_password@localhost:27017/rust_scraper
```

## Project Structure

```
.
├── src/
│   ├── api/           # API modules
│   ├── db/           # Database modules
│   └── bin/          # Binary executables
├── tests/            # Integration tests
├── docker/           # Docker configurations
└── config/           # Application configurations
```

## Development Workflow

1. Create a new feature branch:
```bash
git checkout -b feature/your-feature-name
```

2. Make your changes and write tests

3. Run the test suite:
```bash
cargo test
```

4. Format your code:
```bash
cargo fmt
```

5. Run the linter:
```bash
cargo clippy
```

6. Commit your changes:
```bash
git add .
git commit -m "feat: your feature description"
```

7. Push your changes and create a pull request:
```bash
git push origin feature/your-feature-name
```

## Troubleshooting

### MongoDB Connection Issues

If you can't connect to MongoDB, check:
1. Docker container status: `docker ps`
2. Container logs: `docker logs rust_scraper_mongodb`
3. MongoDB connection string in `config/default.toml`

### Cleaning Up

To remove all development containers and volumes:
```bash
docker-compose down -v
```