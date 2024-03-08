use command_notifier::postgres::{
    verify_nsc_user_exists,
    insert_nsc_user,
    delete_nsc_user,
    update_creds_admin,
    update_creds_user,
    get_creds_admin
};
use std::sync::Arc;
use tokio_postgres::NoTls;
use uuid::Uuid;
use std::panic;

#[cfg(test)]
fn get_user_uuid() -> Uuid {
    // yohan.gouzerh+test@outlook.com
    Uuid::parse_str("7c278ecc-d624-45a0-aa87-9add7253b517").unwrap()
}

#[cfg(test)]
async fn setup_postgres_client() -> tokio_postgres::Client {
    use std::env;

    let database_connection_string = env::var("DATABASE_CONNECTION_STRING").expect("DATABASE_CONNECTION_STRING must be set");
    let (postgres_client, connection) =
        tokio_postgres::connect(&database_connection_string, NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    postgres_client

}

#[cfg(test)]
async fn cleanup_user(user_id: Uuid) {
    let postgres_client = setup_postgres_client().await;
    let result = postgres_client.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
        .await;
    assert!(result.is_ok(), "Failed to delete user: {:?}", result);
}

#[tokio::test]
async fn test_get_creds_admin() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client_02 = setup_postgres_client().await;
    let postgres_client_03 = setup_postgres_client().await;
    let uuid = get_user_uuid();
    let creds_admin = "A12345";

    let result = tokio::spawn(async move {
        let _result = insert_nsc_user(Arc::new(postgres_client), uuid).await;
        let _result = update_creds_admin(Arc::new(postgres_client_02), uuid, creds_admin).await;
        
        let result = get_creds_admin(Arc::new(postgres_client_03), uuid).await;
        assert!(result.is_ok(), "Failed to get creds_admin: {:?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Creds_admin should not be empty");
        assert!(result == creds_admin, "Creds_admin should be equal to {}", creds_admin);
    }).await;

    cleanup_user(uuid).await;

    assert!(result.is_ok(), "Failed to get creds_admin: {:?}", result);
}

#[tokio::test]
async fn test_insert_nsc_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client_02 = setup_postgres_client().await;
    let result = tokio::spawn(async move {
        let uuid = get_user_uuid();
        let result = insert_nsc_user(Arc::new(postgres_client), uuid).await;
        assert!(result.is_ok(), "Failed to insert user: {:?}", result);
        assert!(result.unwrap() == true, "User should have been inserted");

        let rows = postgres_client_02.query("SELECT * FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        assert!(rows.len() > 0, "User should exist")
    }).await;

    assert!(result.is_ok(), "Failed to insert user: {:?}", result);

    cleanup_user(get_user_uuid()).await;
}

#[tokio::test]
async fn test_delete_nsc_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client_02 = setup_postgres_client().await;
    let postgres_client_03 = setup_postgres_client().await;
    let result = tokio::spawn(async move {
        let uuid = get_user_uuid();
        let _result = insert_nsc_user(Arc::new(postgres_client), uuid).await;

        let result = delete_nsc_user(Arc::new(postgres_client_02), uuid).await;
        assert!(result.is_ok(), "Failed to delete user: {:?}", result);
        assert!(result.unwrap() == true, "User should have been deleted");

        let rows = postgres_client_03.query("SELECT * FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        assert!(rows.len() == 0, "User should not exist")
    }).await;

    assert!(result.is_ok(), "Failed to delete user: {:?}", result);
}

#[tokio::test]
async fn test_update_creds_admin() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client_02 = setup_postgres_client().await;
    let postgres_client_03 = setup_postgres_client().await;
    let result = tokio::spawn(async move {
        let uuid = get_user_uuid();
        let _result = insert_nsc_user(Arc::new(postgres_client_02), uuid).await;

        let creds_admin = "A12345";
        let result = update_creds_admin(Arc::new(postgres_client), uuid, creds_admin).await;
        assert!(result.is_ok(), "Failed to update creds_admin: {:?}", result);
        assert!(result.unwrap() == true, "Creds_admin should have been updated");

        let rows = postgres_client_03.query("SELECT creds_admin FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        let row = rows.get(0);
        let creds_admin_db: String = row.unwrap().get(0);
        assert_eq!(creds_admin_db, creds_admin, "Creds_admin should have been updated");
    }).await;

    assert!(result.is_ok(), "Failed to update creds_admin: {:?}", result);

    cleanup_user(get_user_uuid()).await;
}

#[tokio::test]
async fn test_update_creds_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client_02 = setup_postgres_client().await;
    let postgres_client_03 = setup_postgres_client().await;
    let result = tokio::spawn(async move {
        let uuid = get_user_uuid();
        let _result = insert_nsc_user(Arc::new(postgres_client_02), uuid).await;

        let creds_user = "U12345";
        let result = update_creds_user(Arc::new(postgres_client), uuid, creds_user).await;
        assert!(result.is_ok(), "Failed to update creds_user: {:?}", result);
        assert!(result.unwrap() == true, "Creds_user should have been updated");

        let rows = postgres_client_03.query("SELECT creds_user FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        let row = rows.get(0);
        let creds_user_db: String = row.unwrap().get(0);
        assert_eq!(creds_user_db, creds_user, "Creds_user should have been updated");
    }).await;

    assert!(result.is_ok(), "Failed to update creds_user: {:?}", result);

    // cleanup_user(get_user_uuid()).await;
}


#[tokio::test]
async fn test_check_user_well_not_exists() {
    let postgres_client = setup_postgres_client().await;
    let uuid =  Uuid::parse_str("6f422cbc-b2d5-43eb-b61a-9c7c892d2eb2").unwrap();
    let result = verify_nsc_user_exists(Arc::new(postgres_client), uuid).await;
    assert!(result.is_ok(), "Failed to verify user exists: {:?}", result);
    assert!(result.unwrap() == false, "User should not exist");
}

#[tokio::test]
async fn test_check_user_well_exists() {
    let postgres_client = setup_postgres_client().await;
    let uuid =  Uuid::parse_str("6f462cbc-b2d5-43eb-b61a-9c7c892d2eb2").unwrap();
    let result = verify_nsc_user_exists(Arc::new(postgres_client), uuid).await;
    assert!(result.is_ok(), "Failed to verify user exists: {:?}", result);
    assert!(result.unwrap() == true, "User should exist");
}