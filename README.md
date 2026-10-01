# MAGI System 🧠

**Tactical Multi-Agent Consensus Engine inspired by Neon Genesis Evangelion**

[![CI](https://github.com/x-name15/magi-system/actions/workflows/ci.yml/badge.svg)](https://github.com/x-name15/magi-system/actions/workflows/ci.yml)
[![CodeQL](https://github.com/x-name15/magi-system/actions/workflows/codeql.yml/badge.svg)](https://github.com/x-name15/magi-system/actions/workflows/codeql.yml)
[![Release](https://img.shields.io/github/v/release/x-name15/magi-system?include_prereleases&label=release)](https://github.com/x-name15/magi-system/releases)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![SpacetimeDB](https://img.shields.io/badge/SpacetimeDB-Consensus-orange.svg)](https://spacetimedb.com)
[![Docker](https://img.shields.io/badge/Docker-Ready-2496ED.svg)](docker-compose.yml)

> 🧪 **Project Spirit:** A personal, experimental playground built to research distributed multi-agent consensus algorithms combined with [SpacetimeDB](https://spacetimedb.com). It faithfully brings the iconic supercomputer MAGI from *Neon Genesis Evangelion* to life in your terminal.

---

## What is MAGI System?

Instead of relying on a single LLM prompt that may suffer from hallucinations, blind spots, or confirmation bias, **MAGI System** audits technical proposals, source code, and incidents through a **parallel 2-round debate protocol** across three specialized archetypes:

```text
                     ┌───────────────────────┐
                     │      BALTHASAR-2      │
                     │   MOTHER · SECURITY   │
                     │RISK [███░░░░░░░]  3/10│
                     │ R2:  95%  LAT: 2500ms │
                     ├───────────────────────┤
                     │    ██ AGREEMENT ██    │
                     │     [  AGREEMENT  ]   │
                     └───────────┬───────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                 │
    ┌───────────┴───────────┐         ┌───────────┴───────────┐
    │       CASPER-3        │         │      MELCHIOR-1       │
    │  WOMAN · PRAGMATICS   │         │   SCIENTIST · ARCH    │
    │RISK [█████████░]  9/10│         │RISK [███░░░░░░░]  3/10│
    │ R2: 100%  LAT: 3200ms │         │ R2:  95%  LAT: 2700ms │
    ├───────────────────────┼─────────┼───────────────────────┤
    │     ██ DENIAL ██      │         │    ██ AGREEMENT ██    │
    │     [    DENIAL   ]   │         │     [  AGREEMENT  ]   │
    └───────────────────────┘         └───────────────────────┘
```

* **Melchior-1 (The Scientist):** Logic, clean architecture, algorithmic complexity, and scalability.
* **Balthasar-2 (The Mother):** Defensive cybersecurity, OWASP Top 10, CWE classification, and **unilateral security veto power** (`risk >= 8`).
* **Casper-3 (The Woman):** Real-world pragmatism, delivery velocity, developer experience (DX), and **zero tolerance for overengineering**.

Deliberations are deterministically collected, evaluated, and sealed by an in-memory transactional database ([SpacetimeDB](https://spacetimedb.com)) running WebAssembly (WASM).

---

## Quick Start (in 60 seconds)

### Windows (PowerShell)
```powershell
# 1. Setup environment
Copy-Item .env.example .env

# 2. Start SpacetimeDB & build modules
.\magi.ps1 start
.\magi.ps1 build

# 3. Deliberate!
.\magi.ps1 idea docs\IDEA.md
```

### Linux / macOS (Docker Compose)
```bash
cp .env.example .env
docker compose up -d spacetimedb
docker compose run --rm magi idea docs/IDEA.md
```

### Pre-compiled Standalone Binaries
Download ready-to-run binaries from [GitHub Releases](https://github.com/x-name15/magi-system/releases):
* **Linux (x86_64):** `magi-linux-amd64.tar.gz`
* **Windows (x86_64):** `magi-windows-amd64.zip` (`magi.exe`)

---

## Operational Modes & Commands

MAGI operates both as a scriptable one-shot CLI and as an interactive full-screen retro TUI console:

### 1. One-Shot Scriptable CLI
Direct commands that output diegetic NERV monitors and detailed findings straight to `stdout`:

| Command | Purpose | Example |
| :--- | :--- | :--- |
| `idea` | Audits an RFC or technical proposal for viability | `.\magi.ps1 idea docs\IDEA.md` |
| `maintain` | Checks code against architecture guidelines | `.\magi.ps1 maintain src\lib.rs --guidelines docs\OPERATIONS.md` |
| `triage` | Classifies errors, assigns a lead specialist, and debates | `.\magi.ps1 triage logs\panic.log --code src\main.rs` |
| `history` | Inspects previous deliberation records in SpacetimeDB | `.\magi.ps1 history` |
| `status` | Pings the SpacetimeDB transactional engine | `.\magi.ps1 status` |

### 2. Interactive Full-Screen NERV TUI Console
Launch the full-screen terminal dashboard (`ratatui`) with keyboard navigation and natural-language intent understanding:
```powershell
.\magi.ps1 tui       # (or .\magi.ps1 console)
```
Inside the console, you can query system status (`status`, `history`), or type free-form requests in natural language (e.g. `revisa docs/IDEA.md`, `panic at connection pool: index out of bounds`).

---

## Documentation Index

Deep dive into the architecture, configuration, and internal protocols:

- **[Getting Started & Free AI Keys Guide](docs/guides/GETTING_STARTED.md):** Zero-cost setup with Google Gemini, Groq, OpenRouter, or offline `MOCK`.
- **[The Evangelion Trinity & Debate Protocol](docs/architecture/TRINITY.md):** The three archetypes, 2-round cross-examination, and decision matrix.
- **[System Architecture](docs/architecture/ARCHITECTURE.md):** SpacetimeDB transactional engine, WASM reducers, and multi-agent concurrency.
- **[Operations Runbook](docs/guides/OPERATIONS.md):** Docker container lifecycle, volume persistence, and deployment.
- **[Project Roadmap](docs/rfcs/ROADMAP.md):** Future milestones, V1 verification, and upcoming features.

---

## License

Licensed under the **GNU General Public License v3.0 (GPL-3.0)**. See [LICENSE](LICENSE) for details.

### Credits
**Author:** Mr Jacket / Felix Manrique / x-name15 (we are all the same person)