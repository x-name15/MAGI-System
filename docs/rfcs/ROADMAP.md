# MAGI System — Project Roadmap & Status 

MAGI System is a **personal developer tool and tactical AI consensus companion** inspired by the supercomputer from *Neon Genesis Evangelion*. It operates as an asynchronous multi-agent code auditor and architectural referee for local pair programming and autonomous IDE workflows.

---

## Current Status & Evolution

| Version | Status | Focus | Core Deliverables |
| :--- | :--- | :--- | :--- |
| **0.2.6** | **Completed** | Subagents, Server Tools & Localized Consensus | OpenRouter server tools (`openrouter:subagent`, `openrouter:web_search`), specialized worker personas (`systems_analyst`, `security_scanner`, `pragmatic_evaluator`), live worker telemetry, and localized debate fallbacks. |
| **0.2.5** | **Completed** | Hot-Standby Failover & Truncation Recovery | Hot-standby backup models (`MAGI_FALLBACK_MODEL`), configurable token limits (`MAGI_MAX_TOKENS`), JSON truncation recovery, native `magi purge`, and test isolation. |
| **0.2.4** | **Completed** | Interactive History Explorer & Hybrid Archive | Unified SpacetimeDB + disk reader for `deliberations/`, interactive Ratatui history browser (`magi history`), and hybrid `magi show <id>`. |
| **0.2.3** | **Completed** | Resilience, Degraded Quorum & Root Archiving | Fault-tolerant 2-of-3 quorum, exponential backoff retries (anti-429), context budgeting safeguards, and canonical root `deliberations/` persistence. |
| **0.2.2** | **Completed** | Smart Ingestion & Context Discovery | Automatic project context discovery (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`), smart noise/lockfile filtering in diffs, and integration with `lineamientos-pw`. |
| **0.2.1** | **Completed** | Git Diff Auditing, Dilemma Debates & Expanded MCP Suite | `magi diff` (working tree, staged, branch), `magi debate` (architectural dilemmas), and 5 specialized MCP tools. |
| **0.2.0** | **Completed** | Modular Architecture & Protocol Integration | Decoupled node helpers, Model Context Protocol (`magi mcp`), pure local consensus engine, multi-round debates, and i18n localization. |

---

## 📦 Milestone Breakdown

### 🔹 0.2.6 — OpenRouter Server Tools & Subagent Ecosystem (Completed)
* [x] **OpenRouter Server Tools Integration:** Natively delegates micro-level tasks to smaller/faster models mid-generation using `openrouter:subagent`.
* [x] **Specialized Persona Workers:** Melchior delegates to `systems_analyst`, Balthasar to `security_scanner`, and Casper to `pragmatic_evaluator`.
* [x] **Live Subagent Telemetry:** Real-time console logs when workers are invoked.
* [x] **Anti-Recursion Safeguard:** Prevents self-reference loops if worker matches calling node model.
* [x] **Localized Rationale Fallbacks:** Fully internationalized debate conclusion fallbacks across `es.json` and `en.json`.

---

### 🔹 0.2.5 — Hot-Standby Failover & Truncation Recovery (Completed)
* [x] **Contingency Backup Brain (`MAGI_FALLBACK_MODEL`):** Transparent failover when primary model runs out of credits (HTTP 402) or suffers outages.
* [x] **Configurable Token Budgets (`MAGI_MAX_TOKENS`):** Explicit token limits resolving cutoff issues with reasoning models.
* [x] **JSON Truncation Recovery:** Gracefully balances quotes, braces, and brackets in cut-off LLM streams.
* [x] **Native `magi purge` Subcommand:** Cleans SpacetimeDB state and local Markdown archives.
* [x] **Host Isolation in Tests:** Zero test markdown leakage to the developer's workspace.

---

### 🔹 0.2.4 — Interactive History Explorer & Hybrid Archive (Completed)
* [x] **Hybrid Archive Loader:** Merges records from SpacetimeDB tables with local Markdown reports in `deliberations/`.
* [x] **Interactive TUI History Browser (`magi history`):** Ratatui dual-pane browser with live keyboard navigation, filter queries (`/`), and modal report reader (`Enter`).
* [x] **Hybrid `magi show <id>`:** Retrieves and displays past deliberations even if SpacetimeDB was wiped.

---

### 🔹 Upcoming: 0.3.0 — Deep Codebase Awareness & Dependency Graphing (Planned)
* [ ] **Cross-File Context Ingestion:** Tree-sitter / AST based dependency graphs for multi-file impact analysis during `diff` and `maintain`.
* [ ] **Automated Benchmark Extraction:** Performance regression detection by parsing benchmark outputs in code maintenance audits.
* [ ] **Local Embedded Engine (Single Binary):** Experimental zero-daemon mode bundling SpacetimeDB in-process without Docker requirement.
