//! Orbitport's canonical types.
//!
//! These are the platform-wide vocabulary every Celestial Integration Plugin maps
//! onto. The rest of Orbitport only ever sees these — never a provider's wire shapes.
//!
//! You do not need to change this file.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PassQuery {
    pub satellite_id: String,
    pub ground_station: Option<String>,
    pub from: DateTime<Utc>,
    pub to: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pass {
    pub id: String,
    pub satellite_id: String,
    pub ground_station: String,
    pub aos: DateTime<Utc>,     // acquisition of signal
    pub los: DateTime<Utc>,     // loss of signal
    pub max_elevation_deg: f64, // degrees, 0..90
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Contact {
    pub id: String,
    pub pass_id: String,
    pub status: ContactStatus,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ContactStatus {
    Scheduled,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payload {
    pub contact_id: String,
    pub content_type: String,
    pub data: Vec<u8>,
}
