# Getting Started with MAGI System 

This guide will get you up and running with MAGI System in less than 2 minutes.

---

## Quick Start

### Option A: PowerShell Runner (Windows)

The simplest way to run MAGI on Windows with Docker Desktop:

```powershell
# 1. Clone the repository
git clone https://github.com/x-name15/magi-system.git
cd magi-system

# 2. Copy the sample environment file and configure your keys
Copy-Item .env.example .env

# 3. Start SpacetimeDB and build the modules
.\magi.ps1 start
.\magi.ps1 build

# 4. Run your first audit deliberation!
.\magi.ps1 idea docs/rfcs/ROADMAP.md
```

### Option B: Docker Compose Directly (Linux / macOS / Windows)

```bash
# 1. Clone and enter directory
git clone https://github.com/x-name15/magi-system.git
cd magi-system
cp .env.example .env

# 2. Start SpacetimeDB in background
docker compose up -d spacetimedb

# 3. Execute deliberation via client container
docker compose run --rm magi idea docs/rfcs/ROADMAP.md
```

---

## Running Workflows

```powershell
# Review an RFC or Architecture proposal
.\magi.ps1 idea docs/rfcs/ROADMAP.md

# Review code under team guidelines
.\magi.ps1 maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md

# Diagnose a panic or error log
.\magi.ps1 triage logs/panic.log --code client/src/main.rs

# Launch the interactive full-screen TUI console
.\magi.ps1 tui
```

For complete CLI options, flags, and TUI commands, see the [CLI & TUI Manual](CLI_AND_TUI.md).

---

## Environment Variables Reference

| Variable | Description | Default |
| :--- | :--- | :--- |
| `SPACETIMEDB_URI` | SpacetimeDB WebSocket/HTTP connection endpoint | `http://spacetimedb:3000` |
| `SPACETIMEDB_DATABASE` | Published database module name | `magi-system` |
| `MAGI_TIMEOUT_SECONDS` | Maximum timeout per node request in seconds | `60` |
| `MAGI_AUTHOR` | Author/operator identifier for deliberations | `developer@magi` |
| `MAGI_LANG` | Language override (`es` for Spanish, `en` for English, or auto-detected) | Auto |
| `MELCHIOR_PROVIDER` | Provider for Node 1 (`gemini`, `openai`, `anthropic`, `grok`, `deepseek`, `ollama`, `mock`) | Auto-inferred |
| `MELCHIOR_MODEL` | Model ID for Node 1 | Canonical model |
| `BALTHASAR_PROVIDER` | Provider for Node 2 | Auto-inferred |
| `BALTHASAR_MODEL` | Model ID for Node 2 | Canonical model |
| `CASPER_PROVIDER` | Provider for Node 3 | Auto-inferred |
| `CASPER_MODEL` | Model ID for Node 3 | Canonical model |
