//! Helios GS — mock provider API.
//!
//! A fully offline stand-in for the (fictional) Helios ground-station provider, so the
//! take-home runs with no credentials and no network. You should not need to edit this,
//! but you are welcome to read it to understand the wire shapes you integrate against.
//!
//!   cargo run --bin mock     # serves on http://127.0.0.1:8081
//!
//! Auth: every route requires `Authorization: Bearer helios-dev-token`.
//!
//! Endpoints:
//!   GET  /v1/windows?sat=&start=&end=     -> { "windows": [ ... ] }
//!   POST /v1/bookings  { "window_ref" }   -> { "booking_id", "window_ref", "state" }
//!   GET  /v1/bookings/{id}/download       -> { "mime", "b64" }
//!
//! Deterministic fixtures (documented in README.md):
//!   * /v1/windows returns a fixed set of windows (incl. `w_1a2b`); any can be booked
//!   * booking `bk_done` is pre-seeded in state DONE and serves a payload
//!   * freshly created bookings stay RESERVED, so /download returns 503 "not ready"
//!   * unknown window_ref / booking id returns 404

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use axum::{
    extract::{Path, State},
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::Engine;
use serde_json::{json, Value};

const TOKEN: &str = "helios-dev-token";
const KNOWN_WINDOW: &str = "w_1a2b";

/// Bytes the pre-seeded `bk_done` booking serves (a cTRNG-style entropy beacon).
const BEACON_BYTES: &[u8] = b"ORBITPORT-HELIOS-ENTROPY-BEACON!";

/// One advertised contact window.
struct WindowFixture {
    reference: &'static str,
    spacecraft: &'static str,
    site: &'static str,
    start: &'static str,
    end: &'static str,
    peak_elevation_deg: f64,
}

/// The fixed set of windows the mock advertises: `w_1a2b` (the one the README and test
/// reference by name) plus a few more so candidates have several passes to book. Varied
/// satellites, sites, and elevations on purpose. Any `ref` here can be booked.
const WINDOWS: &[WindowFixture] = &[
    WindowFixture {
        reference: KNOWN_WINDOW,
        spacecraft: "SAT-42",
        site: "svalbard-01",
        start: "2026-08-12T14:03:00Z",
        end: "2026-08-12T14:11:30Z",
        peak_elevation_deg: 61.5,
    },
    WindowFixture {
        reference: "w_2c3d",
        spacecraft: "SAT-42",
        site: "svalbard-01",
        start: "2026-08-12T15:47:10Z",
        end: "2026-08-12T15:55:40Z",
        peak_elevation_deg: 34.2,
    },
    WindowFixture {
        reference: "w_3e4f",
        spacecraft: "SAT-42",
        site: "troll-02",
        start: "2026-08-13T03:21:00Z",
        end: "2026-08-13T03:29:05Z",
        peak_elevation_deg: 78.9,
    },
    WindowFixture {
        reference: "w_5a6b",
        spacecraft: "SAT-7",
        site: "awarua-01",
        start: "2026-08-13T09:12:30Z",
        end: "2026-08-13T09:19:50Z",
        peak_elevation_deg: 12.6,
    },
];

fn window_exists(reference: &str) -> bool {
    WINDOWS.iter().any(|w| w.reference == reference)
}

#[derive(Clone)]
struct Booking {
    window_ref: String,
    state: &'static str, // RESERVED | ACTIVE | DONE | ERROR
}

#[derive(Clone)]
struct AppState {
    bookings: Arc<Mutex<HashMap<String, Booking>>>,
    next_id: Arc<Mutex<u64>>,
}

#[tokio::main]
async fn main() {
    // Seed one already-DONE booking (`bk_done`) at boot. Fresh bookings start RESERVED and
    // never advance, so this pre-completed one is how `fetch_payload` can be tested.
    let mut seed = HashMap::new();
    seed.insert(
        "bk_done".to_string(),
        Booking {
            window_ref: KNOWN_WINDOW.to_string(),
            state: "DONE",
        },
    );

    let state = AppState {
        bookings: Arc::new(Mutex::new(seed)),
        next_id: Arc::new(Mutex::new(77)),
    };

    let app = Router::new()
        .route("/v1/windows", get(list_windows))
        .route("/v1/bookings", post(create_booking))
        .route("/v1/bookings/:id/download", get(download))
        .with_state(state);

    let addr = "127.0.0.1:8081";
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("Helios GS mock listening on http://{addr}");
    println!("  token: {TOKEN}   window: {KNOWN_WINDOW}   completed booking: bk_done");
    axum::serve(listener, app).await.unwrap();
}

/// Returns `Some(401 response)` unless the request carries
/// `Authorization: Bearer helios-dev-token`.
fn auth_failure(headers: &HeaderMap) -> Option<Response> {
    let ok = headers
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .map(|v| v == format!("Bearer {TOKEN}"))
        .unwrap_or(false);
    if ok {
        None
    } else {
        Some(error(StatusCode::UNAUTHORIZED, "invalid or missing token"))
    }
}

fn error(status: StatusCode, msg: &str) -> Response {
    (status, Json(json!({ "error": msg }))).into_response()
}

async fn list_windows(headers: HeaderMap) -> Response {
    if let Some(resp) = auth_failure(&headers) {
        return resp;
    }
    // The mock ignores the sat/start/end query params and returns the full fixed set.
    let windows: Vec<Value> = WINDOWS
        .iter()
        .map(|w| {
            json!({
                "ref": w.reference,
                "spacecraft": w.spacecraft,
                "site": w.site,
                "window_start": w.start,
                "window_end": w.end,
                "peak_elevation_deg": w.peak_elevation_deg,
            })
        })
        .collect();
    Json(json!({ "windows": windows })).into_response()
}

async fn create_booking(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<Value>,
) -> Response {
    if let Some(resp) = auth_failure(&headers) {
        return resp;
    }
    let window_ref = body
        .get("window_ref")
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if !window_exists(window_ref) {
        return error(StatusCode::NOT_FOUND, "unknown window_ref");
    }

    let booking_id = {
        let mut n = state.next_id.lock().unwrap();
        let id = format!("bk_{n}");
        *n += 1;
        id
    };
    state.bookings.lock().unwrap().insert(
        booking_id.clone(),
        Booking {
            window_ref: window_ref.to_string(),
            state: "RESERVED",
        },
    );

    Json(json!({
        "booking_id": booking_id,
        "window_ref": window_ref,
        "state": "RESERVED"
    }))
    .into_response()
}

async fn download(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Response {
    if let Some(resp) = auth_failure(&headers) {
        return resp;
    }
    let booking = state.bookings.lock().unwrap().get(&id).cloned();
    match booking {
        None => error(StatusCode::NOT_FOUND, "unknown booking"),
        Some(b) if b.state == "DONE" => {
            let _ = &b.window_ref; // present for realism; unused by the mock response
            let b64 = base64::engine::general_purpose::STANDARD.encode(BEACON_BYTES);
            Json(json!({ "mime": "application/octet-stream", "b64": b64 })).into_response()
        }
        Some(_) => error(StatusCode::SERVICE_UNAVAILABLE, "payload not ready"),
    }
}
