use std::sync::Arc;
use uuid::Uuid;
use tokio_postgres::NoTls;

// Schema of nats table
// id / nsc_account_id / creds_admin / creds_user / account_jwt / created_at 

pub async fn setup_postgres_client() -> tokio_postgres::Client {
    // TODO: See if need to pass connection string in environment here or not
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

pub async fn verify_nsc_user_exists(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    // Check if user exists in the database
    let rows = postgres_client.query("SELECT * FROM nats WHERE id = $1", &[&user_id])
        .await?;
    Ok(rows.len() > 0)
}

// creds_admin / creds_user / account_jwt / created_at 
pub async fn insert_nsc_user(
    postgres_client: Arc<tokio_postgres::Client>,
    user_id: Uuid,
    nsc_account_id: &str,
    creds_admin: &str,
    creds_user: &str,
    account_jwt: &str
) -> Result<bool, tokio_postgres::Error>{
    // let result = postgres_client.execute("INSERT INTO nats (id) VALUES ($1)", &[&user_id])
        // .await?;
    let result = postgres_client.execute("INSERT INTO nats (id, nsc_account_id, creds_admin, creds_user, account_jwt) VALUES ($1, $2, $3, $4, $5)", &[&user_id, &nsc_account_id, &creds_admin, &creds_user, &account_jwt])
        .await?;
    Ok(result > 0)
}

pub async fn delete_nsc_user_from_postgres(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
        .await?;
    Ok(result > 0)
}

pub async fn get_creds_admin(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<String, String>{
    let rows = postgres_client.query("SELECT creds_admin FROM nats WHERE id = $1", &[&user_id])
        .await
        .map_err(|err| format!("Failed to run query: {}", err));

    let rows = rows.unwrap();
    let row = rows.get(0);
    if let None = row {
        return Err("No rows found".to_string());
    }
    let creds_admin: String = row.unwrap().get(0);
    Ok(creds_admin)
}

pub async fn update_creds_admin(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid, creds_admin: &str) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("UPDATE nats SET creds_admin = $1 WHERE id = $2", &[&creds_admin, &user_id])
        .await?;
    Ok(result > 0)
}

pub async fn update_creds_user(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid, creds_user: &str) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("UPDATE nats SET creds_user = $1 WHERE id = $2", &[&creds_user, &user_id])
        .await?;
    Ok(result > 0)
}

pub async fn update_account_jwt(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid, account_jwt: &str) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("UPDATE nats SET account_jwt = $1 WHERE id = $2", &[&account_jwt, &user_id])
        .await?;
    Ok(result > 0)
}

pub async fn get_api_key(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<String, tokio_postgres::Error>{
    todo!()
}