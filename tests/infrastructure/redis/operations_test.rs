use super::helpers::{get_test_connection, cleanup_test_data};
use redis::{Commands, RedisResult, Value};
use std::collections::HashMap;

#[tokio::test]
async fn test_basic_operations() {
    let mut conn = get_test_connection()
        .expect("Failed to get Redis connection");
    
    // Clean up any previous test data
    cleanup_test_data(&mut conn, "test:*")
        .expect("Failed to clean up test data");
    
    // Test String operations
    let set_result: RedisResult<Value> = redis::cmd("SET")
        .arg("test:string")
        .arg("hello")
        .query(&mut conn);
    assert!(set_result.is_ok());
    
    let value: String = conn.get("test:string").expect("Failed to get string");
    assert_eq!(value, "hello");
    
    // Test List operations
    let push_result: RedisResult<Value> = redis::cmd("LPUSH")
        .arg("test:list")
        .arg("item1")
        .query(&mut conn);
    assert!(push_result.is_ok());
    
    let push_result: RedisResult<Value> = redis::cmd("LPUSH")
        .arg("test:list")
        .arg("item2")
        .query(&mut conn);
    assert!(push_result.is_ok());
    
    let items: Vec<String> = conn.lrange("test:list", 0, -1).expect("Failed to get list items");
    assert_eq!(items, vec!["item2", "item1"]);
    
    // Test Hash operations
    let mut hash = HashMap::new();
    hash.insert("field1", "value1");
    hash.insert("field2", "value2");
    
    // Convert HashMap to Vec of tuples for hset_multiple
    let hash_items: Vec<(&str, &str)> = hash.iter()
        .map(|(&k, &v)| (k, v))
        .collect();
    
    let hset_result: RedisResult<Value> = redis::cmd("HMSET")
        .arg("test:hash")
        .arg(&hash_items.iter().flat_map(|(k, v)| vec![*k, *v]).collect::<Vec<&str>>())
        .query(&mut conn);
    assert!(hset_result.is_ok());
    
    let value: String = conn.hget("test:hash", "field1").expect("Failed to get hash field");
    assert_eq!(value, "value1");
    
    // Test Set operations
    let sadd_result: RedisResult<Value> = redis::cmd("SADD")
        .arg("test:set")
        .arg("member1")
        .query(&mut conn);
    assert!(sadd_result.is_ok());
    
    let sadd_result: RedisResult<Value> = redis::cmd("SADD")
        .arg("test:set")
        .arg("member2")
        .query(&mut conn);
    assert!(sadd_result.is_ok());
    
    let exists: bool = conn.sismember("test:set", "member1").expect("Failed to check set membership");
    assert!(exists);
    
    // Clean up
    cleanup_test_data(&mut conn, "test:*")
        .expect("Failed to clean up test data");
}

#[tokio::test]
async fn test_expiration() {
    let mut conn = get_test_connection()
        .expect("Failed to get Redis connection");
    
    // Clean up any previous test data
    cleanup_test_data(&mut conn, "test:expiry:*")
        .expect("Failed to clean up test data");
    
    // Set key with expiration
    let set_result: RedisResult<Value> = redis::cmd("SET")
        .arg("test:expiry:key")
        .arg("value")
        .query(&mut conn);
    assert!(set_result.is_ok());
    
    let expire_result: RedisResult<Value> = redis::cmd("EXPIRE")
        .arg("test:expiry:key")
        .arg(1)
        .query(&mut conn);
    assert!(expire_result.is_ok());
    
    // Verify key exists
    let exists: bool = conn.exists("test:expiry:key").expect("Failed to check key existence");
    assert!(exists);
    
    // Wait for expiration
    std::thread::sleep(std::time::Duration::from_secs(2));
    
    // Verify key has expired
    let exists: bool = conn.exists("test:expiry:key").expect("Failed to check key existence");
    assert!(!exists);
} 