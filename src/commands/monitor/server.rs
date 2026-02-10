use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::Router;
use axum::extract::State;
use axum::http::{StatusCode, header};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use rust_embed::Embed;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

use super::state::DashboardState;

#[derive(Embed)]
#[folder = "assets/"]
struct Assets;

pub fn create_router(state: Arc<DashboardState>) -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/assets/{*path}", get(asset_handler))
        .route("/api/events", get(sse_handler))
        .with_state(state)
}

async fn index_handler() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(content) => {
            Html(String::from_utf8_lossy(content.data.as_ref()).to_string()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

async fn asset_handler(axum::extract::Path(path): axum::extract::Path<String>) -> Response {
    match Assets::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path)
                .first_or_octet_stream()
                .to_string();
            ([(header::CONTENT_TYPE, mime)], content.data.to_vec()).into_response()
        }
        None => (StatusCode::NOT_FOUND, "not found").into_response(),
    }
}

async fn sse_handler(
    State(state): State<Arc<DashboardState>>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    // Send initial snapshot first, then stream live events
    let snapshot = state.snapshot().await;
    let rx = state.subscribe();

    let snapshot_stream = tokio_stream::once(Ok(Event::default()
        .event("message")
        .data(serde_json::to_string(&snapshot).unwrap_or_default())));

    let live_stream = BroadcastStream::new(rx).filter_map(|result| {
        match result {
            Ok(event) => {
                let data = serde_json::to_string(&event).unwrap_or_default();
                Some(Ok(Event::default().event("message").data(data)))
            }
            Err(_) => None, // Lagged — skip
        }
    });

    let combined = snapshot_stream.chain(live_stream);

    Sse::new(combined).keep_alive(
        KeepAlive::new()
            .interval(Duration::from_secs(15))
            .text("heartbeat"),
    )
}
