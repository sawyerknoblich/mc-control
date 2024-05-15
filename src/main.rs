use axum::{http::StatusCode, response::IntoResponse, routing::post, Router};
use tokio::process::Command;

#[tokio::main]
async fn main() {
    let app = Router::new().route("/restart", post(restart_server));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
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
