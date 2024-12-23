use bson::doc;
use super::helpers::{get_admin_db, get_test_db};

#[tokio::test]
async fn test_mongodb_connection() {
    // First ensure we can connect as admin
    let admin_db = get_admin_db().await;
    let admin_result = admin_db.run_command(doc! {"ping": 1}, None)
        .await
        .expect("Failed to ping database as admin");
    assert_eq!(admin_result.get_f64("ok").unwrap(), 1.0);

    // Then test application user connection
    let app_db = get_test_db().await;
    let app_result = app_db.run_command(doc! {"ping": 1}, None)
        .await
        .expect("Failed to ping database as application user");
    assert_eq!(app_result.get_f64("ok").unwrap(), 1.0);
} 