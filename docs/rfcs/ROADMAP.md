# MAGI System — Project Roadmap 

This roadmap defines the engineering milestones for MAGI System, focusing strictly on its identity as a **high-reliability, local-first developer tool for terminal code auditing and architectural consensus**.

---

## Version Map

| Version | Status | Focus | Core Deliverables |
| :--- | :--- | :--- | :--- |
| **0.2.0** | **Current** | Modular Helpers, Deep Deliberation & MCP Integration | Helper separation (`core/helpers`, `llm/helpers`, `ui/helpers`), Model Context Protocol (`magi mcp` over stdio), pure local consensus engine, `--rounds N` deep debate, semantic exit codes (0/1/2/3), `--output json`, and 100% dynamic i18n localization. |
| **0.3.0** | **Next** | Streaming & Local-First DX | Diegetic token streaming in CRT monitors, 1-click offline Ollama profiles, and interactive TUI log scrolling. |
| **0.4.0** | **Planned** | Codebase Awareness | Git diff auditing (`--diff`, `--staged`), automatic ecosystem detection (`Cargo.toml`/`package.json`), and smart noise filtering. |
| **1.0.0** | **Target** | Production & Distribution | Standalone precompiled binaries (Windows/Linux/macOS), interactive HTML/JSON compliance reports, and multi-model benchmark suite. |

---

## 📦 Milestone Breakdown

### 🔹 0.2.0 — Modular Helpers, Deep Deliberation & MCP Server (Current)
* [x] **Model Context Protocol (MCP) Server:** Native `magi mcp` implementation over stdio exposing `deliberate_with_magi` for autonomous agent integration (Antigravity, Claude Desktop).
* [x] **Pure Local Consensus Engine:** Offline-capable consensus and security veto resolver with zero mandatory database dependencies.
* [x] **Deep Deliberation Rounds (`--rounds N`):** Configurable multi-round debate depth across all workflows with iterative peer exposure.
* [x] **Semantic Process Exit Codes:** Semantic exit codes (`0` for APPROVED, `1` for REJECTED/VETO, `2` for SPLIT, `3` for ERROR) for automated CI/CD gating.
* [x] **Machine-Readable JSON Output (`--output json`):** Structured JSON serialization envelope for programmatic consumption.
* [x] **Architectural Helper Decoupling:** Dedicated `helpers/` submodules across `core`, `llm`, and `ui`.
* [x] **Complete i18n String Externalization:** Zero hardcoded prompt strings or branching; 100% catalog-driven telemetry and prompts.

---

### 🔹 0.3.0 — Streaming & Local-First DX
* [ ] **Diegetic Token Streaming:** Real-time typewriter phosphor rendering inside NERV monitor boxes as each LLM node streams its tokens over SSE/WebSockets.
* [ ] **Zero-Config Local Profiles:** Pre-configured one-click templates for 100% offline local execution via Ollama (`qwen2.5-coder`, `deepseek-r1:8b`, `llama3.2`).
* [ ] **Interactive TUI Enhancements:** Live log scrolling and filterable deliberation history search in the Ratatui command deck.

---

### 🔹 0.4.0 — Codebase Awareness & Smart Ingestion
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
