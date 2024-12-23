use super::helpers::{get_test_client, get_test_connection, get_test_connection_manager};

#[tokio::test]
async fn test_redis_connection() {
    // Test basic client connection
    let client = get_test_client().expect("Failed to create Redis client");
    assert!(client.get_connection().is_ok());

    // Test synchronous connection with auth
    let mut conn = get_test_connection()
        .expect("Failed to get Redis connection");
    let ping: String = redis::cmd("PING")
        .query(&mut conn)
        .expect("Failed to execute PING");
    assert_eq!(ping, "PONG");

    // Test async connection manager
    let mut conn_manager = get_test_connection_manager()
        .await
        .expect("Failed to get Redis connection manager");
    
    let ping: String = redis::cmd("PING")
        .query_async(&mut conn_manager)
        .await
        .expect("Failed to ping Redis");
    assert_eq!(ping, "PONG");
} 