# MAGI System — Roadmap

## Version Map

* **0.1.3 (Current)** — Lean Docker toolchain, Rust 1.90/SpacetimeDB 1.12 compatibility, and clean workspace validation.
* **0.1.2** — Real event-driven NERV terminal UI, two-round Trinity debate, mandatory Trinity incident verdicts, and consensus integrity hardening.
* **0.1.1** — Consensus integrity hardening, concurrency-safe deliberation correlation, fail-closed vote persistence, and prompt-only universal deliberation.
* **0.1.0** — Core Architecture: SpacetimeDB server module, deterministic consensus engine with Balthasar security veto, asynchronous multi-provider CLI, and NERV terminal theme.
* **0.2.0** — Extended Provider Ecosystem: Local LLM fallback chains, Google Gemini direct adapter, and streaming argument parsing.
* **1.0.0** — Production Stabilization: Comprehensive local test harness, benchmarked latency profiling, and persistent audit report export.

---

## 0.1.0 — Core Architecture & Consensus Engine

**What gets built:**
- SpacetimeDB module in Rust (`cdylib` / `wasm32-unknown-unknown`) implementing tables `deliberation`, `node_vote`, and `consensus_result`.
- Reducers `create_deliberation`, `submit_node_vote`, and consensus evaluation with Balthasar's veto.
- Client CLI in Rust (`clap`) with asynchronous orchestrator (`tokio`, `reqwest`).
- Provider adapters for Anthropic (`Melchior-1`), OpenAI (`Balthasar-2`), and Ollama (`Casper-3`).
- NERV aesthetic terminal user interface with spinners and evaluation tables.
- Docker and WSL containerization harness for turnkey build and testing.

**Done when:**
Running `magi audit src/auth.rs --prompt "Review authentication handler"` connects to a running SpacetimeDB instance, queries all three LLM personas concurrently, registers the votes, triggers the consensus reducer, and renders the resolved NERV diagnostic summary in the terminal.

---

## 0.2.0 — Extended Provider Ecosystem & Fallback

**What gets built:**
- Google Gemini provider adapter.
- Automatic failover to local Ollama if remote APIs encounter rate limits or timeouts.
- Real-time token streaming visualization during node deliberation.

**Done when:**
Disconnecting external internet access gracefully falls back to local Ollama models without breaking consensus transactions.

---

## 1.0.0 — Production Stabilization & Local Audit Reports

**What gets built:**
- Comprehensive local test harness and integration stress tests.
- Benchmarked latency profiling across local and remote providers.
- Local audit report export (`--output-format json|markdown`) to save deliberations to disk.

**Done when:**
Operators can execute full autonomous audits offline and generate structured reports for local inspection.
