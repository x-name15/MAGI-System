# MAGI System — Project Roadmap 

This roadmap defines the engineering milestones for MAGI System, focusing strictly on its identity as a **high-reliability, local-first developer tool for terminal code auditing and architectural consensus**.

---

## Version Map

| Version | Status | Focus | Core Deliverables |
| :--- | :--- | :--- | :--- |
| **0.2.2** | **Current** | Smart Ingestion & Context Discovery | Automatic project context discovery (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`), smart noise/lockfile filtering in diffs, and integration with `lineamientos-pw`. |
| **0.2.1** | **Completed** | Git Diff Auditing, Dilemma Debates & Expanded MCP Suite | `magi diff` (working tree, staged, branch), `magi debate` (architectural dilemmas), and 5 specialized MCP tools (`deliberate_with_magi`, `audit_git_changes`, `triage_incident_with_magi`, `check_security_veto`, `debate_technical_dilemma`). |
| **0.2.0** | **Completed** | Modular Helpers, Deep Deliberation & MCP Integration | Helper separation (`core/helpers`, `llm/helpers`, `ui/helpers`), Model Context Protocol (`magi mcp` over stdio), pure local consensus engine, `--rounds N` deep debate, semantic exit codes (0/1/2/3), `--output json`, and 100% dynamic i18n localization. |
| **0.3.0** | **Next** | Interactive Reports & Visual Export | Standalone diegetic NERV HTML audit report export (`--export html`), interactive TUI history log browser, and token streaming. |
| **1.0.0** | **Target** | Production & Distribution | Standalone precompiled binaries (Windows/Linux/macOS), offline local Ollama profiles, and multi-model benchmark suite. |

---

## 📦 Milestone Breakdown

### 🔹 0.2.2 — Smart Ingestion, Noise Filtering & Context Discovery (Current)
* [x] **Automatic Project Context Discovery:** Deep recursive inspection of workspace manifests (`Cargo.toml` edition/workspaces, `package.json` TypeScript/frameworks, `go.mod`, `pyproject.toml`) automatically contextualizing Trinity evaluations.
* [x] **Smart Token & Noise Filtering:** Automated filtering of lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `go.sum`), minified assets, bundles, and build directories (`target/`, `dist/`, `node_modules/`) from diff payloads.
* [x] **CLI & MCP Context Ingestion:** Integrated discovery telemetry banners across `magi diff`, `maintain`, `audit`, and `debate`, plus markdown summary annotations in MCP `audit_git_changes`.
* [x] **MAGI System Arbitrator in `lineamientos-pw`:** Formal integration of MAGI as the binding architectural and security veto arbitrator for AI-assisted development.

---

### 🔹 0.2.1 — Git Diff Auditing, Dilemma Debates & Expanded MCP Suite (Completed)
* [x] **Git Diff Auditing (`magi diff`):** Audits uncommitted working tree changes, `--staged` index deltas, or branch comparisons without manual file copying.
* [x] **Architectural Dilemma Debates (`magi debate`):** Multi-agent trade-off debates on technology dilemmas and architectural decisions.
* [x] **Expanded MCP Tool Suite (5 Tools):** Exposed `audit_git_changes`, `triage_incident_with_magi`, `check_security_veto`, and `debate_technical_dilemma` alongside `deliberate_with_magi`.
* [x] **Security Veto Direct Evaluator:** Fast-path defensive analysis with Balthasar-2 for CWE detection and veto thresholds.

---

### 🔹 0.2.0 — Modular Helpers, Deep Deliberation & MCP Server (Completed)
* [x] **Model Context Protocol (MCP) Server:** Native `magi mcp` implementation over stdio exposing `deliberate_with_magi` for autonomous agent integration (Antigravity, Claude Desktop).
* [x] **Pure Local Consensus Engine:** Offline-capable consensus and security veto resolver with zero mandatory database dependencies.
* [x] **Deep Deliberation Rounds (`--rounds N`):** Configurable multi-round debate depth across all workflows with iterative peer exposure.
* [x] **Semantic Process Exit Codes:** Semantic exit codes (`0` for APPROVED, `1` for REJECTED/VETO, `2` for SPLIT, `3` for ERROR) for automated CI/CD gating.
* [x] **Machine-Readable JSON Output (`--output json`):** Structured JSON serialization envelope for programmatic consumption.
* [x] **Architectural Helper Decoupling:** Dedicated `helpers/` submodules across `core`, `llm`, and `ui`.
* [x] **Complete i18n String Externalization:** Zero hardcoded prompt strings or branching; 100% catalog-driven telemetry and prompts.

---

### 🔹 0.3.0 — Interactive Reports & Visual Export (Next)
* [ ] **Standalone Diegetic NERV HTML Reports (`--export html`):** Generates standalone interactive HTML compliance reports with CRT monitor visuals, risk meters, and full debate trajectories.
* [ ] **Diegetic Token Streaming:** Real-time typewriter phosphor rendering inside NERV monitor boxes as each LLM node streams its tokens over SSE/WebSockets.
* [ ] **Interactive TUI Enhancements:** Live log scrolling and filterable deliberation history search in the Ratatui command deck.

---

### 🔹 1.0.0 — Production Stabilization & Standalone Distribution
* [ ] **Standalone Native Binaries:** GitHub Releases with single-binary standalone distributions for Windows (`magi.exe`), Linux (`x86_64`), and macOS (`arm64`/`x86_64`).
* [ ] **Zero-Config Local Profiles:** Pre-configured one-click templates for 100% offline local execution via Ollama (`qwen2.5-coder`, `deepseek-r1:8b`, `llama3.2`).
* [ ] **Multi-Model Consensus Benchmarking Suite:** Built-in benchmarking harness to profile latency, token costs, and consensus divergence across different model combinations.
* [ ] **Long-Term State Migration:** Automated migration tooling for SpacetimeDB database schemas across major upgrades.
