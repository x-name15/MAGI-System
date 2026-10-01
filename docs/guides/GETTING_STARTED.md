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

## 🆓 Zero-Cost Testing Guide (100% Free AI Keys)

You can run full multi-agent deliberations completely free without entering a credit card by using any of the following providers:

### 1. Google Gemini API (Recommended — Very Fast & Free Tier)
* **Website:** [Google AI Studio](https://aistudio.google.com/)
* **Free Quota:** Generous free tier for `gemini-1.5-flash` and `gemini-1.5-pro`.
* **Config (`.env`):**
  ```bash
  MELCHIOR_PROVIDER=gemini
  MELCHIOR_API_KEY=AIzaSy...
  MELCHIOR_MODEL=gemini-1.5-flash

  BALTHASAR_PROVIDER=gemini
  BALTHASAR_API_KEY=AIzaSy...
  BALTHASAR_MODEL=gemini-1.5-flash

  CASPER_PROVIDER=gemini
  CASPER_API_KEY=AIzaSy...
  CASPER_MODEL=gemini-1.5-flash
  ```

### 2. Groq Cloud (Ultra Low Latency — Free Tier)
* **Website:** [Groq Console](https://console.groq.com/)
* **Models:** `llama-3.3-70b-versatile`, `mixtral-8x7b-32768`.
* **Config (`.env`):**
  ```bash
  MELCHIOR_PROVIDER=groq
  MELCHIOR_API_KEY=gsk_...
  MELCHIOR_MODEL=llama-3.3-70b-versatile
  ```

### 3. OpenRouter (Free Tier Models)
* **Website:** [OpenRouter](https://openrouter.ai/)
* **Models:** `meta-llama/llama-3.2-3b-instruct:free`, `google/gemini-2.0-flash-exp:free`.
* **Config (`.env`):**
  ```bash
  BALTHASAR_PROVIDER=openrouter
  BALTHASAR_API_KEY=sk-or-v1-...
  BALTHASAR_MODEL=meta-llama/llama-3.2-3b-instruct:free
  ```

### 4. Fully Offline Simulation (`MOCK`)
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
