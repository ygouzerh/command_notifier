use uuid::Uuid;

use crate::nsc_accounts_utils::{check_if_creds_exists, get_creds_path};
use crate::postgres::{
    get_creds_admin,
    setup_postgres_client,
    verify_nsc_user_exists
};

use std::sync::Arc;


pub async fn get_admin_creds_if_not_exists(creds_base_path: &str, operator_name: &str, account_name: &str, username: &str) -> Result<String, String>{
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