use axum::error_handling::HandleErrorLayer;
use axum::{http::StatusCode, BoxError, Router};
use axum_macros::debug_handler;
use server_config::ServerConfig;
use sqlx::{Pool, Sqlite};
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod api;
mod controllers;
mod error;
mod jwt_claims;
mod models;
mod mw;
mod server_config;

#[tokio::main]
async fn main() {
    let _ = dotenv::dotenv();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_testing=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::debug!("starting server");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    axum::serve(listener, app().await.into_make_service())
        .await
        .unwrap();
}

async fn app() -> Router {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
    let pool = controllers::sqlite::create_db_pool(database_url).await;
    let server_config = load_config(pool);

    let login_routes = api::routes_login::routes(server_config.clone());

    Router::new()
        .merge(login_routes)
        .nest(
            "/health",
            api::routes_health::health_routes(server_config.clone()),
        )
        .layer(CorsLayer::new().allow_origin(tower_http::cors::Any))
        .layer(TraceLayer::new_for_http())
        .layer(CookieManagerLayer::new())
}

fn load_config(db_pool: Pool<Sqlite>) -> ServerConfig {
    let jwt_secret_key = std::env::var("JWT_SECRET_KEY").expect("JWT_SECRET_KEY not set");
    ServerConfig {
        jwt_secret_key,
        db_pool,
    }
}
