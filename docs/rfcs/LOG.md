# MAGI System — Engineering Log 

## 2026-09-30: Adoption of SpacetimeDB as In-Memory State Machine

**Context:** MAGI System requires real-time pub/sub synchronization between the CLI orchestrator and consensus storage, with deterministic, race-condition-free consensus evaluation when three concurrent LLM votes arrive.

**Options considered:**
1. Traditional relational database (PostgreSQL/SQLite) with custom WebSocket daemon.
2. Redis pub/sub with separate Python/Node orchestrator service.
3. SpacetimeDB as an integrated in-memory relational database and WASM execution runtime.

**Decision:** Adopted SpacetimeDB. Tabular state and transactional business logic (reducers) execute in the same memory space as WebAssembly. This allows `submit_node_vote` to evaluate the 3-node consensus and Balthasar veto atomically in the exact same transaction without distributed locking overhead.

**Consequences:**
- Reducers execute in a sandboxed WASM environment and cannot make arbitrary outbound HTTP network calls.
- The CLI client handles external LLM API calls and feeds the responses into SpacetimeDB reducers.
- Compiling the server module requires `wasm32-unknown-unknown` target.

---

## 2026-09-30: Decoupled Multi-Provider LLM Abstraction with Dynamic Parameters

**Context:** The three MAGI nodes represent distinct analytical perspectives (Melchior: logic/architecture, Balthasar: security/risk, Casper: pragmatism/DX). Hardcoding vendor APIs or models would prevent private or offline deployments.

**Decision:** Implemented `LlmProvider` trait and dynamic parameterization across Google Gemini, OpenAI, Anthropic, xAI Grok, DeepSeek, Ollama, and Mock.

**Consequences:**
- Enables offline local testing via Ollama or Mock provider.
- Eliminates hardcoded magic strings or static endpoints.
- Fully respects the zero-hardcoding principle across all components.

---

## 2026-09-30: Containerized Toolchain via Docker & WSL

**Context:** Development host environment runs Windows without native Rust or SpacetimeDB installed.

**Decision:** Selected Docker and WSL for reproducible compilation, execution, and local SpacetimeDB deployment.

**Consequences:**
- Zero host pollution on Windows.
- Consistent Linux toolchain with guaranteed `wasm32-unknown-unknown` support.

---

## 2026-09-30: Ephemeral Docker Lifecycle with Host-Bound State & Context Persistence

**Context:** The developer requires a workflow where Docker containers can be spun up on demand and destroyed immediately, without losing the SpacetimeDB database, deliberation history, or context payloads on the host PC.

**Decision:**
- Host directory bind-mounts for SpacetimeDB (`./.spacetimedb_data:/stdb`) and auto-persisting human-readable Markdown reports into the host repository (`./deliberations/deliberation_XXXXXX_<slug>.md`).
- Docker containers can be freely killed, stopped, or recreated; all database transactions and deliberated context remain intact on the developer's PC.

---

## 2026-09-30: MAGI Node Modularization (`melchior.rs`, `balthasar.rs`, `casper.rs`)

**Context:** The LLM integration layer previously retained generic provider file names (`anthropic.rs`, `openai.rs`, `ollama.rs`). In true Evangelion architecture, the codebase treats the Trinity nodes as first-class software modules (`melchior.rs`, `balthasar.rs`, `casper.rs`), where each node module encapsulates its unique analytical persona and criteria while remaining completely agnostic to whatever underlying LLM provider the user assigns.

**Decision:**
- Replaced provider-named files with dedicated node modules:
  - `client/src/llm/melchior.rs`: Melchior-1 (The Scientist)
  - `client/src/llm/balthasar.rs`: Balthasar-2 (The Mother, with VETO power)
  - `client/src/llm/casper.rs`: Casper-3 (The Woman)
  - `client/src/llm/mock.rs`: Test & simulation provider
- Centralized protocol negotiation (`dispatch_llm_request`) inside `client/src/llm/mod.rs`.

---

## 2026-10-01: Decoupling Personas to Markdown Skills (`client/skills/magi-system/`)

**Context:** System prompt strings for Melchior, Balthasar, and Casper were previously hardcoded in Rust source code constants inside `orchestrator.rs`. Modifying personas required recompiling the Rust binary.

**Decision:**
- Extracted all persona instructions into external Markdown files (`client/skills/magi-system/melchior.md`, `balthasar.md`, `casper.md`).
- Created `PromptLoader` in `client/src/skills/mod.rs` to load system prompts dynamically at runtime with fallback support.
- Added `--skill <path>` CLI flag to allow operators to inject custom team rules or coding guidelines at runtime.

---

## 2026-10-02: Documentation Overhaul & Roadmap Realignment

**Context:** Previous documentation contained legacy command references and an outdated roadmap with conflicting milestone priorities.

**Decision:**
- Reorganized `docs/` into modular categories: `architecture/`, `guides/`, and `rfcs/`.
- Created dedicated manuals: `GETTING_STARTED.md`, `CLI_AND_TUI.md`, `OPERATIONS.md`.
- Realigned `ROADMAP.md` around four high-impact local-developer milestones: v0.1.3 (Foundation), v0.2.0 (Streaming & Local DX), v0.3.0 (Codebase Awareness), and v1.0.0 (Standalone Distribution).
