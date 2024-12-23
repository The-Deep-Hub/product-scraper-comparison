use redis::{Client, Connection, RedisResult, Value};
use redis::aio::ConnectionManager;
use dotenv::dotenv;
use std::env;

/// Gets a Redis client for testing
pub fn get_test_client() -> RedisResult<Client> {
    dotenv().ok();
    
    let redis_password = env::var("REDIS_PASSWORD").expect("REDIS_PASSWORD must be set");
    let redis_host = env::var("REDIS_HOST").unwrap_or_else(|_| "localhost".to_string());
    let redis_port = env::var("REDIS_PORT").unwrap_or_else(|_| "6379".to_string());
    
    let redis_url = format!("redis://:{}@{}:{}", redis_password, redis_host, redis_port);
    Client::open(redis_url)
}

/// Gets a Redis connection for testing
pub fn get_test_connection() -> RedisResult<Connection> {
    let client = get_test_client()?;
    client.get_connection()
}

/// Gets a Redis connection manager for testing
pub async fn get_test_connection_manager() -> RedisResult<ConnectionManager> {
    let client = get_test_client()?;
    ConnectionManager::new(client).await
}

/// Cleans up test data with a given pattern
pub fn cleanup_test_data(conn: &mut Connection, pattern: &str) -> RedisResult<bool> {
    let keys: Vec<String> = redis::cmd("KEYS")
        .arg(pattern)
        .query(conn)?;
    
    if !keys.is_empty() {
        let del_result: RedisResult<Value> = redis::cmd("DEL")
            .arg(keys)
            .query(conn);
        
        match del_result {
            Ok(_) => Ok(true),
            Err(e) => Err(e),
        }
    } else {
        Ok(false)
    }
} 