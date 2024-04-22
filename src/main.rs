use axum::{
    debug_handler, extract::{Extension, Path, Request, State}, http::StatusCode, response::IntoResponse, routing::{get, post}, Json, Router,
    body::Body,
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
use tower::ServiceBuilder;

use tower_http::auth::AddAuthorization;

use axum::middleware::from_fn;

#[derive(Deserialize)]
struct SendMessage {
    message: String,
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

async fn auth_middleware<Body>(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {

    let AppState {
        creds_base_path: _,
        operator_name: _,
        postgres_client,
        main_topic: _,
        nats_url: _
    } = state;

    let auth_header = request
        .headers()
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth_header) if auth_header == "Bearer toto" => {
            next.run(request).await
        }
        _ => StatusCode::UNAUTHORIZED.into_response(),
    }
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
    
    // let auth_layer = APIKeyAuthorizationLayer {
    //     postgres_client: Arc::clone(&postgres_client)
    // };

    // Set up the router
    let app_state = state.clone();
    let app = Router::new()
        .route("/send/:user_id", post(send_message))
        .layer(from_fn(move |path, request, next| {
            let state = app_state.clone();
            auth_middleware::<Body>(axum::extract::State(state), path, request, next)
        }))
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
