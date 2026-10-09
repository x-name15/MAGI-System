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
- Extracted all persona instructions into external Markdown files (`client/skills/magi-system/melchoir-1.md`, `balthasar-2.md`, `casper-3.md`).
- Created `PromptLoader` in `client/src/skills/mod.rs` to load system prompts dynamically at runtime with fallback support.
- Added `--skill <path>` CLI flag to allow operators to inject custom team rules or coding guidelines at runtime.

---

## 2026-10-07: Specialized MCP Suite & Git Diff Auditing (v0.2.1)

**Context:** AI coding assistants (Antigravity, Claude Desktop) need granular, specialized tools rather than a single generic entry point, and developers need to audit git diffs prior to commits.

**Decision:**
- Created dedicated subcommands `magi diff` (working tree, `--staged`, `--branch`) and `magi debate` (architectural dilemmas).
- Expanded MCP server over stdio to provide 5 purpose-built tools: `deliberate_with_magi`, `audit_git_changes`, `triage_incident_with_magi`, `check_security_veto`, and `debate_technical_dilemma`.

---

## 2026-10-07: Automatic Project Context Discovery & Noise Filtering (v0.2.2)

**Context:** Auditing git diffs and files without project context resulted in vague or generic LLM evaluations, while lockfiles (`Cargo.lock`, `package-lock.json`) exhausted LLM token budgets.

**Decision:**
- Implemented `discovery_helper` to inspect workspace manifests (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`) and inject ecosystem traits into prompts.
- Implemented `noise_filter_helper` to strip lockfiles, minified bundles, and generated artifacts from git diff payloads automatically.

---

## 2026-10-08: Degraded Quorum Resilience & Exponential Backoff Retries (v0.2.3)

**Context:** Transient network disconnects, HTTP 429 rate limits, and individual provider outages would cause entire multi-round deliberations to abort, frustrating developers.

**Decision:**
- Implemented **Degraded Quorum Tolerance (2-of-3)**: If 1 node fails, MAGI synthesizes an offline neutral position and continues debate with the remaining 2 nodes, resolving via majority consensus.
- Added smart exponential backoff with jitter and dynamic `Retry-After` header parsing.
- Added 60/40 head-tail context truncation safeguarding against LLM context window overflows.

---

## 2026-10-08: Dual-Archive Hybrid Loader & Interactive History Browser (v0.2.4)

**Context:** Developers required a convenient way to browse past deliberation records in the terminal, even when SpacetimeDB was stopped, wiped, or offline.

**Decision:**
- Created `history_loader` to merge records from both SpacetimeDB tables and local `./deliberations/*.md` reports.
- Built a full-screen interactive dual-pane TUI History Browser (`magi history`) using `ratatui` and `crossterm` with live search filtering (`/`) and modal Markdown report reading (`Enter`).
- Added non-interactive ANSI table rendering (`--table` or piped output) and hybrid `magi show <id>`.

---

## 2026-10-08: Hot-Standby Failover Brain & Truncation Recovery (v0.2.5)

**Context:** Primary models occasionally hit API credit exhaustion (HTTP 402) mid-session or suffered provider outages, while reasoning models truncated output tokens before closing JSON braces.

**Decision:**
- Implemented `MAGI_FALLBACK_MODEL` hot-standby failover circuit: automatically switches to a backup model without failing the deliberation.
- Configurable `MAGI_MAX_TOKENS` per node and globally.
- Implemented `json_repair_helper` to automatically recover and balance cut-off JSON braces, brackets, and quotes.
- Added native `magi purge` subcommand and eliminated test report leakage to host storage.

---

## 2026-10-09: OpenRouter Server Tools & Subagent Worker Ecosystem (v0.2.6)

**Context:** Frontier models spend unnecessary tokens and latency on micro-level sub-tasks, while OpenRouter introduced native server tools (`openrouter:subagent`).

**Decision:**
- Integrated native OpenRouter server tools allowing nodes to delegate micro-analytical tasks to faster worker models (`systems_analyst`, `security_scanner`, `pragmatic_evaluator`).
- Added anti-recursion protection, live console worker telemetry, and fully internationalized fallback conclusions across `client/i18n/es.json` and `en.json`.
