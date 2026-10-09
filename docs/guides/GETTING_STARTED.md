# Getting Started with MAGI System 

This guide will get you up and running with MAGI System in less than 2 minutes.

---

## 1. Quickstart

### Option A: Windows PowerShell Runner (Recommended for Windows)

The simplest way to run MAGI on Windows with Docker Desktop:

```powershell
# 1. Clone the repository
git clone https://github.com/x-name15/magi-system.git
cd magi-system

# 2. Setup your environment file
Copy-Item .env.example .env
# Edit .env to configure your OpenRouter or local model keys

# 3. Start SpacetimeDB daemon and compile WASM modules
.\magi.ps1 start
.\magi.ps1 build

# 4. Run your first audit deliberation!
.\magi.ps1 debate "Should we store JWTs in HttpOnly cookies or localStorage for an internal SPA?"
```

---

### Option B: Linux / macOS (Docker Compose Directly)

```bash
# 1. Clone and enter directory
git clone https://github.com/x-name15/magi-system.git
cd magi-system
cp .env.example .env

# 2. Start SpacetimeDB in background
docker compose up -d spacetimedb

# 3. Execute deliberation via client container
docker compose run --rm magi debate "Should we store JWTs in HttpOnly cookies or localStorage for an internal SPA?"
```

---

### Option C: Pre-compiled Standalone Binaries

Download ready-to-run releases from [GitHub Releases](https://github.com/x-name15/magi-system/releases):
* **Linux (x86_64):** `magi-linux-amd64.tar.gz`
* **Windows (x86_64):** `magi-windows-amd64.zip` (`magi.exe`)

```powershell
# Run directly against a running SpacetimeDB instance
magi debate "PostgreSQL vs ClickHouse for audit log telemetry"
```

---

## 2. Core Workflows

Once installed, use MAGI across these developer workflows:

### 1. Audit Git Changes (`diff`)
Audit your working tree, staged index deltas, or branch comparisons before committing or opening a PR:
```powershell
.\magi.ps1 diff --staged
```

### 2. Debate Technical Dilemmas (`debate`)
Submit an architectural trade-off or technology decision to Trinity cross-examination:
```powershell
.\magi.ps1 debate "Adopt Argon2id vs BCrypt for credential hashing in an auth service"
```

### 3. Review RFCs & Architecture Proposals (`idea`)
Audit an architectural specification for theoretical soundness, security threats, and overengineering:
```powershell
.\magi.ps1 idea docs/rfcs/ROADMAP.md
```

### 4. Code Maintenance under Guidelines (`maintain`)
Audit code against team standards, clean code rules, or personal guidelines:
```powershell
.\magi.ps1 maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md
```

### 5. Diagnose Panics & Incidents (`triage`)
Route runtime errors and panics to a lead specialist, followed by full Trinity consensus:
```powershell
.\magi.ps1 triage logs/panic.log --code client/src/main.rs
```

### 6. Browse Past Deliberations (`history`)
Launch the interactive dual-pane TUI History Browser to inspect deliberation audit trails:
```powershell
.\magi.ps1 history
```

### 7. Interactive NERV Command Deck (`tui`)
Launch the full-screen terminal dashboard:
```powershell
.\magi.ps1 tui
```

### 8. Run as Model Context Protocol (MCP) Server (`mcp`)
Connect MAGI to autonomous IDE coding assistants (Antigravity, Claude Desktop, Cursor):
```powershell
.\magi.ps1 mcp
```

---

## 3. Next Steps

* **[Model & API Configuration Guide](CONFIGURATION.md):** Configure hot-standby failover models, OpenRouter subagent workers, local models (Ollama), and granular per-node API keys.
* **[CLI & TUI Manual](CLI_AND_TUI.md):** Complete reference for flags (`--rounds`, `--output json`, `--skill`), semantic exit codes, and MCP tools.
* **[Operations Runbook](OPERATIONS.md):** Container lifecycle, volume persistence, purging databases, and troubleshooting.
