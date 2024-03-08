use std::sync::Arc;
use uuid::Uuid;

// Schema of nats table
// id / nsc_account_id / creds_admin / creds_user / created_at

pub async fn verify_nsc_user_exists(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    // Check if user exists in the database
    let rows = postgres_client.query("SELECT * FROM nats WHERE id = $1", &[&user_id])
        .await?;
    Ok(rows.len() > 0)
}

pub async fn insert_nsc_user(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("INSERT INTO nats (id) VALUES ($1)", &[&user_id])
        .await?;
    Ok(result > 0)
}

pub async fn delete_nsc_user(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    let result = postgres_client.execute("DELETE FROM nats WHERE id = $1", &[&user_id])
        .await?;
    Ok(result > 0)
}

pub async fn get_creds_admin(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<String, tokio_postgres::Error>{
    let rows = postgres_client.query("SELECT creds_admin FROM nats WHERE id = $1", &[&user_id])
        .await?;

    let row = rows.get(0);
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