# MAGI System Operations Guide

## Status

Version `0.1.3` is ready for a local end-to-end smoke test through Docker.

The following checks have passed inside the Docker toolchain:

- `cargo check --workspace`
- `cargo test --workspace` with 7 consensus tests passing
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo fmt --all -- --check`
- `cargo check -p magi-server --target wasm32-unknown-unknown`
- Docker Compose configuration validation
- Release WASM build and local SpacetimeDB publication

The local module is published as the SpacetimeDB database `magi-system`.

## What Was Corrected

### MAGI deliberation

- Added a two-round Trinity protocol.
- Round one produces independent Melchior, Balthasar, and Casper positions.
- Round two gives those positions back to all three nodes for challenge and final voting.
- Triage uses a specialist for the opening diagnosis, but the final decision always requires the full Trinity.
- Balthasar's `REJECT` with risk `8..=10` still has unilateral veto authority.
- Final consensus is calculated atomically by the SpacetimeDB reducer.

### Integrity and concurrency

- Deliberations use a unique request correlation ID.
- Unknown nodes, duplicate votes, invalid postures, invalid assignments, and invalid risk scores are rejected by the server.
- Timeout handling cancels provider futures instead of leaving detached tasks running.
- Database failures no longer become fabricated local consensus results; the UI reports `CONSENSUS_UNAVAILABLE`.

### TUI and reporting

- Replaced the line-oriented console with `ratatui` and `crossterm`.
- Renamed the module to `client/src/ui/tui.rs`.
- The TUI keeps a visible transcript of verdicts, summaries, risks, votes, and final positions.
- Markdown reports are persisted in `deliberations/`.

### Docker and toolchain

- The development image uses Rust `1.90-slim-bookworm`, required by SpacetimeDB `1.12`.
- The unused interactive SpacetimeDB CLI installer was removed from the development image.
- `.dockerignore` excludes secrets, build artifacts, database state, and reports from the build context.
- `magi-dev` is behind the `dev` Compose profile and has a 2 GB memory limit and 2 CPU limit.
- SpacetimeDB has a 512 MB memory limit and 1 CPU limit.
- `docker compose up` starts only SpacetimeDB; the development container is started explicitly.

## Provider Variables

Each node has its own provider selector:

- `MELCHIOR_PROVIDER`
- `BALTHASAR_PROVIDER`
- `CASPER_PROVIDER`

These variables select the HTTP protocol adapter, not the MAGI persona. Supported values are:

- `openai`: OpenAI-compatible `/chat/completions` endpoints, including compatible Gemini, Groq, OpenRouter, and DeepSeek endpoints.
- `anthropic`: Anthropic `/v1/messages` endpoint.
- `ollama`: Ollama `/api/generate` endpoint.

The persona remains fixed by the node module. The provider, model, endpoint, and API key are configurable independently for each node.

## First-Time Local Setup

1. Copy `.env.example` to `.env`.
2. Put test credentials in `.env` if using remote providers. Do not commit `.env`.
3. Keep `SPACETIMEDB_DATABASE=magi-system`; this is the published local module name.
4. Start the database:

```powershell
docker compose up -d spacetimedb
```

5. Build the server module:

```powershell
docker compose run --rm magi-dev cargo build -p magi-server --target wasm32-unknown-unknown --release
```

6. Publish the WASM module to the local SpacetimeDB instance:

```powershell
docker compose exec spacetimedb spacetime publish `
  --server local `
  --bin-path /workspace/target/wasm32-unknown-unknown/release/magi_server.wasm `
  --yes magi-system
```

The `spacetimedb` service mounts the workspace read-only at `/workspace` only for this publication step.

## Smoke Tests

### Deterministic mock deliberation

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- idea docs/IDEA.md --mock
```

### Balthasar veto simulation

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- idea docs/IDEA.md --mock --simulate-veto
```

Expected result: `VETO_BALTHASAR_SECURITY`.

### Universal prompt-only deliberation

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- deliberate `
  --prompt "Should this service use a queue or synchronous HTTP for critical writes?" `
  --mock
```

### Error triage through the full Trinity

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- triage `
  "Error: JWT signature validation failed" `
  --code fixtures/sample_auth_service.rs `
  --mock
```

Expected flow:

1. The classifier selects the opening specialist.
2. The specialist produces an opening diagnosis.
3. All three nodes debate the incident.
4. The three final positions are persisted.
5. SpacetimeDB calculates and persists the final verdict.

### TUI

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- console --mock
```

Use keyboard input in the terminal UI. Press `Enter` to submit a request and `Esc` to leave the interface.

## Inspecting Results

```powershell
docker compose run --rm magi-dev cargo run --bin magi -- status
docker compose run --rm magi-dev cargo run --bin magi -- history
docker compose run --rm magi-dev cargo run --bin magi -- show <DELIBERATION_ID>
```

Human-readable reports are saved under `deliberations/`. SpacetimeDB state is stored under `.spacetimedb_data/`.

## Shutdown

Stop the database without deleting persisted state:

```powershell
docker compose stop spacetimedb
```

Remove the database container while retaining host-bound state:

```powershell
docker compose rm -f spacetimedb
```

Do not delete `.spacetimedb_data/` unless a clean database is intentionally required.

## Documentation Layout

- `README.md`: project overview and public quick reference.
- `CHANGELOG.md`: release source of truth used by automation.
- `docs/OPERATIONS.md`: this setup, deployment, and testing guide.
- `docs/ARCHITECTURE.md`: system design and data flow.
- `docs/IDEA.md`: original project proposal and evolution.
- `docs/LOG.md`: engineering decisions.
- `docs/ROADMAP.md`: future work.
- `.github/*.md`: GitHub governance and contribution policies.
- `fixtures/*.md`: test input data, not project documentation.
