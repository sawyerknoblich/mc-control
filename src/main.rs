use std::time::Duration;

use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use tokio::{process::Command, signal};
use tower_http::{timeout::TimeoutLayer, trace::TraceLayer};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/restart", post(restart_server))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(5)));

    let port = "3000";
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    tracing::info!("Listening on port {port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

async fn health_check() -> impl IntoResponse {
    let res = Command::new("kubectl")
        .args(["version", "--client"])
        .output()
        .await;

    match res {
        Ok(output) => match output.status.success() {
            true => (
                StatusCode::OK,
                String::from_utf8(output.stdout).unwrap_or("Unable to parse stderr".to_owned()),
            ),
            false => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from_utf8(output.stderr)
                    .unwrap_or("Unknown error, unable to parse stderr".to_owned()),
            ),
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

async fn restart_server() -> impl IntoResponse {
    let res = Command::new("kubectl")
        .args(["rollout", "restart", "deployment/minecraft-server"])
        .output()
        .await;

    match res {
        Ok(output) => match output.status.success() {
            true => (StatusCode::OK, "".to_owned()),
            false => (
                StatusCode::INTERNAL_SERVER_ERROR,
                String::from_utf8(output.stderr)
                    .unwrap_or("Unknown error, unable to parse stderr".to_owned()),
            ),
        },
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}
