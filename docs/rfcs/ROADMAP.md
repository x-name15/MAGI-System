# MAGI System — Project Roadmap & Status

MAGI System is a **personal developer tool and tactical AI consensus companion** inspired by the Supercomputer from Neon Genesis Evangelion. It operates as an asynchronous multi-agent code auditor and architectural referee for local pair programming and autonomous IDE workflows.

---

## Current Status & Evolution

| Version | Status | Focus | Core Deliverables |
| :--- | :--- | :--- | :--- |
| **0.2.4** | **In Progress** | Interactive History Explorer & Hybrid Archive | Unified SpacetimeDB + disk reader for `deliberations/`, interactive Ratatui history browser (`magi history`), and hybrid `magi show <id>`. |
| **0.2.3** | **Completed** | Resilience, Degraded Quorum & Root Archiving | Fault-tolerant 2-of-3 quorum, exponential backoff retries (anti-429), context budgeting safeguards, and canonical root `deliberations/` persistence. |
| **0.2.2** | **Completed** | Smart Ingestion & Context Discovery | Automatic project context discovery (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`), smart noise/lockfile filtering in diffs, and integration with `lineamientos-pw`. |
| **0.2.1** | **Completed** | Git Diff Auditing, Dilemma Debates & Expanded MCP Suite | `magi diff` (working tree, staged, branch), `magi debate` (architectural dilemmas), and 5 specialized MCP tools. |
| **0.2.0** | **Completed** | Modular Helpers, Deep Deliberation & MCP Integration | Helper separation, Model Context Protocol (`magi mcp` over stdio), pure local consensus engine, multi-round debates, and i18n localization. |

---

## 📦 Milestone Breakdown

### 🔹 0.2.4 — Interactive History Explorer & Hybrid Archive (Active)
* [ ] **Hybrid Archive Loader:** Automatically detects and merges deliberation records from both SpacetimeDB and local Markdown files in `deliberations/`.
* [ ] **Interactive TUI History Browser (`magi history`):** Ratatui dual-pane browser with keyboard navigation (`↑`/`↓`), real-time node vote inspector (Melchior, Balthasar, Casper), and instant report viewer.
* [ ] **Hybrid `magi show <id>`:** Seamlessly retrieves and formats deliberation reports from disk even when SpacetimeDB is offline or was purged.

---

### 🔹 0.2.3 — Stability, Resilience & Quorum Safeguards (Completed)
* [x] **Degraded Quorum Tolerance (2-of-3):** Deliberation continues when 1 node drops offline or hits quota limits, synthesizing a neutral position and proceeding with majority consensus.
* [x] **Smart Retries & Exponential Backoff:** Automatic retry loop for HTTP 429 rate limits, 5xx errors, and transport disconnects with `Retry-After` header inspection.
* [x] **Dynamic Schema Fallback:** Transparent recovery from HTTP 400 when endpoints lack `json_object` support.
* [x] **Payload Context Budgeting:** Intelligent 60/40 head-tail truncation protecting token limits against massive diffs.
* [x] **Canonical Root Directory Resolution:** Guarantees `deliberations/` is strictly saved at the repository root, eliminating nested directories.

---

### 🔹 0.2.2 — Smart Ingestion, Noise Filtering & Context Discovery (Completed)
* [x] **Automatic Project Context Discovery:** Deep recursive inspection of workspace manifests (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`).
* [x] **Smart Noise & Lockfile Filtering:** Automated filtering of lockfiles, minified assets, bundles, and build outputs from diff payloads.
* [x] **MAGI System Arbitrator in `lineamientos-pw`:** Formal integration of MAGI as the binding architectural and security veto arbitrator for AI-assisted development.

---

### 🔹 0.2.1 — Git Diff Auditing & Expanded MCP Suite (Completed)
* [x] **Git Diff Auditing (`magi diff`):** Audits uncommitted working tree changes, `--staged` index deltas, or branch comparisons.
* [x] **Architectural Dilemma Debates (`magi debate`):** Multi-agent trade-off debates on technology dilemmas and architectural choices.
* [x] **5 Specialized MCP Tools:** `deliberate_with_magi`, `audit_git_changes`, `triage_incident_with_magi`, `check_security_veto`, and `debate_technical_dilemma`.
