use uuid::Uuid;

use crate::nsc_accounts_utils::{check_if_creds_exists, create_nsc_account, create_nsc_user, delete_nsc_account, delete_nsc_user, get_account_jwt, get_creds_path};
use crate::postgres::{
    delete_nsc_user_from_postgres, get_creds_admin, insert_nsc_user, setup_postgres_client, verify_nsc_user_exists
};

use std::sync::Arc;

pub async fn create_and_insert_user(postgres_client: Arc<tokio_postgres::Client>, creds_base_path: &str, operator_name: &str, username: Uuid) -> Result<(), String> {
    // Assumption: username is not in nats table yet + username in auth table already

    let account_name = username.to_string();

    let nsc_account_id = create_nsc_account(&account_name)
        .map_err(|err| format!("Failed to create nsc account: {}", err))?;

    create_nsc_user(&account_name, "user_01")
        .map_err(|err| format!("Failed to create nsc user: {}", err))?;
    
    let creds_path_user = get_creds_path(creds_base_path, operator_name, &account_name, "user_01");

    let creds_user_content = std::fs::read_to_string(&creds_path_user)
        .map_err(|err| format!("Failed to read creds_user file: {}", err))?;

    create_nsc_user(&account_name, "admin_01")
        .map_err(|err| format!("Failed to create nsc user: {}", err))?;

    let creds_path_admin = get_creds_path(creds_base_path, operator_name, &account_name, "admin_01");

    let creds_admin_content = std::fs::read_to_string(&creds_path_admin)
        .map_err(|err| format!("Failed to read creds_user file: {}", err))?;

    let account_jwt = get_account_jwt(&account_name)
        .map_err(|err| format!("Failed to get account jwt: {}", err))?;
    
    insert_nsc_user(postgres_client, username, &nsc_account_id, &creds_admin_content, &creds_user_content, &account_jwt)
        .await
        .map_err(|err| format!("Failed to insert nsc user into the database : {}", err))?;

    Ok(())
}

pub async fn delete_user_everywhere(postgres_client: Arc<tokio_postgres::Client>, creds_base_path: &str, operator_name: &str, username: Uuid) -> Result<(), String> {
    let account_name = username.to_string();
    let nsc_username_admin = "admin_01";
    let nsc_username_user = "user_01";

    let _result = delete_nsc_user(&account_name, nsc_username_admin);
    let _result = delete_nsc_user(&account_name, nsc_username_user);
    let _result = delete_nsc_account(&account_name);
    let _result = std::fs::remove_file(get_creds_path(&creds_base_path, operator_name, &account_name, &nsc_username_admin));
    let _result = std::fs::remove_file(get_creds_path(&creds_base_path, operator_name, &account_name, &nsc_username_user));
    let _result = delete_nsc_user_from_postgres(Arc::clone(&postgres_client), username)
        .await;
    Ok(())
}

pub async fn get_admin_creds_if_not_exists(creds_base_path: &str, operator_name: &str, account_name: &str) -> Result<String, String>{
    // This function will check if the admin_creds are already downloaded under creds_path/uuid or not, otherwise it will pull them from the database
    
    let user_uuid = Uuid::parse_str(account_name)
        .map_err(|err| format!("Failed to parse account name as an uuid: {}", err))?;

    let username = "admin_01";

    if let Ok(creds_path) = check_if_creds_exists(creds_base_path, operator_name, account_name, username) {
        return Ok(creds_path);
    }
    
    let postgres_client = Arc::new(setup_postgres_client().await);

    let creds_admin = get_creds_admin(Arc::clone(&postgres_client), user_uuid)
        .await
        .map_err(|err| format!("Failed to get creds_admin: {}", err))?;

    let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

    std::fs::write(&creds_path, creds_admin)
        .map_err(|err| format!("Failed to write creds_admin to file: {}", err))?;

    Ok(creds_path)
}