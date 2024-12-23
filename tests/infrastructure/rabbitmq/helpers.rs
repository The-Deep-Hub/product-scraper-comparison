use lapin::{
    Connection, ConnectionProperties, Channel,
    options::{QueueDeclareOptions, BasicPublishOptions, BasicConsumeOptions},
    types::FieldTable,
    protocol::basic::AMQPProperties,
};
use dotenv::dotenv;
use std::env;

/// Gets a RabbitMQ connection for testing
pub async fn get_test_connection() -> Result<Connection, lapin::Error> {
    dotenv().ok();
    
    let user = env::var("RABBITMQ_USER").expect("RABBITMQ_USER must be set");
    let password = env::var("RABBITMQ_PASSWORD").expect("RABBITMQ_PASSWORD must be set");
    let host = env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
    let port = env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());
    let vhost = env::var("RABBITMQ_VHOST").unwrap_or_else(|_| "/".to_string());
    
    let uri = format!(
        "amqp://{}:{}@{}:{}/{}",
        user, password, host, port, vhost
    );
    
    Connection::connect(
        &uri,
        ConnectionProperties::default()
    ).await
}

/// Gets a channel from a connection
pub async fn get_test_channel(conn: &Connection) -> Result<Channel, lapin::Error> {
    conn.create_channel().await
}

/// Declares a test queue
pub async fn declare_test_queue(
    channel: &Channel,
    queue_name: &str,
    durable: bool
) -> Result<lapin::Queue, lapin::Error> {
    channel.queue_declare(
        queue_name,
        QueueDeclareOptions {
            durable,
            auto_delete: false,
            ..Default::default()
        },
        FieldTable::default()
    ).await
}

/// Publishes a message to a queue
pub async fn publish_test_message(
    channel: &Channel,
    queue_name: &str,
    message: &str
) -> Result<(), lapin::Error> {
    channel.basic_publish(
        "",
        queue_name,
        BasicPublishOptions::default(),
        message.as_bytes(),
        AMQPProperties::default()
    ).await?;
    
    Ok(())
}

/// Consumes messages from a queue
pub async fn consume_test_messages(
    channel: &Channel,
    queue_name: &str
) -> Result<lapin::Consumer, lapin::Error> {
    channel.basic_consume(
        queue_name,
        "test_consumer",
        BasicConsumeOptions::default(),
        FieldTable::default()
    ).await
}

/// Cleans up test queues
pub async fn cleanup_test_queue(
    channel: &Channel,
    queue_name: &str
) -> Result<(), lapin::Error> {
    channel.queue_delete(
        queue_name,
        lapin::options::QueueDeleteOptions::default()
    ).await?;
    
    Ok(())
} 