use axum::http::StatusCode;
use axum::middleware;
use axum::{response::IntoResponse, routing::get, Router};
use tower::ServiceBuilder;

use crate::mw::mw_auth;
use crate::server_config::ServerConfig;

pub fn health_routes(env_config: ServerConfig) -> Router {
    let unauth_routes = Router::new().route("/", get(health_check_handler));
    let auth_routes = Router::new()
        .route("/logged-in", get(auth_check_handler))
        .route_layer(ServiceBuilder::new().layer(middleware::from_fn_with_state(
            env_config,
            mw_auth::mw_require_auth,
        )));
    unauth_routes.merge(auth_routes)
}

async fn health_check_handler() -> impl IntoResponse {
    StatusCode::OK
}

async fn auth_check_handler() -> impl IntoResponse {
    StatusCode::OK
}

#[cfg(test)]
mod tests {
    use crate::app;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt; // for `oneshot` and `ready`

    #[tokio::test]
    async fn hello_world() {
        let app = app();

        // `Router` implements `tower::Service<Request<Body>>` so we can
        // call it like any tower service, no need to run an HTTP server.
        let response = app
            .await
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }
}
