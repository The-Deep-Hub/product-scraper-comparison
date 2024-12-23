use bson::{doc, Document, DateTime};
use super::helpers::{get_test_db, cleanup_test_data};

#[tokio::test]
async fn test_user_collection_operations() {
    let db = get_test_db().await;
    let collection = db.collection::<Document>("users");
    
    // Test data
    let test_user = doc! {
        "email": "test@example.com",
        "name": "Test User",
        "created_at": DateTime::now()
    };
    
    // Clean up any previous test data
    cleanup_test_data(&db, "users", doc! {"email": "test@example.com"}).await;
    
    // Test Insert
    let insert_result = collection.insert_one(test_user.clone(), None)
        .await
        .expect("Failed to insert test user");
    assert!(insert_result.inserted_id.as_object_id().is_some());
    
    // Test Read
    let found_user = collection.find_one(doc! {"email": "test@example.com"}, None)
        .await
        .expect("Failed to find user")
        .expect("User not found");
    assert_eq!(found_user.get_str("email").unwrap(), "test@example.com");
    
    // Test Update
    let update_result = collection.update_one(
        doc! {"email": "test@example.com"},
        doc! {"$set": {"name": "Updated Test User"}},
        None
    )
    .await
    .expect("Failed to update user");
    assert_eq!(update_result.modified_count, 1);
    
    // Test Delete
    let delete_result = collection.delete_one(doc! {"email": "test@example.com"}, None)
        .await
        .expect("Failed to delete user");
    assert_eq!(delete_result.deleted_count, 1);
}

#[tokio::test]
async fn test_unique_email_constraint() {
    let db = get_test_db().await;
    let collection = db.collection::<Document>("users");
    
    // Clean up any previous test data
    cleanup_test_data(&db, "users", doc! {"email": "duplicate@example.com"}).await;
    
    // Insert first user
    let test_user1 = doc! {
        "email": "duplicate@example.com",
        "name": "First User"
    };
    collection.insert_one(test_user1, None)
        .await
        .expect("Failed to insert first test user");
    
    // Try to insert second user with same email
    let test_user2 = doc! {
        "email": "duplicate@example.com",
        "name": "Second User"
    };
    let result = collection.insert_one(test_user2, None).await;
    
    // Should fail due to unique email constraint
    assert!(result.is_err());
    
    // Clean up
    cleanup_test_data(&db, "users", doc! {"email": "duplicate@example.com"}).await;
} 