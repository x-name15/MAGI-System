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

---

## LLM Configuration (100% Vendor-Agnostic)

MAGI speaks standard **OpenAI Chat Completions REST API** (`POST /chat/completions`). It has zero hardcoded vendor dependencies, allowing you to connect any cloud provider, local inference engine, or proxy gateway effortlessly.

> **Comprehensive Configuration Guide:** For detailed examples covering OpenRouter, free-tier routers, hybrid models, and granular per-node API keys, see **[Model & API Configuration Guide (CONFIGURATION.md)](CONFIGURATION.md)**.

### 1. Global Setup (Single endpoint for all 3 nodes)

Set these in `.env`:

* **100% Offline / Local Models (Zero API key needed):**
  ```env
  # Ollama (runs at localhost:11434/v1)
  MAGI_ENDPOINT=http://localhost:11434/v1
  MAGI_MODEL=qwen2.5-coder:7b
  MAGI_API_KEY=
  ```
  *(Compatible with LM Studio on `http://localhost:1234/v1`, vLLM on `http://localhost:8000/v1`, LocalAI, etc.)*

* **Cloud Providers (Online):**
  ```env
  # OpenAI
  MAGI_ENDPOINT=https://api.openai.com/v1
  MAGI_MODEL=gpt-4o-mini
  MAGI_API_KEY=sk-...

  # DeepSeek
  MAGI_ENDPOINT=https://api.deepseek.com/v1
  MAGI_MODEL=deepseek-chat
  MAGI_API_KEY=sk-...

  # Google Gemini (via official OpenAI-compatible endpoint)
  MAGI_ENDPOINT=https://generativelanguage.googleapis.com/v1beta/openai
  MAGI_MODEL=gemini-2.5-flash
  MAGI_API_KEY=AIza...
  ```

* **Anthropic Claude (via OpenAI Proxy):**
  Because direct Anthropic APIs use proprietary headers, in an agnostic architecture you simply route Claude requests through any standard OpenAI-compatible gateway (e.g. OpenRouter or LiteLLM):
  ```env
  # Via OpenRouter
  MAGI_ENDPOINT=https://openrouter.ai/api/v1
  MAGI_MODEL=anthropic/claude-3.5-sonnet
  MAGI_API_KEY=sk-or-...
  ```

* **Offline Mock Mode (Zero configuration):**
  If no endpoint or API keys are specified, MAGI runs in deterministic Mock mode for offline testing and CI workflows.

### 2. Granular Per-Node Overrides

You can optionally assign different models, endpoints, or keys to specific Trinity personas:
```env
# E.g. Melchior on local Ollama, Balthasar on security-specialized model
MELCHIOR_MODEL=qwen2.5-coder:7b
BALTHASAR_ENDPOINT=https://api.openai.com/v1
BALTHASAR_MODEL=gpt-4o
BALTHASAR_API_KEY=sk-...
```

---

## Environment Variables Reference

| Variable | Description | Default |
| :--- | :--- | :--- |
| `SPACETIMEDB_URI` | SpacetimeDB connection endpoint | `http://spacetimedb:3000` |
| `SPACETIMEDB_DATABASE` | Published database module name | `magi-system` |
| `MAGI_TIMEOUT_SECONDS` | Maximum timeout per node request in seconds | `60` |
| `MAGI_AUTHOR` | Author/operator identifier for deliberations | `developer@magi` |
| `MAGI_LANG` | Language override (`es`, `en`, or auto-detected) | Auto |
| `MAGI_ENDPOINT` | Global OpenAI-compatible API base URL | `http://localhost:11434/v1` |
| `MAGI_MODEL` | Global model name for all 3 nodes | `default` |
| `MAGI_API_KEY` | Global API key (optional for local models) | None |
| `{NODE}_ENDPOINT` | Granular override (`MELCHIOR_ENDPOINT`, etc.) | Falls back to global |
| `{NODE}_MODEL` | Granular override (`BALTHASAR_MODEL`, etc.) | Falls back to global |
| `{NODE}_API_KEY` | Granular override (`CASPER_API_KEY`, etc.) | Falls back to global |
