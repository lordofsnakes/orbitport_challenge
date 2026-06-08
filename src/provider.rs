//! The contract every Celestial Integration Plugin implements.
//!
//! Orbitport routes a developer request to whichever provider can service it; each
//! provider is hidden behind this trait so the platform only deals in canonical types
//! (see [`crate::types`]) and in [`ProviderError`] — never raw transport errors or
//! provider-shaped JSON.
//!
//! You do not need to change this file.

use crate::types::*;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("authentication failed")]
    Auth,
    #[error("not found: {0}")]
    NotFound(String),
    #[error("provider unavailable: {0}")]
    Unavailable(String),
    #[error("could not decode provider response: {0}")]
    Decode(String),
    #[error(transparent)]
    Transport(#[from] reqwest::Error),
}

#[async_trait]
pub trait GroundStationProvider: Send + Sync {
    /// Stable identifier used in routing and logs, e.g. "helios".
    fn name(&self) -> &'static str;

    /// Upcoming contact windows matching the query.
    async fn list_passes(&self, query: PassQuery) -> Result<Vec<Pass>, ProviderError>;

    /// Book a specific pass. `pass_id` is the `Pass::id` returned above.
    async fn schedule_contact(&self, pass_id: &str) -> Result<Contact, ProviderError>;

    /// Fetch the data produced by a completed contact.
    async fn fetch_payload(&self, contact_id: &str) -> Result<Payload, ProviderError>;
}
