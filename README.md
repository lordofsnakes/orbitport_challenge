# Orbitport — Celestial Integration Plugin take-home

**Time required: ~2 hours.** If you're struggling to find the time, or you need a bit
more, let us know and we'll move your call forward.

We are not looking for production polish or full coverage. We want to see how you read an
existing Rust interface, integrate against an unfamiliar API, and reason about the gaps.
Do use AI freely, that's how we work at SpaceComputer too. Lean on it to learn, to think, to
get unstuck.

---

## Background

Orbitport is SpaceComputer's gateway between Earth and orbital infrastructure. It exposes
one developer-facing API and routes requests to whichever ground-station provider can
service them. This is how we call orbital services such as onboard TEEs and others.

Each provider (e.g. Leaf Space, KSAT, AWS Ground Station, and others) has its own API
shape, auth model, and data formats. Orbitport hides those differences behind a plugin
layer, let's call it **Celestial Integration Plugins**: every provider implements the same Rust trait,
and the rest of the platform only ever sees Orbitport's canonical types.

> This starter is a deliberately simplified slice of that plugin boundary: one trait, one
> provider, in-process, against a local mock. The real platform is larger; we've cut it
> down to the part that matters for this exercise.

Your task is to write one such plugin for a new (fictional) provider, **"Helios GS"**,
whose API is provided as a mock server in this repo.

---

## Build this

Implement the `GroundStationProvider` trait (provided, in `src/provider.rs`) for Helios GS,
in **`src/plugins/helios.rs`**. This is the _only file_ you need to write. Concretely:

1. **`list_passes`** calls the Helios "windows" endpoint and returns Orbitport `Pass`
   values. Helios uses different field names than Orbitport, so part of the work is
   mapping its schema onto ours.
2. **`schedule_contact`** books a pass through the Helios "bookings" endpoint and returns
   a `Contact`.
3. **`fetch_payload`** downloads the data produced by a completed contact and returns a
   `Payload`. (For Helios this is a cTRNG-style entropy beacon, base64-encoded.)
4. **Error mapping**: Helios returns `401`, `404`, and `503` in the obvious situations.
   Map these onto the right `ProviderError` variants rather than letting raw transport
   errors leak.

Then write a short `SOLUTION.md` (see **Deliverables**).

The starter already wires your plugin into a minimal Axum gateway (`src/main.rs`, one
illustrative `GET /passes` route) and includes one integration-test skeleton. Get the test
green, and add at least one test of your own.

---

## The provided interface

You do not need to change these files; they define the contract you implement against:

- **`src/types.rs`** — Orbitport's canonical types (`PassQuery`, `Pass`, `Contact`,
  `ContactStatus`, `Payload`).
- **`src/provider.rs`** — `ProviderError` and the `GroundStationProvider` trait.

`src/plugins/helios.rs` ships with the struct, constructors (`new` / `from_env`), and a
trait impl whose method bodies are `todo!()`. Fill them in.

---

## The Helios GS mock API (what you integrate against)

Base URL comes from `HELIOS_BASE_URL` (defaults to `http://127.0.0.1:8081`). Auth is
`Authorization: Bearer <HELIOS_TOKEN>` (the mock's token is **`helios-dev-token`**, which
is also the `from_env` default). A wrong or missing token returns `401`.

Note the deliberate mismatches with Orbitport's canonical types — handling them is the
point. Helios uses its own field names and its own `state` strings, and its payloads come
back base64-encoded.

### `GET /v1/windows?sat={id}&start={iso8601}&end={iso8601}`

```json
{
  "windows": [
    {
      "ref": "w_1a2b",
      "spacecraft": "SAT-42",
      "site": "svalbard-01",
      "window_start": "2026-08-12T14:03:00Z",
      "window_end":   "2026-08-12T14:11:30Z",
      "peak_elevation_deg": 61.5
    }
  ]
}
```

The mock returns several such windows (only one shown here); any of their `ref`s can be
booked. `window_start` / `window_end` map to `aos` / `los`. `ref` is the pass id.
`spacecraft` is the satellite id. `site` is the ground station. `peak_elevation_deg` is in
degrees, same unit as the canonical `max_elevation_deg`.

### `POST /v1/bookings` with body `{ "window_ref": "w_1a2b" }`

```json
{ "booking_id": "bk_77", "window_ref": "w_1a2b", "state": "RESERVED" }
```

Helios `state` values: `RESERVED`, `ACTIVE`, `DONE`, `ERROR`. Map these onto
`ContactStatus`. A `window_ref` that does not exist returns `404`.

### `GET /v1/bookings/{booking_id}/download`

```json
{ "mime": "application/octet-stream", "b64": "Vu8k...==" }
```

`b64` is base64-encoded bytes. A booking that is not yet `DONE` returns `503` with
`{ "error": "payload not ready" }`. An unknown booking returns `404`.

### Documented fixtures

The mock is deterministic:

| Fixture | Value | Use |
| --- | --- | --- |
| Token | `helios-dev-token` | anything else → `401` |
| Windows | `w_1a2b`, `w_2c3d`, `w_3e4f`, `w_5a6b` | the set `/v1/windows` returns; any can be booked |
| Completed booking | `bk_done` | pre-seeded in `DONE`; use it to exercise `fetch_payload` |
| Fresh bookings | `bk_77`, … | created by `POST /v1/bookings`, stay `RESERVED` → `503` on download |

> Heads-up: the trait has no "poll booking status" method, and freshly created bookings
> stay `RESERVED`. To exercise `fetch_payload` happy-path, use the pre-seeded `bk_done`.
> (Why a real provider needs a status/confirmation step is a good thing to note in
> `SOLUTION.md`.)

---

## Running the starter

```sh
cargo run --bin mock   # starts the Helios mock on :8081
cargo test             # runs the integration test (start the mock first)
cargo run              # starts the Orbitport gateway with your plugin on :8080
```

Try the gateway once your plugin works:

```sh
curl 'http://127.0.0.1:8080/passes?satellite_id=SAT-42&from=2026-08-01T00:00:00Z&to=2026-09-01T00:00:00Z'
```

The provided integration test (`tests/integration.rs`) is `#[ignore]`d so a fresh clone is
green. Once your plugin is wired up, remove the `#[ignore]` and run
`cargo test -- --include-ignored`. We left the error-path and `fetch_payload` coverage for
**your** test.

---

## Deliverables

1. **Code**: `src/plugins/helios.rs` implementing the trait, the provided test passing,
   and at least one test you wrote (a happy path or an error case is fine).
2. **`SOLUTION.md`** (create it — roughly half a page, no longer):
   - One paragraph on how you structured the schema mapping and error handling, and any
     shortcut you took because of the time box.
   - **Integration reasoning**: Orbitport will next onboard a real ground-station
     provider. From what the Helios mock did (and didn't) make you handle, name two things
     a production provider would likely throw at you that this mock didn't — e.g.
     pagination, async booking confirmation, rate limits, auth refresh, time-zone or unit
     quirks. A few bullets is plenty.
   - **One extension idea**: propose one feature that would make Orbitport more useful to
     a developer consuming it (routing, retries across providers, observability, a
     data-fetching primitive — anything). Two or three sentences on what and why.

Submit as a git repo or a zip. Include any assumptions in `SOLUTION.md`.

---

## Constraints and notes

- Use whatever crates you like. The starter pulls in `tokio`, `axum`, `reqwest`, `serde`,
  `serde_json`, `async-trait`, `thiserror`, `chrono`, and `base64`.
- A clean, partial solution beats a sprawling, broken one. If you run low on time, get
  `list_passes` and `fetch_payload` solid and say so in `SOLUTION.md`.
- No real provider credentials are needed or wanted. Everything runs against the local mock.
- AI tools are allowed. We care about your judgment on the design questions, which is
  where they help least.
