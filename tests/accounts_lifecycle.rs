use command_notifier::accounts_lifecycle::{
    get_admin_creds_if_not_exists,
    create_and_insert_user,
    delete_user_everywhere
};

use command_notifier::nsc_accounts_utils::{delete_nsc_account, delete_nsc_user, get_creds_path};
use command_notifier::postgres::{delete_nsc_user_from_postgres, setup_postgres_client, update_creds_admin};
use uuid::Uuid;

use std::env;
use std::sync::Arc;

mod common;

use common::utils::{insert_dummy_nsc_user, cleanup_postgres_user, get_user_uuid, check_if_jwt};

#[cfg(test)]
fn delete_creds_files_of_full_user(creds_base_path: &str, operator_name: &str, account_name: &str) {
    let creds_path_user = get_creds_path(creds_base_path, operator_name, account_name, "user_01");
    let creds_path_admin = get_creds_path(creds_base_path, operator_name, account_name, "admin_01");

    let _result = std::fs::remove_file(creds_path_user);
    let _result = std::fs::remove_file(creds_path_admin);
}

#[tokio::test]
async fn test_create_and_insert_user() {
    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    let username = get_user_uuid();

    let posgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(posgres_client);

    cleanup_user(&username.to_string(), &operator_name, &username.to_string()).await;
    delete_creds_files_of_full_user(&creds_base_path, &operator_name, &username.to_string());

    let operator_name_cloned = operator_name.to_owned();
    let creds_base_path_cloned = creds_base_path.to_owned();

    let result = tokio::spawn(async move {

        let result = create_and_insert_user(Arc::clone(&postgres_client), &creds_base_path, &operator_name, username).await;
        
        assert!(result.is_ok(), "Failed to create and insert user: {:?}", result);
        
        // Query the database to extract all the fields
        let rows = Arc::clone(&postgres_client).query("SELECT nsc_account_id, creds_admin, creds_user, account_jwt FROM nats WHERE id = $1", &[&username])
            .await
            .unwrap();
    
        // Extract the fields
        let nsc_account_id: String = rows[0].get(0);
        let creds_admin_db: String = rows[0].get(1);
        let creds_user_db: String = rows[0].get(2);
        let account_jwt_db: String = rows[0].get(3);

        assert!(!nsc_account_id.is_empty(), "Nsc_account_id should not be empty");
        assert!(!creds_admin_db.is_empty(), "Creds_admin should not be empty");
        assert!(!creds_user_db.is_empty(), "Creds_user should not be empty");
        assert!(check_if_jwt(&account_jwt_db), "Account jwt is not a jwt");
    }).await;

    // cleanup_user(&username.to_string(), &operator_name_cloned, &username.to_string()).await;

    delete_creds_files_of_full_user(&creds_base_path_cloned, &operator_name_cloned, &username.to_string());

    assert!(result.is_ok(), "Test failed");
    
}

#[tokio::test]
async fn test_delete_user_everywhere() {
    use command_notifier::nsc_accounts_utils::check_if_creds_exists;

    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    let username = get_user_uuid();
    let account_name = username.to_string();

    let posgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(posgres_client);

    cleanup_user(&username.to_string(), &operator_name, &username.to_string()).await;
    delete_creds_files_of_full_user(&creds_base_path, &operator_name, &username.to_string());

    let operator_name_cloned = operator_name.to_owned();
    let creds_base_path_cloned = creds_base_path.to_owned();

    let result = tokio::spawn(async move {

        let result = create_and_insert_user(Arc::clone(&postgres_client), &creds_base_path, &operator_name, username).await;
        
        assert!(result.is_ok(), "Failed to create and insert user: {:?}", result);

        let result = delete_user_everywhere(Arc::clone(&postgres_client), &creds_base_path, &operator_name, username).await;
        
        assert!(result.is_ok(), "Failed to delete user: {:?}", result);

        println!("Account name: {}", account_name);

        let output = std::process::Command::new("nsc")
            .arg("describe")
            .arg("user")
            .arg("user_01")
            .arg("--account")
            .arg(&account_name)
            .output()
            .unwrap();
        assert!(!output.status.success(), "nsc describe user user_01 should fail");

        let output = std::process::Command::new("nsc")
            .arg("describe")
            .arg("user")
            .arg("admin_01")
            .arg("--account")
            .arg(&account_name)
            .output()
            .unwrap();
        assert!(!output.status.success(), "nsc describe user user_01 should fail");


        let output = std::process::Command::new("nsc")
            .arg("describe")
            .arg("account")
            .arg(&&account_name)
            .output()
            .unwrap();
        assert!(!output.status.success(), "nsc describe user user_01 should fail");


        let result = check_if_creds_exists(&creds_base_path, &operator_name, &account_name, "user_01");

        assert!(result.is_err(), "Creds file of user_01 should not exist");

        let result = check_if_creds_exists(&creds_base_path, &operator_name, &account_name, "admin_01");

        assert!(result.is_err(), "Creds file of admin_01 should not exist");

        // Query the database to extract all the fields
        let rows = Arc::clone(&postgres_client).query("SELECT * FROM nats WHERE id = $1", &[&username])
            .await
            .unwrap();

        assert!(rows.len() == 0, "User should not exist in the database");

    }).await;

    cleanup_user(&username.to_string(), &operator_name_cloned, &username.to_string()).await;

    delete_creds_files_of_full_user(&creds_base_path_cloned, &operator_name_cloned, &username.to_string());

    assert!(result.is_ok(), "Test failed");
}

#[cfg(test)]
async fn cleanup_user(username: &str, operator_name:&str, account_name: &str) {
    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let username_uuid = Uuid::parse_str(username).unwrap();

    let _result = delete_nsc_user(account_name, username);
    let _result = delete_nsc_account(username);
    let _result = std::fs::remove_file(get_creds_path(&creds_base_path, operator_name, account_name, username));
    let _result = cleanup_postgres_user(username_uuid).await;
}

#[tokio::test]
async fn test_get_admin_creds_not_exists_should_fail() {
    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    let account_name = "test_acooount";
    let username = "7c278e3c-d344-45a0-a287-9add7253b512";

    cleanup_user(username, &operator_name, account_name).await;

    let result = get_admin_creds_if_not_exists(&creds_base_path, &operator_name, account_name, username).await;

    assert!(result.is_err(), "Result should be an error {:?}", result);
}

#[tokio::test]
async fn test_get_admin_creds_not_yet_exists() {

    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");

    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    let account_name = "test_account";
    let creds_admin = "A656878dqdqdqwd";
    let username = "7c278ecc-d624-45a0-aa87-9add7253b517";
    let username_uuid = Uuid::parse_str(username).unwrap();

    let postgres_client = Arc::new(setup_postgres_client().await);
    let postgres_client_clone = Arc::clone(&postgres_client);
    
    let creds_base_path_cloned = creds_base_path.to_owned();

    let operator_name_cloned = operator_name.to_owned();

    cleanup_user(username, &operator_name, account_name).await;
    
    let result = tokio::spawn(async move {
        
        let _result = insert_dummy_nsc_user(username_uuid).await;
        
        let _result = update_creds_admin(Arc::clone(&postgres_client), username_uuid, creds_admin).await;
        
        let result = get_admin_creds_if_not_exists(&creds_base_path, &operator_name, account_name, username).await;
        
        let creds_path = result.unwrap();
        
        let correct_base_path = format!("{}/{}/{}/{}.creds", creds_base_path, operator_name, account_name, username);
        
        assert_eq!(creds_path, correct_base_path, "Creds path is incorrect");
        
        // Content of the file should be the same than creds_admin
        let content = std::fs::read_to_string(&creds_path)
        .map_err(|err| format!("Failed to read creds_admin file: {}", err))
        .unwrap();
    assert_eq!(content, creds_admin, "Content of the file is incorrect");
    
}).await;

cleanup_user(username, &operator_name_cloned, account_name).await;

assert!(result.is_ok(), "Test failed");

let creds_path = get_creds_path(creds_base_path_cloned.as_str(), &operator_name_cloned, account_name, username);

let _result = std::fs::remove_file(&creds_path);

let _result = delete_nsc_user_from_postgres(Arc::clone(&postgres_client_clone), username_uuid).await;

}

#[tokio::test]
async fn test_get_admin_creds_already_exists() {
    
    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let creds_base_path_cloned = creds_base_path.to_owned();

    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    let account_name = "test_account";
    let username = "7c278ecc-d324-45a0-aa87-9add7253b512";

    let creds_admin = "A656878dqdqdqwd";

    cleanup_user(username, &operator_name, account_name).await;
    
    let operator_name_cloned = operator_name.to_owned();
    
    let result = tokio::spawn(async move {
        
        let creds_path = get_creds_path(&creds_base_path, &operator_name, account_name, username);
        
        println!("Creds path: {}", creds_path);
        
        std::fs::write(&creds_path, creds_admin)
        .map_err(|err| format!("Failed to write creds_admin to file: {}", err))
        .unwrap();
    
    let result = get_admin_creds_if_not_exists(&creds_base_path, &operator_name, account_name, username).await;
    
    assert!(result.is_ok(), "Failed to get creds path: {:?}", result);
    
    let creds_path = result.unwrap();
    
    assert!(!creds_path.is_empty(), "Creds path should not be empty");
    
    let correct_base_path = format!("{}/{}/{}/{}.creds", creds_base_path, operator_name, account_name, username);
    
    assert_eq!(creds_path, correct_base_path, "Creds path is incorrect");
    
    // Content of the file should be the same than creds_admin
    let content = std::fs::read_to_string(&creds_path)
    .map_err(|err| format!("Failed to read creds_admin file: {}", err))
    .unwrap();
        assert_eq!(content, creds_admin, "Content of the file is incorrect");
    }).await;

    cleanup_user(username, &operator_name_cloned, account_name).await;

    // Remove creds file
    let creds_path = get_creds_path(creds_base_path_cloned.as_str(), &operator_name_cloned, account_name, username);

    let _result = std::fs::remove_file(&creds_path);

    assert!(result.is_ok(), "Test failed");
}