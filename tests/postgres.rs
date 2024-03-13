use command_notifier::postgres::{
    verify_nsc_user_exists,
    insert_nsc_user,
    delete_nsc_user,
    update_creds_admin,
    update_creds_user,
    get_creds_admin,
    update_account_jwt
};

use std::sync::Arc;
use uuid::Uuid;

mod common;

use common::utils::{get_user_uuid, setup_postgres_client, insert_dummy_nsc_user, cleanup_postgres_user};

#[tokio::test]
async fn test_get_creds_admin() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);
    let uuid = get_user_uuid();
    let creds_admin = "A12345";

    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let _result = insert_dummy_nsc_user( uuid).await;
        let _result = update_creds_admin(Arc::clone(&postgres_client), uuid, creds_admin).await;
        
        let result = get_creds_admin(Arc::clone(&postgres_client), uuid).await;
        assert!(result.is_ok(), "Failed to get creds_admin: {:?}", result);
        let result = result.unwrap();
        assert!(!result.is_empty(), "Creds_admin should not be empty");
        assert!(result == creds_admin, "Creds_admin should be equal to {}", creds_admin);
    }).await;

    cleanup_postgres_user(uuid).await;

    assert!(result.is_ok(), "Failed to get creds_admin: {:?}", result);
}

#[tokio::test]
async fn test_insert_nsc_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let uuid = get_user_uuid();
    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let creds_admin = "A12345";
        let creds_user = "U12345";
        let account_jwt = "JWT.123.456";
        let result = insert_nsc_user(Arc::clone(&postgres_client), uuid, creds_admin, creds_user, account_jwt).await;
        assert!(result.is_ok(), "Failed to insert user: {:?}", result);
        assert!(result.unwrap() == true, "User should have been inserted");

        let rows = Arc::clone(&postgres_client).query("SELECT creds_admin, creds_user, account_jwt FROM nats WHERE id = $1", &[&uuid])
            .await.unwrap();
        assert!(rows.len() > 0, "User should exist");
        // Verify that fields are correct
        let row = rows.get(0);
        let creds_admin_db: String = row.unwrap().get(0);
        let creds_user_db: String = row.unwrap().get(1);
        let account_jwt_db: String = row.unwrap().get(2);
        assert_eq!(creds_admin_db, creds_admin, "Creds_admin should be equal to {}", creds_admin);
        assert_eq!(creds_user_db, creds_user, "Creds_user should be equal to {}", creds_user);
        assert_eq!(account_jwt_db, account_jwt, "Account_jwt should be equal to {}", account_jwt);

    }).await;

    assert!(result.is_ok(), "Failed to insert user: {:?}", result);

    cleanup_postgres_user(get_user_uuid()).await;
}

#[tokio::test]
async fn test_delete_nsc_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let uuid = get_user_uuid();
    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let _result = insert_dummy_nsc_user( uuid).await;

        let result = delete_nsc_user(Arc::clone(&postgres_client), uuid).await;
        assert!(result.is_ok(), "Failed to delete user: {:?}", result);
        assert!(result.unwrap() == true, "User should have been deleted");

        let rows = Arc::clone(&postgres_client).query("SELECT * FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        assert!(rows.len() == 0, "User should not exist")
    }).await;

    assert!(result.is_ok(), "Failed to delete user: {:?}", result);
}

#[tokio::test]
async fn test_update_creds_admin() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let uuid = get_user_uuid();
    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let _result = insert_dummy_nsc_user(uuid).await;

        let creds_admin = "A12345";
        let result = update_creds_admin(Arc::clone(&postgres_client), uuid, creds_admin).await;
        assert!(result.is_ok(), "Failed to update creds_admin: {:?}", result);
        assert!(result.unwrap() == true, "Creds_admin should have been updated");

        let rows = Arc::clone(&postgres_client).query("SELECT creds_admin FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        let row = rows.get(0);
        let creds_admin_db: String = row.unwrap().get(0);
        assert_eq!(creds_admin_db, creds_admin, "Creds_admin should have been updated");
    }).await;

    assert!(result.is_ok(), "Failed to update creds_admin: {:?}", result);

    cleanup_postgres_user(get_user_uuid()).await;
}

#[tokio::test]
async fn test_update_creds_user() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let uuid = get_user_uuid();
    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let _result = insert_dummy_nsc_user(uuid).await;

        let creds_user = "U12345";
        let result = update_creds_user(Arc::clone(&postgres_client), uuid, creds_user).await;
        assert!(result.is_ok(), "Failed to update creds_user: {:?}", result);
        assert!(result.unwrap() == true, "Creds_user should have been updated");

        let rows =Arc::clone(&postgres_client).query("SELECT creds_user FROM nats WHERE id = $1", &[&uuid])
        .await.unwrap();
        let row = rows.get(0);
        let creds_user_db: String = row.unwrap().get(0);
        assert_eq!(creds_user_db, creds_user, "Creds_user should have been updated");
    }).await;

    assert!(result.is_ok(), "Failed to update creds_user: {:?}", result);

    // cleanup_postgres_user(get_user_uuid()).await;
}

#[tokio::test]
async fn test_get_account_jwt() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let uuid = get_user_uuid();
    cleanup_postgres_user(uuid).await;

    let result = tokio::spawn(async move {
        let _result = insert_dummy_nsc_user(uuid).await;

        let account_jwt = "JWT12345";
        let result = update_account_jwt(Arc::clone(&postgres_client), uuid, account_jwt).await;
        assert!(result.is_ok(), "Failed to update account_jwt: {:?}", result);
        assert!(result.unwrap() == true, "Account_jwt should have been updated");

        let rows = Arc::clone(&postgres_client).query("SELECT account_jwt FROM nats WHERE id = $1", &[&uuid])
            .await.unwrap();
        let row = rows.get(0);
        let account_jwt_db: String = row.unwrap().get(0);
        assert_eq!(account_jwt_db, account_jwt, "Account_jwt should have been updated");
    }).await;

    assert!(result.is_ok(), "Failed to update account_jwt: {:?}", result);

    cleanup_postgres_user(get_user_uuid()).await;

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