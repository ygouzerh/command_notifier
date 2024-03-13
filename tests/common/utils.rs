use std::sync::Arc;

use command_notifier::postgres::insert_nsc_user;
use tokio_postgres::NoTls;
use uuid::Uuid;
use std::process::Command;

#[cfg(test)]
pub fn get_user_uuid() -> Uuid {
    // yohan.gouzerh+test@outlook.com
    Uuid::parse_str("7c278ecc-d624-45a0-aa87-9add7253b517").unwrap()
}

#[cfg(test)]
pub async fn insert_dummy_nsc_user(user_id: Uuid) -> Result<(), String>{
    let postgres_client = setup_postgres_client().await;
    let creds_admin = "creds_admin_dummy";
    let creds_user = "creds_user_dummy";
    let account_jwt = "account_jwt_dummy";
    let _result = insert_nsc_user(Arc::new(postgres_client), user_id, creds_admin, creds_user, account_jwt)
        .await
        .map_err(|err| format!("Failed to insert user: {}", err))?;
    Ok(())
}

#[cfg(test)]
pub async fn setup_postgres_client() -> tokio_postgres::Client {
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
pub async fn cleanup_postgres_user(user_id: Uuid) {
    let postgres_client = setup_postgres_client().await;
    let result = postgres_client.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
        .await;
    assert!(result.is_ok(), "Failed to delete user: {:?}", result);
}

#[cfg(test)]
pub fn cleanup_nsc_account(account_name: &str) {
    // Clean up the generated account and user

    let _ = Command::new("nsc")
        .arg("delete")
        .arg("account")
        .arg(account_name)
        .output();
}

#[cfg(test)]
pub fn cleanup_nsc_user(account_name: &str, username: &str) {
    // Clean up the generated account and user
    
    let _ = Command::new("nsc")
        .arg("delete")
        .arg("user")
        .arg("--account")
        .arg(account_name)
        .arg(username)
        .output();
}