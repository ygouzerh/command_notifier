use axum::{
    Json,
    routing::get,
    routing::post,
    Router,
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    debug_handler
};

use command_notifier::{accounts_lifecycle::get_admin_creds_if_not_exists, postgres::{self, setup_postgres_client, verify_nsc_user_exists}};
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

    if account_id == "ABVGXFJMD4UNCZ3HYFBHTTFFMMPW3GKGIAW2JCJOPDEZEWRHXXRBRJMH" {
        return "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJEQ0xDS1VCUjNIT1QyTVQ3VkRLQlFHM1hPVDVNVlNHR0hEN1pFWFhNVkU1VzNONFNKUVVRIiwiaWF0IjoxNzExNDQ2NDkzLCJpc3MiOiJPQUVKUElMQzVNWEdMVzZVVVNOVk1WRTI2VTdXR01KT1hRQ1BINFI1VTNERUE2VVpJRDZONFhGRyIsIm5hbWUiOiJ5b2hhbjAyIiwic3ViIjoiQUJWR1hGSk1ENFVOQ1ozSFlGQkhUVEZGTU1QVzNHS0dJQVcySkNKT1BERVpFV1JIWFhSQlJKTUgiLCJuYXRzIjp7ImxpbWl0cyI6eyJzdWJzIjotMSwiZGF0YSI6LTEsInBheWxvYWQiOi0xLCJpbXBvcnRzIjotMSwiZXhwb3J0cyI6LTEsIndpbGRjYXJkcyI6dHJ1ZSwiY29ubiI6LTEsImxlYWYiOi0xfSwiZGVmYXVsdF9wZXJtaXNzaW9ucyI6eyJwdWIiOnt9LCJzdWIiOnt9fSwiYXV0aG9yaXphdGlvbiI6eyJhdXRoX3VzZXJzIjpudWxsfSwidHlwZSI6ImFjY291bnQiLCJ2ZXJzaW9uIjoyfX0.jTDdqJx7bTEvUrvGpGsBaegzSo0CMLf2PsJGjPxOXysUL9TUwiIIX4EjCNQAna7pv-EmtqH9FEsmEkyjv9JnBg".to_string()
    }

    if account_id == "ABOX6UJ4REBHGMEGKOVKSJC3YR2N33TIILK5VJVP4E4E24UNFDQ2XZR6" {
        return "eyJ0eXAiOiJKV1QiLCJhbGciOiJlZDI1NTE5LW5rZXkifQ.eyJqdGkiOiJLRFM0N1hJTjJLTk9aTFNEQ0FYQ1laREVXSUQ3SzRXRkNWUFJPQkFVVEEzRTJKWU5KWTdRIiwiaWF0IjoxNzEyNjU2MzQ5LCJpc3MiOiJPQUVKUElMQzVNWEdMVzZVVVNOVk1WRTI2VTdXR01KT1hRQ1BINFI1VTNERUE2VVpJRDZONFhGRyIsIm5hbWUiOiI3YzI3OGVjYy1kNjI0LTQ1YTAtYWE4Ny05YWRkNzI1M2I1MTciLCJzdWIiOiJBQk9YNlVKNFJFQkhHTUVHS09WS1NKQzNZUjJOMzNUSUlMSzVWSlZQNEU0RTI0VU5GRFEyWFpSNiIsIm5hdHMiOnsibGltaXRzIjp7InN1YnMiOi0xLCJkYXRhIjotMSwicGF5bG9hZCI6LTEsImltcG9ydHMiOi0xLCJleHBvcnRzIjotMSwid2lsZGNhcmRzIjp0cnVlLCJjb25uIjotMSwibGVhZiI6LTF9LCJkZWZhdWx0X3Blcm1pc3Npb25zIjp7InB1YiI6e30sInN1YiI6e319LCJhdXRob3JpemF0aW9uIjp7ImF1dGhfdXNlcnMiOm51bGx9LCJ0eXBlIjoiYWNjb3VudCIsInZlcnNpb24iOjJ9fQ.CLDG89mwIoFZfMuEqgtuVEyMb1bREgWWpctaBAQOCg60MnutfLpc4IC8e7iEzY-fLdTr7BAFrQFcL2SyVJzqDQ".to_string()
    }

    "Account not found".to_string()
}

async fn accounts_base() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}

#[derive(Clone)]
struct AppState {
    creds_base_path: String,
    operator_name: String,
    postgres_client: Arc<tokio_postgres::Client>,
    main_topic: String,
    nats_url: String
}

#[debug_handler]
async fn send_message(
    Path(user_id): Path<String>, 
    State(state): State<AppState>,
    Json(payload): Json<SendMessage>
) -> impl IntoResponse {

    // Verify if user exists in the database

    let AppState {
        creds_base_path,
        operator_name,
        postgres_client,
        main_topic,
        nats_url
    } = state;

    let user_uuid = Uuid::parse_str(&user_id);

    if user_uuid.is_err() {
        return (StatusCode::BAD_REQUEST, "Invalid user id, it should be an uuid").into_response();
    }
    let user_uuid = user_uuid.unwrap();
    let account_name = &user_id;
    let user_name = &user_id;
    let user_exists = verify_nsc_user_exists(Arc::clone(&postgres_client), user_uuid).await.unwrap();
    if !user_exists {
        return (StatusCode::NOT_FOUND, "User not found").into_response();
    }
    let creds_admin_path = get_admin_creds_if_not_exists(&creds_base_path, &operator_name, &account_name, &user_name).await;
    if let Err(e) = creds_admin_path {
        // Log the error using a logging library or custom logging mechanism
        println!("Failed to get the admin credentials of the user: {:?}", e);
        
        // Return an internal server error response
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to get the admin credentials of the user, contact administrator").into_response();
    }
    let creds_admin_path = creds_admin_path.unwrap();
    let nats_client = nats::Options::with_credentials(creds_admin_path).connect(nats_url).unwrap();

    let _ = match nats_client.publish(&main_topic, &payload.message) {
        Ok(_) => (
            StatusCode::OK,
            format!("Sent to account {} : {}", account_name, &payload.message)
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error sending message to account {}: {}", account_name, e)
        )
    };
    
    (StatusCode::OK, "Sent to user").into_response()
}

#[tokio::main]
async fn main() {
    
    let creds_base_path = env::var("CREDS_BASE_PATH").expect("CREDS_BASE_PATH must be set");
    let operator_name = env::var("TEST_OPERATOR_NAME").expect("TEST_OPERATOR_NAME must be set");
    
    let postgres_client = setup_postgres_client().await;
    let postgres_client = Arc::new(postgres_client);

    let state = AppState {
        creds_base_path: creds_base_path,
        operator_name: operator_name,
        postgres_client: Arc::clone(&postgres_client),
        main_topic: "topic01".to_string(),
        nats_url: "localhost:4222".to_string()
    };
    
    // Set up the router
    let app = Router::new()
        // .route("/jwt/v1/accounts/:account_id", get(account_details)) // Dynamic segment
        // .route("/jwt/v1/accounts/", get(accounts_base)) // Base path
        .route("/send/:user_id", post(send_message))
        .with_state(state);
    
    // Define the server address
    let addr = SocketAddr::from(([127, 0, 0, 1], 9090));
    println!("Listening on {}", addr);

    // Start the server
    axum_server::bind(addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
