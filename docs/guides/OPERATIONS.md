# MAGI System — Operations & Runbook 

This guide covers container lifecycle management, persistence guarantees, resource limits, security hardening, and operational troubleshooting for MAGI System.

---

## 1. System Status & Verification

Version `0.1.4` is fully containerized and verified inside the Docker toolchain:

* `cargo check --workspace` — **Passed**
* `cargo test --workspace` — **Passed (12/12 unit, i18n & consensus tests)**
* `cargo clippy --workspace --all-targets -- -D warnings` — **Passed (0 warnings)**
* `cargo fmt --all -- --check` — **Passed (0 diffs)**
* `cargo check -p magi-server --target wasm32-unknown-unknown` — **Passed**

---

## 2. Architecture & Persistence Model

MAGI utilizes an **ephemeral container lifecycle with host-bound persistence**:

* **Database Persistence:** SpacetimeDB table state and consensus histories are bound to `./.spacetimedb_data:/stdb`. Containers can be stopped, killed, or recreated without data loss.
* **Audit Report Persistence:** Every deliberation automatically writes a human-readable Markdown summary to `./deliberations/deliberation_XXXXXX_<slug>.md`.
* **Resource Quotas:**
  * `magi-spacetimedb`: `mem_limit: 512m` (reserva `128m`), `cpus: 1.0`
  * `magi` client container: `mem_limit: 2g` (reserva `512m`), `cpus: 2.0`
* **Cargo Caches:** Crates and git dependencies are cached in Docker named volumes (`cargo_cache`, `cargo_git`) to keep the host directory clean.

---

## 3. Container Management

### Using PowerShell Automation (`magi.ps1`)

```powershell
# Start SpacetimeDB daemon and compile/publish server WASM module
.\magi.ps1 start

# Recompile the workspace
.\magi.ps1 build

# Execute test suite
.\magi.ps1 test

# Stop SpacetimeDB container
.\magi.ps1 stop

# Clean build artifacts
.\magi.ps1 clean
```

### Using Docker Compose Directly

```bash
# Start SpacetimeDB in the background
docker compose up -d spacetimedb

# Run a deliberation
docker compose run --rm magi idea docs/rfcs/ROADMAP.md

# Stop SpacetimeDB
docker compose stop spacetimedb

# Remove SpacetimeDB container (database files in ./.spacetimedb_data remain intact)
docker compose rm -f spacetimedb
```

---

## 4. Smoke Testing & Verification

### 1. Deterministic Mock Deliberation
```powershell
.\magi.ps1 idea docs/rfcs/ROADMAP.md --mock
```

### 2. Balthasar Security Veto Simulation
```powershell
.\magi.ps1 idea docs/rfcs/ROADMAP.md --mock --simulate-veto
```
*Expected Result:* `VETO_BALTHASAR_SECURITY` triggered with risk score `10/10`.

### 3. Incident Triage with Specialist Routing
```powershell
.\magi.ps1 triage logs/panic.log --code client/src/main.rs --mock
```
*Expected Flow:*
1. Specialist routing assigns opening analysis to the relevant persona (e.g. Casper-3 for config, Balthasar-2 for security, Melchior-1 for panics).
2. Specialist generates initial diagnosis.
3. Full Trinity conducts a 2-round cross-peer debate.
4. SpacetimeDB calculates and seals the atomic consensus result.

### 4. Interactive TUI Smoke Test
```powershell
.\magi.ps1 tui --mock
```

---

## 5. Security & Isolation

1. **Least Privilege Principle:** SpacetimeDB only mounts `./.spacetimedb_data:/stdb`, preventing arbitrary access to host source code or private keys.
2. **Package Hardening:** `Dockerfile.dev` runs `apt-get upgrade -y` to patch Debian base OS CVEs.
3. **Secret Isolation:** `.env` is ignored in `.gitignore` and `.dockerignore`.
4. **Fail-Closed Consensus:** If the SpacetimeDB connection is disrupted, the client reports `CONSENSUS_UNAVAILABLE` rather than fabricating an unverified verdict.
