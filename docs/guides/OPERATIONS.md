# MAGI System — Operations & Runbook 

This runbook covers container lifecycle management, persistence guarantees, resource limits, verification checklists, database purging, and operational troubleshooting for **MAGI System v0.2.6**.

---

## 1. System Status & Verification Checklist

Version `0.2.6` is verified inside the Docker toolchain with zero warnings:

* `cargo check --workspace` — **Passed**
* `cargo test --workspace` — **Passed (72/72 unit, consensus, i18n & MCP tests)**
* `cargo clippy --workspace --all-targets -- -D warnings` — **Passed (0 warnings)**
* `cargo fmt --all -- --check` — **Passed (0 diffs)**
* `cargo check -p magi-server --target wasm32-unknown-unknown` — **Passed**

---

## 2. Architecture & Persistence Model

* **Database State Persistence:** SpacetimeDB table state and consensus histories are persisted in a dedicated Docker named volume (`spacetimedb_data:/stdb`) running under `user: root`. Containers can be stopped, restarted, or recreated without data loss or host permission issues.
* **Audit Report Persistence:** Every deliberation automatically writes a human-readable Markdown summary to host storage at `./deliberations/deliberation_XXXXXX_<slug>.md`.
* **Zero Host Pollution:** Rust compilation target caches (`cargo_target:/workspace/target`) and package dependencies (`cargo_cache`, `cargo_git`) reside in dedicated Docker volumes, preventing multi-gigabyte build artifacts from cluttering the host workstation.
* **Resource Quotas (`docker-compose.yml`):**
  * `magi-spacetimedb`: `mem_limit: 512m` (reservation `128m`), `cpus: 1.0`
  * `magi` client container: `mem_limit: 2g` (reservation `512m`), `cpus: 2.0`

---

## 3. Container Management

### Using PowerShell Automation (`magi.ps1`)

```powershell
# Start SpacetimeDB daemon in background
.\magi.ps1 start

# Recompile client binary and publish SpacetimeDB WASM module
.\magi.ps1 build

# Execute workspace test suite
.\magi.ps1 test

# Stop SpacetimeDB daemon
.\magi.ps1 stop

# Clean build artifacts
.\magi.ps1 clean

# Purge SpacetimeDB tables & clean local deliberation files
.\magi.ps1 purge
.\magi.ps1 purge --keep-files
.\magi.ps1 purge --hard
```

### Using Docker Compose Directly (Linux / macOS)

```bash
# Start SpacetimeDB in background
docker compose up -d spacetimedb

# Recompile and run tests
docker compose run --rm magi cargo test --workspace

# Stop SpacetimeDB
docker compose stop spacetimedb

# Wipe SpacetimeDB container and named volume
docker compose down -v
```

---

## 4. Smoke Testing & Verification

### 1. Deterministic Mock Deliberation
Verify the complete 2-round consensus pipeline without consuming LLM API credits:
```powershell
.\magi.ps1 debate "Test proposal" --mock
```

### 2. Balthasar Security Veto Simulation
Verify Balthasar's unilateral veto threshold (`risk_score >= 8` overrides affirmative majority):
```powershell
.\magi.ps1 debate "Veto simulation" --mock --simulate-veto
```
*Expected Result:* `VETO_BALTHASAR_SECURITY` with risk score `10/10`.

### 3. Incident Triage with Specialist Routing
Verify specialist lead persona assignment and subsequent Trinity consensus:
```powershell
.\magi.ps1 triage logs/panic.log --code client/src/main.rs --mock
```

### 4. Interactive History Browser Smoke Test
```powershell
.\magi.ps1 history --table
```

---

## 5. Maintenance & State Purging

### Soft Database Purge
Wipes in-memory tables in SpacetimeDB and clears local `./deliberations/*.md` files:
```powershell
.\magi.ps1 purge
```

### Preserve Audit Markdown Files
Wipes SpacetimeDB while leaving your disk reports in `./deliberations/` untouched:
```powershell
.\magi.ps1 purge --keep-files
```

### Hard Factory Reset
Stops all Docker containers, removes the SpacetimeDB Docker volume (`spacetimedb_data`), and deletes local reports:
```powershell
.\magi.ps1 purge --hard
```

---

## 6. Operational Troubleshooting

| Symptom | Probable Cause | Remediation |
| :--- | :--- | :--- |
| `Cannot connect to SpacetimeDB at http://spacetimedb:3000` | The daemon is not running or still initializing. | Run `.\magi.ps1 start` and check logs with `docker compose logs spacetimedb`. |
| Port `3000` is already in use | Another local service (e.g. Node, Grafana) is listening on port 3000. | Stop the conflicting process or change host port binding in `docker-compose.yml`. |
| `HTTP 402 Payment Required` from OpenRouter | Primary model account has exhausted credits. | Configure `MAGI_FALLBACK_MODEL=cohere/north-mini-code:free` in `.env` to engage hot-standby backup. |
| `HTTP 429 Too Many Requests` | Provider rate limit exceeded. | MAGI automatically applies exponential backoff up to `MAGI_MAX_RETRIES`. Increase `MAGI_RETRY_DELAY_MS`. |
| PowerShell encoding / parse errors | Windows PowerShell 5.1 interpreting UTF-8 characters incorrectly. | `magi.ps1` is hardened for ASCII compatibility. Run with `powershell -ExecutionPolicy Bypass -File .\magi.ps1`. |
