use std::time::Duration;

use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use tokio::{process::Command, signal};
use tower_http::{services::ServeDir, timeout::TimeoutLayer, trace::TraceLayer};

#[derive(Clone)]
struct AppState {
    password: String,
    password_hint: String,
}

#[tokio::main]
async fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    tracing_subscriber::fmt::init();

    let app_state = read_app_state()?;

    let app = Router::new()
        .route("/api/health", get(health_check))
        .route("/api/password_hint", get(password_hint))
        .route("/api/restart", post(restart_server))
        .nest_service("/", ServeDir::new("/app/ui"))
        .layer(TraceLayer::new_for_http())
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .with_state(app_state);

    let port = "3000";
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}")).await?;

    tracing::info!("Listening on port {port}");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

fn read_app_state() -> color_eyre::Result<AppState> {
    let password = std::env::var("MC_PASSWORD")?;
    let password_hint = std::env::var("MC_PASSWORD_HINT")?;

    Ok(AppState {
        password,
        password_hint,
    })
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

async fn password_hint(State(app_state): State<AppState>) -> impl IntoResponse {
    (StatusCode::OK, Json(app_state.password_hint))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RestartParams {
    password: String,
}

async fn restart_server(
    State(app_state): State<AppState>,
    Json(body): Json<RestartParams>,
) -> impl IntoResponse {
    if body.password.trim().to_lowercase() != app_state.password.trim().to_lowercase() {
        tracing::warn!("Invalid authorization detected");
        return (StatusCode::UNAUTHORIZED, "Invalid password".to_owned());
    }

    tracing::info!("Executing rollout restart");

    let res = Command::new("kubectl")
        .args(["rollout", "restart", "deployment/minecraft-server"])
        .output()
        .await;

    match res {
        Ok(output) => match output.status.success() {
            true => {
                tracing::info!("Rollout restart successful");
                (StatusCode::OK, "".to_owned())
            }
            false => {
                let stderr = String::from_utf8(output.stderr)
                    .unwrap_or("Unknown error, unable to parse stderr".to_owned());
                tracing::error!(
                    "Command returned status code {} with stderr: {}",
                    output.status,
                    stderr
                );
                (StatusCode::INTERNAL_SERVER_ERROR, stderr)
            }
        },
        Err(e) => {
            let err_string = e.to_string();
            tracing::error!("Error spawning restart command: {}", err_string);
            (StatusCode::INTERNAL_SERVER_ERROR, err_string)
        }
    }
}
