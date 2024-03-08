use axum::{
    Json,
    routing::get,
    routing::post,
    Router,
    extract::{Extension, Path},
    http::StatusCode,
    response::IntoResponse,
};
use std::{env, net::SocketAddr};
use serde::Deserialize;
use std::sync::Arc;
use std::path;
use std::io::{self, Read};
use std::process::Command;
use std::fs;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

#[derive(Deserialize)]
struct SendMessage {
    message: String,
}

#[derive(Deserialize)]
struct CreateAccountMessage {
    user_id: String,
}

async fn account_details(Path(account_id): Path<String>) -> String {
    "Account not found".to_string()
}

async fn accounts_base() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

async fn send_message(
    Extension(nats_client): Extension<Arc<nats::Connection>>,
    Path(account_id): Path<String>,
    Json(payload): Json<SendMessage>
) -> impl IntoResponse {
    
    let _ = match nats_client.publish("topic01", &payload.message) {
        Ok(_) => (
            StatusCode::OK,
            format!("Sent to account {} : {}", account_id, &payload.message)
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error sending message to account {}: {}", account_id, e)
        )
    };
    
}

pub async fn verify_user_exists(postgres_client: Arc<tokio_postgres::Client>, user_id: Uuid) -> Result<bool, tokio_postgres::Error>{
    // Check if user exists in the database
    let rows = postgres_client.query("SELECT * FROM auth.users WHERE id = $1", &[&user_id])
        .await?;
    Ok(rows.len() > 0)
}

async fn create_nsc_account_handler(
    Extension(client): Extension<Arc<tokio_postgres::Client>>,
    Json(payload): Json<CreateAccountMessage>
) -> impl IntoResponse {
    let rows = client.query("SELECT * FROM nats", &[])
        .await
        .unwrap();
    println!("Got {:?} rows", rows);
    (StatusCode::OK, "OK")
}

#[tokio::main]
async fn main() {
    
    let database_connection_string = env::var("DATABASE_CONNECTION_STRING").expect("DATABASE_CONNECTION_STRING must be set");
    let nats_client = nats::Options::with_credentials("/Users/yohangouzerh/w/random-tests/nats/userYohan01.creds").connect("localhost:4222").unwrap();
    let nats_client = Arc::new(nats_client);
    let (postgres_client, connection) =
    tokio_postgres::connect(&database_connection_string, NoTls)
        .await
        .unwrap();

    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });
    
    let postgres_client = Arc::new(postgres_client);

    // Set up the router
    let app = Router::new()
        .route("/jwt/v1/accounts/:account_id", get(account_details)) // Dynamic segment
        .route("/jwt/v1/accounts/", get(accounts_base)) // Base path
        .route("/send/:account_id", post(send_message))
        .layer(Extension(nats_client.clone()))
        .route("/account/create", post(create_nsc_account_handler))
        .layer(Extension(postgres_client.clone()));
    
    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 9090));
    println!("Listening on {}", addr);

    // Start the server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
