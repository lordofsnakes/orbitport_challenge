//! Integration test against the Helios mock.
//!
//! These tests talk to the mock server, so start it first in another terminal:
//!
//!   cargo run --bin mock        # serves on http://127.0.0.1:8081
//!   cargo test                  # then run this
//!
//! The provided test is `#[ignore]`d so a fresh clone is green. Once your plugin is
//! wired up, remove the `#[ignore]` and run `cargo test -- --include-ignored`.
//!
//! Deliverable: add at least one test of your own — a good target is an error branch
//! (401/404/503) or `fetch_payload` against the pre-seeded `bk_done` booking.

use chrono::Utc;
use orbitport_helios_challenge::plugins::helios::HeliosPlugin;
use orbitport_helios_challenge::provider::GroundStationProvider;
use orbitport_helios_challenge::types::PassQuery;

const MOCK_URL: &str = "http://127.0.0.1:8081";
const TOKEN: &str = "helios-dev-token";

fn plugin() -> HeliosPlugin {
    HeliosPlugin::new(MOCK_URL, TOKEN)
}

fn query() -> PassQuery {
    PassQuery {
        satellite_id: "SAT-42".to_string(),
        ground_station: None,
        from: Utc::now(),
        to: Utc::now(),
    }
}

#[tokio::test]
#[ignore = "requires the mock (cargo run --bin mock); remove once your plugin is wired up"]
async fn list_passes_maps_the_window() {
    let passes = plugin().list_passes(query()).await.unwrap_or_else(|e| {
        panic!(
            "list_passes failed ({e}). Is the mock running on {MOCK_URL}? (cargo run --bin mock)"
        )
    });

    assert!(!passes.is_empty(), "mock advertises several windows");
    // Helios `ref` becomes the canonical `Pass::id`.
    assert!(
        passes.iter().any(|p| p.id == "w_1a2b"),
        "the known window w_1a2b should be present"
    );
}
