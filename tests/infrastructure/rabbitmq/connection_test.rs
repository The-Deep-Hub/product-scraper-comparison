use super::helpers::{get_test_connection, get_test_channel};
use lapin::options::QueueDeclareOptions;
use lapin::types::FieldTable;

#[tokio::test]
async fn test_rabbitmq_connection() {
    // Test basic connection
    let conn = get_test_connection()
        .await
        .expect("Failed to connect to RabbitMQ");
    assert!(conn.status().connected());

    // Test channel creation
    let channel = get_test_channel(&conn)
        .await
        .expect("Failed to create channel");
    
    // Test queue declaration
    let queue = channel
        .queue_declare(
            "test_queue",
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare queue");
    
    assert_eq!(queue.name().as_str(), "test_queue");
    
    // Clean up
    channel
        .queue_delete(
            "test_queue",
            lapin::options::QueueDeleteOptions::default(),
        )
        .await
        .expect("Failed to delete queue");
} 