# Getting Started with MAGI System 🚀

This guide will get you up and running with MAGI System in less than 2 minutes, including zero-cost options to test with free LLM keys or offline mock simulations.

---

## ⚡ Quick Start (Windows & Linux)

### Option A: Using the PowerShell Runner (Windows)

The simplest way to run MAGI on Windows with Docker Desktop:

```powershell
# 1. Clone the repository
git clone https://github.com/x-name15/magi-system.git
cd magi-system

# 2. Copy the sample environment file
Copy-Item .env.example .env

# 3. Start SpacetimeDB and build the modules
.\magi.ps1 start
.\magi.ps1 build

# 4. Run your first audit deliberation!
.\magi.ps1 idea docs\IDEA.md
```

### Option B: Using Docker Compose Directly (Linux / macOS / Windows)

```bash
# 1. Start SpacetimeDB in the background
docker compose up -d spacetimedb

# 2. Publish the WASM consensus server module
docker compose run --rm magi-dev spacetime publish --project-path /workspace/server magi-system

# 3. Execute a deliberation via the client CLI
docker compose run --rm magi-dev /workspace/client/target/release/magi idea /workspace/docs/IDEA.md
```

### Option C: Using Pre-compiled Binaries (GitHub Releases)

Download pre-built standalone binaries from [GitHub Releases](https://github.com/x-name15/magi-system/releases):
* **Linux (x86_64):** `magi-linux-amd64.tar.gz`
* **Windows (x86_64):** `magi-windows-amd64.zip` (contains `magi.exe`)

---

### Fully Offline Simulation (`MOCK`)
To run without any internet access or API keys (ideal for CI/CD or smoke testing):
```bash
MELCHIOR_PROVIDER=mock
BALTHASAR_PROVIDER=mock
CASPER_PROVIDER=mock
```

---

## 💻 CLI Commands Reference

### 1. Idea & Architecture Review
Audits an RFC, design document, or idea for viability and balance:
```powershell
.\magi.ps1 idea docs\IDEA.md
```

### 2. Code Maintenance under Guidelines
Audits source code adherence against team rules or conventions:
```powershell
.\magi.ps1 maintain client\src\main.rs --guidelines docs\OPERATIONS.md
```

### 3. Incident & Error Triage
Classifies errors, routes root cause analysis to a specialist, and deliberates with the full Trinity:
```powershell
.\magi.ps1 triage logs\panic.log --code src\lib.rs
```

### 4. Interactive Terminal UI (TUI)
Launches the full interactive terminal dashboard to inspect previous deliberations and query SpacetimeDB in real-time:
```powershell
.\magi.ps1 tui
```

---

## 🛠️ Environment Variables Reference

| Variable | Description | Default |
| :--- | :--- | :--- |
| `SPACETIMEDB_URI` | SpacetimeDB WebSocket/HTTP connection endpoint | `http://spacetimedb:3000` |
| `SPACETIMEDB_MODULE` | Published database module name | `magi-system` |
| `MAGI_TIMEOUT_SECONDS` | Maximum timeout per node request in seconds | `60` |
| `MELCHIOR_PROVIDER` | Provider for Node 1 (`gemini`, `openai`, `groq`, `openrouter`, `mock`) | `mock` |
| `MELCHIOR_MODEL` | Model ID for Node 1 | `gemini-1.5-flash` |
| `BALTHASAR_PROVIDER` | Provider for Node 2 (`gemini`, `openai`, `groq`, `openrouter`, `mock`) | `mock` |
| `BALTHASAR_MODEL` | Model ID for Node 2 | `gemini-1.5-flash` |
| `CASPER_PROVIDER` | Provider for Node 3 (`gemini`, `openai`, `groq`, `openrouter`, `mock`) | `mock` |
| `CASPER_MODEL` | Model ID for Node 3 | `gemini-1.5-flash` |
