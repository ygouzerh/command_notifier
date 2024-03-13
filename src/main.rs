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
    if account_id == "ACO65G5Q3KWQPDT3DRUDE756FCQH36NQ67JZ6NTOOALJHYBXZUSFNUCI" {
        return "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJVM1AzU0pTVUhQSjZSQUxGVDRRWDNIU1BRSFc0WEhITkNDQ09IMlFUTkE3T1ZBU0VRNkRRIiwiaWF0IjoxNzA5NjI5MDQ0LCJpc3MiOiJPQUVKUElMQzVNWEdMVzZVVVNOVk1WRTI2VTdXR01KT1hRQ1BINFI1VTNERUE2VVpJRDZONFhGRyIsIm5hbWUiOiJ5b2hhbjAxIiwic3ViIjoiQUNPNjVHNVEzS1dRUERUM0RSVURFNzU2RkNRSDM2TlE2N0paNk5UT09BTEpIWUJYWlVTRk5VQ0kiLCJuYXRzIjp7ImxpbWl0cyI6eyJzdWJzIjotMSwiZGF0YSI6LTEsInBheWxvYWQiOi0xLCJpbXBvcnRzIjotMSwiZXhwb3J0cyI6LTEsIndpbGRjYXJkcyI6dHJ1ZSwiY29ubiI6LTEsImxlYWYiOi0xfSwiZGVmYXVsdF9wZXJtaXNzaW9ucyI6eyJwdWIiOnt9LCJzdWIiOnt9fSwiYXV0aG9yaXphdGlvbiI6eyJhdXRoX3VzZXJzIjpudWxsfSwidHlwZSI6ImFjY291bnQiLCJ2ZXJzaW9uIjoyfX0.L98wxq0CMCQyA2L1mrBvTCKawroYLhHKWklrHIlrF61KPFdQuHd_MKyAYiKfyVS6xBzdguYC7aX8MXq4FP_sDQ".to_string()
    }
    if account_id == "ACWPGFSDD6EJ5CGMOFO62AC3Q7CMLKUVRXVJDANMIJIKVTFXF27ACHVS" {
        return "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJQWFQzQlozQ1ZLNVNDNkRHQkZTM0NHUEJSQTNaU0ZIUzNCT0NaVDVUSE5WTUxOWkNVVFNRIiwiaWF0IjoxNzA5NjI4ODQ3LCJpc3MiOiJPQUVKUElMQzVNWEdMVzZVVVNOVk1WRTI2VTdXR01KT1hRQ1BINFI1VTNERUE2VVpJRDZONFhGRyIsIm5hbWUiOiJTWVNfQUNDT1VOVCIsInN1YiI6IkFDV1BHRlNERDZFSjVDR01PRk82MkFDM1E3Q01MS1VWUlhWSkRBTk1JSklLVlRGWEYyN0FDSFZTIiwibmF0cyI6eyJsaW1pdHMiOnsic3VicyI6LTEsImRhdGEiOi0xLCJwYXlsb2FkIjotMSwiaW1wb3J0cyI6LTEsImV4cG9ydHMiOi0xLCJ3aWxkY2FyZHMiOnRydWUsImNvbm4iOi0xLCJsZWFmIjotMX0sImRlZmF1bHRfcGVybWlzc2lvbnMiOnsicHViIjp7fSwic3ViIjp7fX0sImF1dGhvcml6YXRpb24iOnsiYXV0aF91c2VycyI6bnVsbH0sInR5cGUiOiJhY2NvdW50IiwidmVyc2lvbiI6Mn19.UpnAmSoD76TXcvW7RoFUTC-XdFispVaFXbDBbXpjkGIKF4nPG6YftDhd5_UOCo6CBzl1JWZVxrYJQDOJi_zRBQ".to_string()
    }

    if account_id == "AABQNX6YAEIMQWRPPENS5AGZ4RLSY5JPB7QESAZSTBZSU5VY3KKXPTBZ" {
        return "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJTU1ZVQUdLRkNaV0JWN1U0NEVRQ1dETzZMM1oyRENHMzdLNDdXS0JVQzY3RkFOVUIyWUtRIiwiaWF0IjoxNzEwMTU0NTQ4LCJpc3MiOiJPQUVKUElMQzVNWEdMVzZVVVNOVk1WRTI2VTdXR01KT1hRQ1BINFI1VTNERUE2VVpJRDZONFhGRyIsIm5hbWUiOiI3YzI3OGVjYy1kNjI0LTQ1YTAtYWE4Ny05YWRkNzI1M2I1MTciLCJzdWIiOiJBQUJRTlg2WUFFSU1RV1JQUEVOUzVBR1o0UkxTWTVKUEI3UUVTQVpTVEJaU1U1VlkzS0tYUFRCWiIsIm5hdHMiOnsibGltaXRzIjp7InN1YnMiOi0xLCJkYXRhIjotMSwicGF5bG9hZCI6LTEsImltcG9ydHMiOi0xLCJleHBvcnRzIjotMSwid2lsZGNhcmRzIjp0cnVlLCJjb25uIjotMSwibGVhZiI6LTF9LCJkZWZhdWx0X3Blcm1pc3Npb25zIjp7InB1YiI6e30sInN1YiI6e319LCJhdXRob3JpemF0aW9uIjp7ImF1dGhfdXNlcnMiOm51bGx9LCJ0eXBlIjoiYWNjb3VudCIsInZlcnNpb24iOjJ9fQ.WwbEn-VkRviR88goLodXQjdN3ejfd9jG5TkGKF5jpFRZvLaBCH1xb7k8goERBjP_XZOyMra44pSa0njaxDMOAg".to_string()
    }
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
    // let nats_client = nats::Options::with_credentials("/Users/yohangouzerh/w/random-tests/nats/userYohan01.creds").connect("localhost:4222").unwrap();
    // let nats_client = Arc::new(nats_client);
    // let (postgres_client, connection) =
    // tokio_postgres::connect(&database_connection_string, NoTls)
    //     .await
    //     .unwrap();

    // tokio::spawn(async move {
    //     if let Err(e) = connection.await {
    //         eprintln!("Connection error: {}", e);
    //     }
    // });
    
    // let postgres_client = Arc::new(postgres_client);

    // Set up the router
    let app = Router::new()
        .route("/jwt/v1/accounts/:account_id", get(account_details)) // Dynamic segment
        .route("/jwt/v1/accounts/", get(accounts_base)); // Base path
        // .route("/send/:account_id", post(send_message))
        // .layer(Extension(nats_client.clone()))
        // .route("/account/create", post(create_nsc_account_handler))
        // .layer(Extension(postgres_client.clone()));
    
    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 9090));
    println!("Listening on {}", addr);

    // Start the server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
