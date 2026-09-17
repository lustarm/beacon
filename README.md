# Beacon

A minimal HTTP uptime monitor written in Rust that polls a list of URLs on an interval and reports their status to stdout.

Beacon reads a TOML file describing the endpoints you care about, then loops forever issuing `GET` requests against each one and logging whether it answered `200 OK`. It exists as a small, dependency-light alternative to a full monitoring stack when all you need is a process that tells you an endpoint is alive.

> **Status:** early development (`v0.1.0`). The core polling loop works; error handling and configuration wiring are incomplete. See [Known limitations](#known-limitations) before relying on it.

## Features

- Declarative TOML configuration — targets are data, not code
- Named targets, so log output identifies *which* service responded
- Per-target interval and timeout overrides with global defaults
- Async HTTP via `reqwest` on the Tokio runtime
- Single self-contained binary with no runtime dependencies
- Structured, greppable log prefixes: `[config]`, `[info]`, `[status]`, `[error]`

## Tech Stack

**Language**

- Rust (edition 2024 — requires Rust 1.85 or newer)

**Libraries**

| Crate | Version | Role |
| --- | --- | --- |
| [`reqwest`](https://crates.io/crates/reqwest) | 0.13.5 | HTTP client used to probe each target |
| [`tokio`](https://crates.io/crates/tokio) | 1.53.1 (`full`) | Async runtime backing `#[tokio::main]` |
| [`serde`](https://crates.io/crates/serde) | 1.0.229 (`derive`) | Derives deserialization for the config structs |
| [`toml`](https://crates.io/crates/toml) | 1.1.6 | Parses `config.toml` |

**Tooling**

- Cargo (build, run, dependency management)

## Installation

### Prerequisites

- **Rust toolchain 1.85+** — `Cargo.toml` sets `edition = "2024"`, which earlier toolchains cannot compile. Install via [rustup](https://rustup.rs/):

  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```

- A working TLS stack, since `reqwest` is used to reach `https://` targets.

### Build from source

```bash
git clone git@github.com:lustarm/beacon.git
cd beacon
cargo build --release
```

Cargo resolves and compiles all dependencies from `Cargo.lock`; there is no separate dependency-installation step.

The resulting binary is at `target/release/beacon`.

## Configuration

Beacon reads a file named **`config.toml` from the current working directory** at startup. The path is not configurable and there are no command-line flags — run the binary from a directory containing this file.

If the file is missing or fails to parse, the process exits with `[error] No config.toml file` or `[error] Failed to parse config.toml file`.

### Schema

| Key | Type | Required | Description |
| --- | --- | --- | --- |
| `defaults.interval_seconds` | integer | yes | Seconds to wait after probing a target that has no override |
| `defaults.timeout_seconds` | integer | yes | Request timeout for targets with no override |
| `targets[].name` | string | yes | Label used in log output |
| `targets[].url` | string | yes | Absolute URL to `GET` |
| `targets[].custom_interval_seconds` | integer | no | Per-target interval override |
| `targets[].custom_timeout_seconds` | integer | no | Per-target timeout override |

Both `[defaults]` keys must be present — they are non-`Option` fields, so omitting either aborts parsing.

### Example `config.toml`

```toml
[defaults]
interval_seconds = 30
timeout_seconds = 5

[[targets]]
name = "marketing-site"
url = "https://example.com"

[[targets]]
name = "client-api"
url = "https://api.example.com/health"
custom_interval_seconds = 10
custom_timeout_seconds = 3
```

Beacon requires no environment variables, API keys, or secrets. Only target URLs live in the config, so it is safe to commit — but note that a URL containing a token in its query string would be stored in plaintext, so prefer unauthenticated health endpoints.

> ⚠️ The `config.toml` currently checked into the repository uses the keys `interval_seconds` and `timeout_seconds` inside `[[targets]]`. The code deserializes `custom_interval_seconds` and `custom_timeout_seconds`; unknown keys are silently ignored, so those overrides have no effect as written. Use the `custom_`-prefixed names shown above.

## Usage

Run the release binary from a directory containing `config.toml`:

```bash
./target/release/beacon
```

Or run directly through Cargo during development:

```bash
cargo run
```

Beacon runs in the foreground until interrupted (`Ctrl+C`). Example output:

```text
[config] Loaded config correctly
[info] Requesting status code from https://example.com
[status] Status code Ok from marketing-site:https://example.com
[info] Requesting status code from https://api.example.com/health
[error] Failed to get status code from client-api:https://api.example.com/health
```

If the config parses but contains no `[[targets]]`, Beacon prints `[info] No targets in config file` and then spins in an empty loop rather than exiting.

## Project Structure

```text
beacon/
├── Cargo.toml       # Package metadata and dependency declarations
├── Cargo.lock       # Pinned dependency graph
├── config.toml      # Monitoring targets and default interval/timeout
├── .gitignore
└── src/
    └── main.rs      # Entire application: config types, probe, and polling loop
```

The project is a single binary crate. `src/main.rs` contains four logical pieces:

- **`Config` / `Defaults` / `Target`** — `serde`-derived structs mirroring the TOML schema.
- **`read_config()`** — reads and parses `config.toml`, returning a `Config`.
- **`get_status()`** — builds a `reqwest::Client` with the resolved timeout and returns the response's `StatusCode`.
- **`main()`** — the `#[tokio::main]` entry point running the monitoring loop.

## How It Works

1. **Startup.** `read_config()` loads `config.toml` into a string and hands it to `toml::from_str`, which populates the `Config` struct. Failures at either step panic with a prefixed message.
2. **Resolution.** For each target, `main()` matches on `custom_interval_seconds` and `custom_timeout_seconds`, falling back to the values in `config.defaults` when they are `None`.
3. **Probe.** `get_status()` constructs a `reqwest::Client` with the resolved timeout, issues a `GET`, and returns the response status.
4. **Report.** `main()` matches the returned status: `StatusCode::OK` logs a `[status]` line, anything else logs `[error]`.
5. **Sleep.** After each target, the loop blocks for that target's interval, then continues to the next target and repeats forever.

Note that targets are probed **sequentially within a single loop iteration**, and the sleep is a blocking `std::thread::sleep` rather than `tokio::time::sleep`. A target's `custom_interval_seconds` therefore acts as a delay *after* that target is checked, not as an independent schedule — one slow or long-interval target delays every target behind it.

## Development

```bash
cargo run            # Build and run in debug mode
cargo build          # Debug build
cargo build --release # Optimized build
cargo check          # Fast type-check without producing a binary
cargo fmt            # Format (rustfmt, ships with rustup)
cargo clippy         # Lint (requires: rustup component add clippy)
```

No custom lint, format, or CI configuration is checked in — the commands above are the stock Cargo toolchain defaults.

## Testing

The repository currently contains **no tests** — there is no `tests/` directory and no `#[cfg(test)]` module in `src/main.rs`. `cargo test` will compile the crate and report zero tests.

Contributions adding coverage are welcome; Rust's built-in test harness needs no extra dependencies.

## Known limitations

These are accurate as of the current commit and are good first contributions:

- **`[defaults]` is parsed but discarded.** `read_config()` reads the section into the `Config` struct, then returns a newly constructed `Defaults { interval_seconds: 30, timeout_seconds: 5 }`. Changing the values in `config.toml` has no effect on runtime behavior.
- **Per-target override keys are mismatched.** The shipped `config.toml` uses `interval_seconds` / `timeout_seconds`, while `Target` expects `custom_interval_seconds` / `custom_timeout_seconds`.
- **Network failures panic.** `get_status()` calls `.unwrap()` on the request result, so a timeout, DNS failure, or connection refusal aborts the process instead of reaching the `[error]` log arm. In practice the `[error]` branch is only hit for non-`200` responses that *were* successfully received.
- **Missing config aborts the process.** `read_config()` uses `.expect()` rather than returning a `Result`.
- **Blocking sleep on an async runtime.** `std::thread::sleep` parks the Tokio worker thread rather than yielding.
- **Empty target list spins.** With zero targets, `main()` enters a `loop` with nothing to iterate over and busy-waits.

## Contributing

1. Fork the repository and create a branch off `main`:
   ```bash
   git checkout -b fix/short-description
   ```
2. Make your change and confirm it compiles cleanly:
   ```bash
   cargo check && cargo clippy && cargo fmt --check
   ```
3. Verify the binary still runs against the example `config.toml`.
4. Open a pull request describing the behavior before and after your change.

If you are picking up an item from [Known limitations](#known-limitations), mention which one in the PR description.

## License

**Not specified.** No `LICENSE` file is present in the repository and `Cargo.toml` declares no `license` field, so the project is unlicensed by default — all rights reserved. *This needs clarification from the maintainer before third-party use or redistribution.*

## Acknowledgements

Built on the Rust async ecosystem — [Tokio](https://tokio.rs/), [reqwest](https://github.com/seanmonstar/reqwest), [serde](https://serde.rs/), and [toml-rs](https://github.com/toml-rs/toml).
