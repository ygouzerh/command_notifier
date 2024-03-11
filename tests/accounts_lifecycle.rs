use command_notifier::accounts_lifecycle::get_admin_creds_if_not_exists;

use command_notifier::nsc_accounts_utils::get_creds_path;
use command_notifier::postgres::{insert_nsc_user, setup_postgres_client, update_creds_admin, delete_nsc_user};
use uuid::Uuid;

use std::sync::Arc;

#[tokio::test]
async fn test_get_admin_creds_not_exists_should_fail() {
    let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
    let operator_name = "ServerBackend";
    let account_name = "test_acooount";
    let username = "7c278e3c-d344-45a0-a287-9add7253b512";

    let result = get_admin_creds_if_not_exists(creds_base_path, operator_name, account_name, username).await;

    assert!(result.is_err(), "Result should be an error {:?}", result);
}

#[tokio::test]
async fn test_get_admin_creds_not_yet_exists() {

    let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
    let operator_name = "ServerBackend";
    let account_name = "test_account";
    let creds_admin = "A656878dqdqdqwd";
    let username = "7c278ecc-d624-45a0-aa87-9add7253b517";
    let username_uuid = Uuid::parse_str(username).unwrap();

    let postgres_client = Arc::new(setup_postgres_client().await);
    let postgres_client_clone = Arc::clone(&postgres_client);
    
    let result = tokio::spawn(async move {
    
        let _result = insert_nsc_user(Arc::clone(&postgres_client), username_uuid).await;
    
        let _result = update_creds_admin(Arc::clone(&postgres_client), username_uuid, creds_admin).await;

        let result = get_admin_creds_if_not_exists(creds_base_path, operator_name, account_name, username).await;
        
        let creds_path = result.unwrap();

        assert_eq!(creds_path, "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend/test_account/7c278ecc-d624-45a0-aa87-9add7253b517.creds", "Creds path is incorrect");

        // Content of the file should be the same than creds_admin
        let content = std::fs::read_to_string(&creds_path)
            .map_err(|err| format!("Failed to read creds_admin file: {}", err))
            .unwrap();
        assert_eq!(content, creds_admin, "Content of the file is incorrect");

    }).await;

    assert!(result.is_ok(), "Test failed");

    let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

    let _result = std::fs::remove_file(&creds_path);

    let _result = delete_nsc_user(Arc::clone(&postgres_client_clone), username_uuid).await;

}

#[tokio::test]
async fn test_get_admin_creds_already_exists() {

    let creds_base_path = "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds";
    let operator_name = "ServerBackend";
    let account_name = "test_account";
    let username = "7c278ecc-d324-45a0-aa87-9add7253b512";

    let creds_admin = "A656878dqdqdqwd";

    
    let result = tokio::spawn(async move {
        
        let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

        println!("Creds path: {}", creds_path);
        
        std::fs::write(&creds_path, creds_admin)
            .map_err(|err| format!("Failed to write creds_admin to file: {}", err))
            .unwrap();

        let result = get_admin_creds_if_not_exists(creds_base_path, operator_name, account_name, username).await;

        assert!(result.is_ok(), "Failed to get creds path: {:?}", result);

        let creds_path = result.unwrap();

        assert!(!creds_path.is_empty(), "Creds path should not be empty");

        assert_eq!(creds_path, "/Users/yohangouzerh/.local/share/nats/nsc/keys/creds/ServerBackend/test_account/7c278ecc-d324-45a0-aa87-9add7253b512.creds", "Creds path is incorrect");

        // Content of the file should be the same than creds_admin
        let content = std::fs::read_to_string(&creds_path)
            .map_err(|err| format!("Failed to read creds_admin file: {}", err))
            .unwrap();
        assert_eq!(content, creds_admin, "Content of the file is incorrect");
    }).await;

    // Remove creds file

    let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

    let _result = std::fs::remove_file(&creds_path);

    assert!(result.is_ok(), "Test failed");
}