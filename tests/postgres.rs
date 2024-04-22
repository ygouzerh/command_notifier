use command_notifier::postgres::{
    delete_nsc_user_from_postgres,
    get_creds_admin,
    insert_nsc_user,
    update_account_jwt,
    update_creds_admin,
    update_creds_user,
    verify_nsc_user_exists,
    add_api_key,
    delete_api_key,
    verify_api_key
};

use std::sync::Arc;
use uuid::Uuid;

mod common;

use common::utils::{get_user_uuid, setup_postgres_client, insert_dummy_nsc_user, cleanup_postgres_user};

#[tokio::test]
async fn test_delete_api_key() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);
    let user_id = get_user_uuid();
    let api_key_value = "APIKEY123";

    let result = add_api_key(Arc::clone(&postgres_client), user_id, api_key_value).await;
    assert!(result.is_ok(), "Failed to add api key: {:?}", result);

    let query_result = postgres_client.query("SELECT id FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    let row = query_result.get(0);

    assert!(row.is_some(), "Api key should exist");

    let api_key_id: Uuid = row.unwrap().get(0);
    let result = delete_api_key(Arc::clone(&postgres_client), api_key_id).await;
    assert!(result.is_ok(), "Failed to delete api key: {:?}", result);

    let query_result = postgres_client.query("SELECT id FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    let row = query_result.get(0);

    assert!(row.is_none(), "Api key should not exist");


}

#[tokio::test]
async fn test_verify_api_key() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);
    let postgres_client_two = Arc::clone(&postgres_client);
    let postgres_client_three = Arc::clone(&postgres_client);
    let user_id = get_user_uuid();
    let api_key_value = "APIKEY123";

    let result = add_api_key(Arc::clone(&postgres_client), user_id, api_key_value).await;
    assert!(result.is_ok(), "Failed to add api key: {:?}", result);

    let result_test = tokio::spawn(async move {
        let result = verify_api_key(Arc::clone(&postgres_client), user_id, api_key_value).await;
        assert!(result.is_ok(), "Failed to verify api key: {:?}", result);
        assert!(result.unwrap() == true, "Api key verifiction should be successfull");
    }).await;

    // Cleanup
    let query_result = postgres_client_two.query("SELECT id FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    let row = query_result.get(0);
    let api_key_id: Uuid = row.unwrap().get(0);
    let result_deletion = delete_api_key(Arc::clone(&postgres_client_three), api_key_id).await;

    assert!(result_deletion.is_ok(), "Failed to delete api key: {:?}", result_deletion);
    assert!(result_test.is_ok(), "Failed to verify api key: {:?}", result_test);

}

#[tokio::test]
async fn test_add_multiple_apis() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);
    let user_id = get_user_uuid();
    let api_key_value = "APIKEY123";

    for _ in 0..3 {
        let result = add_api_key(Arc::clone(&postgres_client), user_id, api_key_value).await;
        assert!(result.is_ok(), "Failed to add api key: {:?}", result);
    }

    // Verify that api key is added
    let query_result = postgres_client.query("SELECT id, api_key_hash FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    assert!(query_result.len() == 3, "API Keys number should be 3");

    // Verify that the content of the api_keys_hash is well different for the 3 rows
    let mut api_keys_hash: Vec<String> = Vec::new();
    for row in query_result.iter() {
        let api_key_hash: String = row.get(1);
        api_keys_hash.push(api_key_hash);
    }

    // Cleanup
    let mut results: Vec<Result<bool, tokio_postgres::Error>> = Vec::new();
    for row in query_result.iter() {
        let api_key_id: Uuid = row.get(0);
        let result = delete_api_key(Arc::clone(&postgres_client), api_key_id).await;
        results.push(result);
    }

    let all_ok = results.into_iter().all(|x| x.is_ok());
    assert!(all_ok, "Failed to delete api key");

}

#[tokio::test]
async fn test_add_one_api() {
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);
    let user_id = get_user_uuid();
    let api_key_value = "APIKEY123";

    let result = add_api_key(Arc::clone(&postgres_client), user_id, api_key_value).await;
    assert!(result.is_ok(), "Failed to add api key: {:?}", result);

    // Verify that api key is added
    let query_result = postgres_client.query("SELECT id, api_key_hash FROM api_keys WHERE user_id = $1", &[&user_id])
        .await
        .unwrap();
    assert!(query_result.len() > 0, "No api key found");
    let row = query_result.get(0);
    let api_key_id: Uuid = row.unwrap().get(0);
    let api_key_hash: String = row.unwrap().get(1);

    // Verify that the api key hash is correct using bcrypt
    let result = bcrypt::verify(api_key_value, &api_key_hash)
        .map_err(|err| format!("Failed to run bcrypt api key: {}", err));

    assert!(result.is_ok(), "Api key hash inserted is not correct: {:?}", result);

    // Cleanup
    let result = delete_api_key(Arc::clone(&postgres_client), api_key_id).await;
    assert!(result.is_ok(), "Failed to delete api key: {:?}", result);

}

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
        let nsc_account_id = "ABCDEF12";
        let creds_admin = "A12345";
        let creds_user = "U12345";
        let account_jwt = "JWT.123.456";
        let result = insert_nsc_user(Arc::clone(&postgres_client), uuid, nsc_account_id, creds_admin, creds_user, account_jwt).await;
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

        let result = delete_nsc_user_from_postgres(Arc::clone(&postgres_client), uuid).await;
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