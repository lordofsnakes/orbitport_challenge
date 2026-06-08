//! Orbitport — Helios GS take-home.
//!
//! A deliberately simplified, single-service slice in the spirit of Orbitport's plugin
//! boundary: every ground-station provider implements [`provider::GroundStationProvider`]
//! and the rest of the platform only ever sees the canonical [`types`].
//!
//! Your work lives in [`plugins::helios`].

pub mod plugins;
pub mod provider;
pub mod types;
