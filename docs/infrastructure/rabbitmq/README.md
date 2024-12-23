# RabbitMQ Configuration

## Overview
This document provides details about the RabbitMQ setup used in the project for message queuing and asynchronous processing.

## Configuration Details

### Environment Variables
The following environment variables must be set in the `.env` file:
```env
RABBITMQ_USER=admin
RABBITMQ_PASSWORD=your_rabbitmq_password_here
RABBITMQ_VHOST=/
RABBITMQ_HOST=localhost
RABBITMQ_PORT=5672
RABBITMQ_MANAGEMENT_PORT=15672
```

### Docker Configuration
RabbitMQ is configured using Docker Compose with the following specifications:
- Image: `rabbitmq:3.12-management-alpine`
- Ports:
  - 5672: AMQP protocol port
  - 15672: Management UI port
- Volume mounts:
  - Configuration file: `./docker/rabbitmq/rabbitmq.conf`
  - Persistent data: `rabbitmq_data:/var/lib/rabbitmq`

### Management UI
The RabbitMQ Management interface is accessible at:
- URL: `http://localhost:15672`
- Default credentials:
  - Username: Value of `RABBITMQ_USER`
  - Password: Value of `RABBITMQ_PASSWORD`

## Testing
Integration tests are provided to verify the RabbitMQ setup:
- Connection tests: Verify basic connectivity
- Basic operations: Test message publishing and consuming
- Message persistence: Verify durable queue and message persistence

To run the tests:
```bash
cargo test infrastructure::rabbitmq
```

## Queue Structure
The following queues are configured:
- `scraper_tasks`: For incoming scraping tasks
  - Durable: true
  - Message TTL: 1 hour
  - Dead letter exchange: "dlx"
- `scraper_results`: For completed scraping results
  - Durable: true
  - Message TTL: 24 hours
- `dead_letter_queue`: For failed messages
  - Durable: true

## Monitoring and Maintenance
- Health checks are configured to run every 5 seconds
- Memory high watermark: 70% of system memory
- Disk free limit: 2GB
- Logging level: info
- Statistics collection interval: 5 seconds

## Best Practices
1. Always use durable queues for important messages
2. Implement proper error handling and message acknowledgment
3. Monitor queue sizes and consumer health
4. Use dead letter queues for failed message handling
5. Regularly check the Management UI for system health 