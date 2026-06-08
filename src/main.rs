//! Orbitport gateway (illustrative).
//!
//! A deliberately minimal Axum server that wires the Helios plugin in behind the
//! canonical [`GroundStationProvider`] trait and exposes ONE representative route so you
//! can see your plugin serve real traffic:
//!
//!   GET /passes?satellite_id=SAT-42&from=<rfc3339>&to=<rfc3339>
//!
//! `ProviderError` is mapped to an HTTP status here — that mapping is the whole reason
//! the error enum exists, so providers can fail in canonical terms. You do not need to
//! change this file.
//!
//!   cargo run            # serves on http://127.0.0.1:8080
//!   cargo run --bin mock # in another terminal, the Helios mock on :8081

use std::sync::Arc;

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::Deserialize;

use orbitport_helios_challenge::plugins::helios::HeliosPlugin;
use orbitport_helios_challenge::provider::{GroundStationProvider, ProviderError};
use orbitport_helios_challenge::types::PassQuery;

type Provider = Arc<dyn GroundStationProvider>;

#[derive(Debug, Deserialize)]
struct PassesParams {
    satellite_id: String,
    ground_station: Option<String>,
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    let provider: Provider = Arc::new(HeliosPlugin::from_env());
    println!("Orbitport gateway using provider: {}", provider.name());

    let app = Router::new()
        .route("/passes", get(passes))
        .with_state(provider);

    let addr = "127.0.0.1:8080";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Orbitport gateway listening on http://{addr}");
    axum::serve(listener, app).await.unwrap();
}

async fn passes(State(provider): State<Provider>, Query(p): Query<PassesParams>) -> Response {
    let query = PassQuery {
        satellite_id: p.satellite_id,
        ground_station: p.ground_station,
        from: p.from,
        to: p.to,
    };
    match provider.list_passes(query).await {
        Ok(passes) => Json(passes).into_response(),
        Err(e) => provider_error_to_response(e),
    }
}

/// The canonical `ProviderError` -> HTTP mapping. Note that raw transport/provider
/// shapes never reach this point: the plugin has already normalized them.
fn provider_error_to_response(err: ProviderError) -> Response {
    let status = match err {
        ProviderError::Auth => StatusCode::UNAUTHORIZED,
        ProviderError::NotFound(_) => StatusCode::NOT_FOUND,
        ProviderError::Unavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
        ProviderError::Decode(_) => StatusCode::BAD_GATEWAY,
        ProviderError::Transport(_) => StatusCode::BAD_GATEWAY,
    };
    (
        status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}
