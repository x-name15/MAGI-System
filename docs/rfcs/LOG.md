# Engineering Log

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

**Options considered:**
1. Single OpenAI client with different system prompts.
2. Hardcoded model names and static endpoints.
3. Trait-based `LlmProvider` abstraction supporting Anthropic, OpenAI, and Ollama, with fully dynamic environment variable overrides (`MELCHIOR_MODEL`, `BALTHASAR_MODEL`, `CASPER_MODEL`, endpoints, and timeouts).

**Decision:** Implemented `LlmProvider` trait and dynamic parameterization.

**Consequences:**
- Enables offline local testing via Ollama.
- Eliminates hardcoded magic strings or static endpoints.
- Fully respects the zero-hardcoding principle across all components.

---

## 2026-09-30: Containerized Toolchain via Docker & WSL

**Context:** Development host environment runs Windows without native Rust or SpacetimeDB installed.

**Options considered:**
1. Install Rust and SpacetimeDB directly into the Windows user profile.
2. Containerize the runtime and toolchain via `docker-compose` and WSL.

**Decision:** Selected Docker and WSL for reproducible compilation, execution, and local SpacetimeDB deployment.

**Consequences:**
- Zero host pollution on Windows.
- Consistent Linux toolchain with guaranteed `wasm32-unknown-unknown` support.

---

## 2026-09-30: Provider-Agnostic Backend & The Three Functional Workflows

**Context:** MAGI must not be tightly coupled to specific LLM companies or models. The database backend and consensus engine must remain completely agnostic, acting as the state and deliberation platform. Furthermore, the system must address three explicit operational use cases: (1) Idea Viability, (2) Code Maintenance under Guidelines, and (3) Targeted Error Triage.

**Options considered:**
1. Hardcoding vendor SDKs (Anthropic, OpenAI) directly into each node persona.
2. Building a provider-agnostic state machine in SpacetimeDB that accepts evaluations regardless of provider, with dedicated CLI subcommands for each of the three developer workflows.

**Decision:** Superseded by the mandatory Trinity incident protocol. SpacetimeDB records typed deliberations and uses specialist routing only for the opening analysis; the final incident verdict always requires all three MAGI nodes.

**Consequences:**
- Users can assign any model (GPT-4o, Gemini 1.5 Pro, Grok-2, Claude 3.5, DeepSeek, or local Ollama) to any module.
- The specialist opening analysis keeps routing focused while the mandatory second round preserves MAGI's collective judgment for every incident.

---

## 2026-09-30: Ephemeral Docker Lifecycle with Host-Bound State & Context Persistence

**Context:** The developer requires a workflow where Docker containers can be spun up on demand and destroyed immediately ("levantamos un docker y lo matamos y asi"), without losing the SpacetimeDB database, deliberation history, or context payloads on the host PC.

**Options considered:**
1. Default Docker named volumes (`spacetime_data`): State persists across restarts, but is stored invisibly inside Docker's internal VM / WSL storage, making it inaccessible directly on the Windows host and prone to being deleted on docker prune.
2. Host directory bind-mounts for both SpacetimeDB (`./.spacetimedb_data:/stdb`) and auto-persisting human-readable Markdown reports and input context into the host repository (`./deliberations/deliberation_XXXX_<slug>.md`).

**Decision:** Adopted Option 2.
- Docker containers can be freely killed, stopped, or recreated; all database transactions and deliberated context remain intact on the developer's PC.

---

## 2026-09-30: MAGI Node Modularization (`melchior.rs`, `balthasar.rs`, `casper.rs`)

**Context:** The LLM integration layer previously retained generic provider file names (`anthropic.rs`, `openai.rs`, `ollama.rs`). In true Evangelion architecture, the codebase must treat the Trinity nodes as first-class software modules (`melchior.rs`, `balthasar.rs`, `casper.rs`), where each node module encapsulates its unique analytical persona and criteria while remaining completely agnostic to whatever underlying LLM provider the user assigns.

**Decision:**
- Replaced provider-named files with dedicated node modules:
  - `client/src/llm/melchior.rs`: Melchior-1 (The Scientist)
  - `client/src/llm/balthasar.rs`: Balthasar-2 (The Mother, with VETO power)
  - `client/src/llm/casper.rs`: Casper-3 (The Woman)
  - `client/src/llm/mock.rs`: Test & simulation provider
- Centralized protocol negotiation (`dispatch_llm_request`) inside `client/src/llm/mod.rs` to support Anthropic, OpenAI-compatible (OpenAI, Gemini, Grok, DeepSeek), and Ollama local transparently.

---

## 2026-10-01: Focus on Standalone Local Workstation Application Scope

**Context:** The project roadmap previously included a milestone (0.3.0) for publishing reusable GitHub Actions and Git pre-commit hooks as external integrations. MAGI System is architected as an interactive, operator-facing local workstation CLI with a diegetic NERV terminal interface and SpacetimeDB state engine.

**Decision:**
- Pruned external CI/CD gating and pre-commit hook publishing from the roadmap.
- Streamlined the trajectory directly towards v0.2.0 (extended local provider ecosystem, offline fallback, and token streaming) and v1.0.0 (production stabilization and structured local report export).
- Removed manual test fixture bloat from the repository.


