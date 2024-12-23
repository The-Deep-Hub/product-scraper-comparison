use mongodb::{Client, Database};
use dotenv::dotenv;
use std::env;

/// Gets a database connection with application user credentials
pub async fn get_test_db() -> Database {
    dotenv().ok();
    
    let database = env::var("MONGO_DATABASE").expect("MONGO_DATABASE must be set");
    let uri = format!(
        "mongodb://{}:{}@localhost:27017/{}",
        env::var("MONGO_APP_USERNAME").expect("MONGO_APP_USERNAME must be set"),
        env::var("MONGO_APP_PASSWORD").expect("MONGO_APP_PASSWORD must be set"),
        database
    );
    
    let client = Client::with_uri_str(&uri)
        .await
        .expect("Failed to create MongoDB client");
    
    client.database(&database)
}

/// Gets a database connection with admin credentials
pub async fn get_admin_db() -> Database {
    dotenv().ok();
    
    let database = env::var("MONGO_DATABASE").expect("MONGO_DATABASE must be set");
    let uri = format!(
        "mongodb://{}:{}@localhost:27017/admin?authSource=admin",
        env::var("MONGO_ROOT_USERNAME").expect("MONGO_ROOT_USERNAME must be set"),
        env::var("MONGO_ROOT_PASSWORD").expect("MONGO_ROOT_PASSWORD must be set")
    );
    
    let client = Client::with_uri_str(&uri)
        .await
        .expect("Failed to create MongoDB client");
    
    client.database(&database)
}

/// Cleans up test data for a given collection and filter
pub async fn cleanup_test_data(db: &Database, collection: &str, filter: bson::Document) {
    db.collection::<bson::Document>(collection)
        .delete_many(filter, None)
        .await
        .expect("Failed to clean up test data");
} 