use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{extract::Json, routing::post, Router};
use serde::Deserialize;

pub fn routes() -> Router {
    Router::new()
        .route("/login", post(login_handler))
        .route("/sign-up", post(sign_up_handler))
    // .with_state(env_config)
}

async fn sign_up_handler() -> impl IntoResponse {
    StatusCode::OK.into_response()
}

async fn login_handler(
    // cookies: Cookies,
    // State(config): State<ServerConfig>,
    Json(creds): Json<LoginPayload>,
) -> impl IntoResponse {
    dbg!(&creds);

    StatusCode::OK.into_response()
}

#[derive(Debug, Deserialize, Clone)]
struct LoginPayload {
    username: String,
    password: String,
}
