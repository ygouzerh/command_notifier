use uuid::Uuid;

use crate::nsc_accounts_utils::{check_if_creds_exists, create_nsc_account, create_nsc_user, get_creds_path, get_account_jwt};
use crate::postgres::{
    get_creds_admin, insert_nsc_user, setup_postgres_client, update_creds_admin, update_creds_user, verify_nsc_user_exists
};

use std::sync::Arc;

pub async fn create_and_insert_user(creds_base_path: &str, operator_name: &str, username: Uuid) -> Result<(), String> {
    // Assumption: username is not in nats table yet + username in auth table already

    let postgres_client = Arc::new(setup_postgres_client().await);

    let account_name = username.to_string();

    create_nsc_account(&account_name)
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

    // Change the 3 function to only one

    // insert_nsc_user(Arc::clone(&postgres_client), username)
    //     .await
    //     .map_err(|err| format!("Failed to insert nsc user into the database : {}", err))?;

    update_creds_admin(Arc::clone(&postgres_client), username, creds_admin_content.as_str())
        .await
        .map_err(|err| format!("Failed to update creds_admin in the database : {}", err))?;

    update_creds_user(Arc::clone(&postgres_client), username, creds_user_content.as_str())
        .await
        .map_err(|err| format!("Failed to update creds_user in the database : {}", err))?;

    Ok(())
}

pub async fn get_admin_creds_if_not_exists(creds_base_path: &str, operator_name: &str, account_name: &str, username: &str) -> Result<String, String>{
    // TODO: Suffix them by _admin
    
    if let Ok(creds_path) = check_if_creds_exists(creds_base_path, operator_name, account_name, username) {
        return Ok(creds_path);
    }
    
    let user_uuid = Uuid::parse_str(username)
        .map_err(|err| format!("Failed to parse user uuid: {}", err))?;

    let postgres_client = Arc::new(setup_postgres_client().await);

    verify_nsc_user_exists(Arc::clone(&postgres_client), user_uuid)
        .await
        .map_err(|err| format!("Failed to verify user exists: {}", err))?;

    let creds_admin = get_creds_admin(Arc::clone(&postgres_client), user_uuid)
        .await
        .map_err(|err| format!("Failed to get creds_admin: {}", err))?;

    let creds_path = get_creds_path(creds_base_path, operator_name, account_name, username);

    std::fs::write(&creds_path, creds_admin)
        .map_err(|err| format!("Failed to write creds_admin to file: {}", err))?;

    Ok(creds_path)
}