# MAGI System — Project Roadmap 

This roadmap defines the engineering milestones for MAGI System, focusing strictly on its identity as a **high-reliability, local-first developer tool for terminal code auditing and architectural consensus**.

---

## Version Map

| Version | Status | Focus | Core Deliverables |
| :--- | :--- | :--- | :--- |
| **0.1.4** | **Current** | Core Foundation & Hardening | In-memory SpacetimeDB engine, 2-round Trinity debate, i18n localization engine, zero-config provider inference, decoupled Markdown skills, diegetic NERV UI, and Ratatui TUI. |
| **0.2.0** | **Next** | Streaming & Local-First DX | Diegetic token streaming in CRT monitors, 1-click offline Ollama profiles, and configurable deep deliberation (`--rounds 3`). |
| **0.3.0** | **Planned** | Codebase Awareness | Git diff auditing (`--diff`, `--staged`), automatic ecosystem detection (`Cargo.toml`/`package.json`), and smart noise filtering. |
| **1.0.0** | **Target** | Production & Distribution | Standalone precompiled binaries (Windows/Linux/macOS), interactive HTML/JSON compliance reports, and multi-model benchmark suite. |

---

## 📦 Milestone Breakdown

### 🔹 0.1.4 — Core Architecture, i18n & Consensus Engine (Current)
* [x] **SpacetimeDB WASM Module:** In-memory relational tables (`deliberation`, `node_vote`, `consensus_result`) and atomic reducers.
* [x] **Two-Round Trinity Protocol:** Round 1 blind evaluation (eliminates anchoring bias) + Round 2 cross-peer debate.
* [x] **Unilateral Balthasar Security Veto:** Fail-closed security rule (`REJECT` with `risk >= 8`).
* [x] **Decoupled Persona Modules & Skills:** External Markdown personas in `client/skills/magi-system/` with runtime `--skill` injection.
* [x] **Universal Provider Engine:** First-class support for Google Gemini, OpenAI, Anthropic, xAI Grok, DeepSeek, Ollama, and offline Mock.
* [x] **Hybrid Terminal Interface:** Diegetic 24-bit TrueColor NERV CRT monitors + full-screen Ratatui interactive TUI console.
* [x] **Host-Bound Persistence:** Ephemeral Docker containers with zero host pollution; database persists to `.spacetimedb_data/` and reports to `deliberations/`.

---

### 🔹 0.2.0 — Streaming & Local-First DX
* [ ] **Diegetic Token Streaming:** Real-time typewriter phosphor rendering inside NERV monitor boxes as each LLM node streams its tokens over SSE/WebSockets.
* [ ] **Zero-Config Local Profiles:** Pre-configured one-click templates for 100% offline local execution via Ollama (`qwen2.5-coder`, `deepseek-r1:8b`, `llama3.2`).
* [ ] **Configurable Deep Deliberation:** Flag `--rounds <N>` (e.g. `--rounds 3` or `--deep`) for multi-stage architectural debriefs on mission-critical proposals.
* [ ] **Interactive TUI Enhancements:** Live log scrolling and filterable deliberation history search in the Ratatui command deck.

---

### 🔹 0.3.0 — Codebase Awareness & Smart Ingestion
* [ ] **Git Diff Auditing:** Native support for `magi audit --diff` or `magi maintain --staged` to evaluate branch deltas or staged changes without manual file copying.
* [ ] **Automatic Project Context Discovery:** Automatic parsing of root manifests (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`) to contextualize Balthasar and Casper assessments.
* [ ] **Smart Token & Noise Filtering:** Automated exclusion of lockfiles, minified bundles, and generated code artifacts before context serialization.
* [ ] **Multi-File Context Aggregation:** Support for auditing related modules concurrently (e.g. `magi maintain src/auth.rs --code src/user.rs`).

---

### 🔹 1.0.0 — Production Stabilization & Standalone Distribution
* [ ] **Standalone Native Binaries:** GitHub Releases with single-binary standalone distributions for Windows (`magi.exe`), Linux (`x86_64`), and macOS (`arm64`/`x86_64`).
* [ ] **Interactive HTML & JSON Report Export:** Single-file standalone HTML audit reports with visual NERV theme and structured JSON schemas for compliance archiving.
* [ ] **Multi-Model Consensus Benchmarking Suite:** Built-in benchmarking harness to profile latency, token costs, and consensus divergence across different model combinations.
* [ ] **Long-Term State Migration:** Automated migration tooling for SpacetimeDB database schemas across major upgrades.
