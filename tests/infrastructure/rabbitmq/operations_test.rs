use super::helpers::*;
use futures_lite::stream::StreamExt;
use lapin::protocol::basic::AMQPProperties;
use serde::{Serialize, Deserialize};
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestMessage {
    id: String,
    content: String,
}

#[tokio::test]
async fn test_basic_operations() {
    // Setup
    let conn = get_test_connection()
        .await
        .expect("Failed to connect to RabbitMQ");
    let channel = get_test_channel(&conn)
        .await
        .expect("Failed to create channel");
    
    // Clean up any existing queue
    let queue_name = "test_operations_queue";
    let _ = cleanup_test_queue(&channel, queue_name).await;
    
    // Declare test queue
    declare_test_queue(&channel, queue_name, true)
        .await
        .expect("Failed to declare queue");
    
    // Create test message
    let test_message = TestMessage {
        id: "123".to_string(),
        content: "test content".to_string(),
    };
    let message_json = serde_json::to_string(&test_message)
        .expect("Failed to serialize message");
    
    // Publish message
    publish_test_message(&channel, queue_name, &message_json)
        .await
        .expect("Failed to publish message");
    
    // Setup consumer
    let mut consumer = consume_test_messages(&channel, queue_name)
        .await
        .expect("Failed to create consumer");
    
    // Consume message
    if let Some(delivery) = consumer.next().await {
        let delivery = delivery.expect("Failed to get delivery");
        let received_message: TestMessage = serde_json::from_slice(&delivery.data)
            .expect("Failed to deserialize message");
        
        assert_eq!(received_message, test_message);
        
        // Acknowledge message
        delivery.ack(Default::default())
            .await
            .expect("Failed to acknowledge message");
    }
    
    // Clean up
    cleanup_test_queue(&channel, queue_name)
        .await
        .expect("Failed to clean up queue");
}

#[tokio::test]
async fn test_message_persistence() {
    let conn = get_test_connection()
        .await
        .expect("Failed to connect to RabbitMQ");
    let channel = get_test_channel(&conn)
        .await
        .expect("Failed to create channel");
    
    // Clean up any existing queue
    let queue_name = "test_persistent_queue";
    let _ = cleanup_test_queue(&channel, queue_name).await;
    
    // Declare durable queue
    declare_test_queue(&channel, queue_name, true)
        .await
        .expect("Failed to declare queue");
    
    // Verify queue is empty
    let queue = channel
        .queue_declare(
            queue_name,
            lapin::options::QueueDeclareOptions {
                durable: true,
                passive: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
        .expect("Failed to get queue info");
    
    assert_eq!(queue.message_count(), 0, "Queue should be empty before test");
    
    // Publish persistent message
    let message = "persistent message";
    channel.basic_publish(
        "",
        queue_name,
        lapin::options::BasicPublishOptions::default(),
        message.as_bytes(),
        AMQPProperties::default()
            .with_delivery_mode(2), // persistent
    )
    .await
    .expect("Failed to publish message");
    
    // Wait a moment to ensure message is persisted
    sleep(Duration::from_secs(1)).await;
    
    // Verify message count
    let queue = channel
        .queue_declare(
            queue_name,
            lapin::options::QueueDeclareOptions {
                durable: true,
                passive: true,
                ..Default::default()
            },
            lapin::types::FieldTable::default(),
        )
        .await
        .expect("Failed to get queue info");
    
    assert_eq!(queue.message_count(), 1, "Queue should have exactly one message");
    
    // Clean up
    cleanup_test_queue(&channel, queue_name)
        .await
        .expect("Failed to clean up queue");
} 