//! Helios GS — Celestial Integration Plugin.  ← THIS FILE IS YOURS TO WRITE.
//!
//! Implement [`GroundStationProvider`] for the (fictional) Helios ground-station
//! provider by integrating against the local Helios mock API. See `README.md` for the
//! task, the Helios wire shapes, and the documented fixtures. The mock runs with:
//!
//!   cargo run --bin mock        # http://127.0.0.1:8081, token: helios-dev-token
//!
//! What to do (full detail in README.md):
//!   1. `list_passes` — GET /v1/windows, map each window onto a canonical `Pass`.
//!   2. `schedule_contact` — POST /v1/bookings, return a `Contact` (map the `state`).
//!   3. `fetch_payload` — GET /v1/bookings/{id}/download, base64-decode into `Payload`.
//!   4. Error mapping — 401 -> Auth, 404 -> NotFound, 503 -> Unavailable, decode failures
//!      -> Decode. Don't leak raw transport/provider shapes past this boundary.
//!
//! Keep Helios's wire shapes private to this module — the rest of the platform only ever
//! sees the canonical types in `crate::types`.

use async_trait::async_trait;

use crate::provider::{GroundStationProvider, ProviderError};
use crate::types::*;

pub struct HeliosPlugin {
    #[allow(dead_code)]
    base_url: String,
    #[allow(dead_code)]
    token: String,
    #[allow(dead_code)]
    http: reqwest::Client,
}

#[derive(serde::Deserialize)]
struct WindowsResponse{
     windows: Vec<HeliosWindow>
}
#[derive(serde::Deserialize)]
struct HeliosWindow {
    #[serde(rename = "ref")]
    reference: String,
    spacecraft: String,
    site: String,
    window_start: chrono::DateTime<chrono::Utc>,
    window_end: chrono::DateTime<chrono::Utc>,
    peak_elevation_deg: f64,
}

impl HeliosPlugin {
    pub fn new(base_url: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: token.into(),
            http: reqwest::Client::new(),
        }
    }

    /// Build from `HELIOS_BASE_URL` / `HELIOS_TOKEN`, with dev defaults that match the
    /// local mock (`http://127.0.0.1:8081`, `helios-dev-token`).
    pub fn from_env() -> Self {
        let base_url = std::env::var("HELIOS_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8081".to_string());
        let token =
            std::env::var("HELIOS_TOKEN").unwrap_or_else(|_| "helios-dev-token".to_string());
        Self::new(base_url, token)
    }
}

#[async_trait]
impl GroundStationProvider for HeliosPlugin {
    fn name(&self) -> &'static str {
        "helios"
    }

    async fn list_passes(&self, _query: PassQuery) -> Result<Vec<Pass>, ProviderError> {
        
        
        
        let response = self.http
        .get(format!("{}/v1/windows", self.base_url))
        .bearer_auth(&self.token)
        .query(&[
            ("sat", _query.satellite_id.as_str()),
            ("start", _query.from.to_rfc3339().as_str()),
            ("end", _query.to.to_rfc3339().as_str()),
        ]).send().await?;
        
        let status = response.status();
        match status{
           reqwest::StatusCode::OK =>{
            
            let body: WindowsResponse = response
            .json()
            .await
            .map_err(|e| ProviderError::Decode(e.to_string()))?;
            
            
            let mut passes = Vec::new();
            for window in body.windows{
                passes.push(Pass{
                    id: window.reference,
                    satellite_id: window.spacecraft,
                    ground_station: window.site,
                    aos: window.window_start,
                    los: window.window_end,
                    max_elevation_deg: window.peak_elevation_deg
                });

            }
            return Ok(passes)
           }
           reqwest::StatusCode::UNAUTHORIZED =>{
            return Err(ProviderError::Auth);
           }
           reqwest::StatusCode::NOT_FOUND =>{
            return Err(ProviderError::NotFound("windows not found".to_string()));
           }
           reqwest::StatusCode::SERVICE_UNAVAILABLE =>{
            return Err(ProviderError::Unavailable("helios unavailable".to_string()));
           }
           other => {
           return Err(ProviderError::Unavailable(format!("unexpected status {other}")));
           }
  
        }
  
        //todo!("call GET /v1/windows and map each Helios window onto a canonical Pass")
    }

    async fn schedule_contact(&self, _pass_id: &str) -> Result<Contact, ProviderError> {
        


        todo!("POST /v1/bookings and return a Contact, mapping the Helios `state`")
    }

    async fn fetch_payload(&self, _contact_id: &str) -> Result<Payload, ProviderError> {
        todo!("GET /v1/bookings/{{id}}/download and base64-decode into a Payload")
    }
}
