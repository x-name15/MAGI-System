# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.3] - 2026-10-08 — Degraded Quorum Resilience, Exponential Backoff Retries & Ingestion Safeguards

### Added
- **Fault-Tolerant Degraded Quorum (`core::orchestrator::deliberate_trinity`):**
  - Added support for 2-of-3 majority consensus even when 1 node drops offline, encounters transport disconnects, or exhausts API credit/rate limits.
  - Automatically synthesizes an explicit `[OFFLINE/DEGRADED QUORUM]` neutral position, proceeding across multi-round debates with remaining active nodes and issuing clear terminal diagnostic warnings without aborting deliberation.
  - Controlled by `MAGI_ALLOW_DEGRADED_QUORUM` (default `true`). Deliberation strictly fails if 2 or more nodes are unreachable to protect consensus integrity.
- **Smart Retries with Exponential Backoff & Jitter (`llm::helpers::dispatch_helper`):**
  - Integrated automatic retry loop with exponential backoff (`delay * 2^attempt`) across transient HTTP errors (429 Rate Limits, 500, 502, 503, 504, connection resets).
  - Dynamically parses `Retry-After` headers and extracts human-readable diagnostic messages from structured JSON error responses (e.g. OpenRouter daily quota exhaustion).
  - Configurable via `MAGI_MAX_RETRIES` (default `3`) and `MAGI_RETRY_DELAY_MS` (default `1000`).
- **Dynamic Schema Mode Fallback (`llm::helpers::dispatch_helper`):**
  - Transparently recovers from HTTP 400 schema incompatibility errors when models do not support `response_format: json_object`, instantly re-dispatching in freeform Markdown text mode coupled with JSON repair.
- **Smart Payload Budgeting & Context Safeguards (`llm::helpers::dispatch_helper`):**
  - Safeguards LLM context limits against massive diffs or error logs with smart 60/40 head-tail preservation (`MAGI_MAX_CONTEXT_CHARS`, default `60000`).
- **MCP Markdown Deliberation Archiving (`mcp::handler`):**
  - Ensured all MCP tool deliberations automatically generate timestamped Markdown audit reports in `deliberations/` and persist records in SpacetimeDB history.

---

## [0.2.2] - 2026-10-07 — Smart Ingestion, Noise Filtering & Automatic Context Discovery

### Added
- **Automatic Project Context Discovery (`core::helpers::discovery_helper`):**
  - Deep recursive inspection of target workspaces detecting root ecosystem manifests: Rust (`Cargo.toml` edition, packages, workspaces), Node/TypeScript (`package.json`, frameworks like Next.js, React, Express, NestJS), Go (`go.mod`), Python (`pyproject.toml`), and Docker.
  - Automatically enriches Trinity prompts with discovered project context, ensuring Melchior, Balthasar, and Casper deliberate with full awareness of runtime idioms, memory safety guarantees, and framework constraints.
- **Smart Diff Noise & Lockfile Filtering (`core::helpers::noise_filter_helper`):**
  - Automated exclusion of high-noise artifacts from git diff payloads, including package lockfiles (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, `yarn.lock`, `go.sum`), minified bundles (`*.min.js`, `*.min.css`), source maps (`*.map`), binaries, and generated directories (`dist/`, `target/`, `node_modules/`).
  - Reports excluded file counts and line savings (`MAGI SMART INGESTION`) on terminal and preserves clean diff payloads for LLM context limits.
- **CLI & MCP Telemetry Telemetry Integration:**
  - Added `MAGI DISCOVERY` telemetry banner across `magi diff`, `magi maintain`, `magi audit`, and `magi debate`.
  - Enriched MCP `audit_git_changes` markdown reports with project context and smart ingestion exclusion notices.
- **Expanded Test Suite:** Added unit tests for manifest parsing, project detection, and noise filter verification, bringing total tests to 44 passing tests (37 client + 7 server) with 0 clippy warnings.

---

## [0.2.1] - 2026-10-06 — Git Diff Auditing, Architectural Dilemma Debates & Expanded MCP Tool Suite

### Added
- **Git Diff Auditing Subcommand (`magi diff`):** Audits uncommitted working tree changes, `--staged` index deltas, or branch diffs (`--branch origin/main`) directly from git with zero manual file copying required. Deliberates changes across logic regressions, security leaks, and cognitive overhead.
- **Architectural Dilemma Debate Subcommand (`magi debate`):** Direct CLI command to debate technical decisions, library choices, or architectural trade-offs (e.g. `magi debate "WebSockets vs SSE for real-time notifications"`), returning multi-round cross-examination and synthesized consensus recommendations.
- **Expanded Model Context Protocol (MCP) Suite (5 Tools):** Upgraded `McpHandler` to register and serve 5 specialized tools over stdio:
  - `deliberate_with_magi`: General code and proposal deliberation.
  - `audit_git_changes`: Inspects repository git diff (working tree, staged, or target branch).
  - `triage_incident_with_magi`: Triages errors/panics with lead specialist routing followed by Trinity debate.
  - `check_security_veto`: Fast-path defensive security review with Balthasar-2 for CWE detection and veto thresholds.
  - `debate_technical_dilemma`: Multi-agent trade-off debate on architectural decisions.
- **Core Helpers & Orchestrator Expansion:**
  - Added `core::helpers::git_helper` for executing and capturing UTF-8 git diff streams.
  - Added `MagiOrchestrator::evaluate_security_veto` for single-node defensive checks.
  - Added `MagiOrchestrator::deliberate_debate` for architectural dilemma evaluation.
  - Added `MagiError::Internal` variant for clean internal error propagation.
- **Expanded Test Coverage:** Added unit tests for git diff inspection, CLI argument parsing for `diff` and `debate`, and MCP mock calls for the new tools, bringing the test suite to 36 passing tests (29 client + 7 server) with 0 clippy warnings.

---

## [0.2.0] - 2026-10-06 — Architectural Helper Decoupling, Pure Local Consensus & Model Context Protocol (MCP) Server

### Added
- **Model Context Protocol (MCP) Server (`magi mcp`):** Native MCP server implementation over `stdio` conforming to the 2024-11-05 JSON-RPC specification. Exposes `deliberate_with_magi` tool allowing AI coding agents (such as Antigravity and Claude Desktop) to invoke the Evangelion Trinity consensus engine directly within their tool execution environment.
- **Pure Local Consensus Engine (`core::helpers::consensus_helper`):** Standalone consensus and security veto resolver supporting offline deliberation, instant evaluation, and i18n summary resolution without requiring an active SpacetimeDB connection.
- **Deep Deliberation Rounds (`--rounds N`):** Configurable multi-round debate depth across all CLI workflows (`idea`, `maintain`, `triage`, `audit`). Intermediate rounds (2 to $N-1$) iteratively expose previous peer positions and risk flags, culminating in a definitive final verdict round.
- **Semantic Process Exit Codes:** CLI subcommands terminate with standard exit codes (`0` for APPROVED, `1` for REJECTED / VETO, `2` for SPLIT / NEUTRAL, `3` for ERROR), enabling seamless integration into automated CI/CD and autonomous quality loops.
- **Machine-Readable JSON Output (`--output json`):** Added structured JSON serialization via `ui::output::JsonOutput` providing machine-ingestible envelopes with deliberation ID, category, verdict summary, debate rounds count, and per-node metrics.
- **Explicit Language Configuration (`--lang`):** Added global `--lang <en|es>` CLI flag and `MAGI_LANG` / `LANG` environment resolution.
- **Architectural Helpers Decoupling:** Modularized internal logic across dedicated submodules:
  - `core::helpers`: `debate_helper`, `triage_helper`, `consensus_helper`.
  - `llm::helpers`: `dispatch_helper`, `json_repair_helper`, `parser_helper`.
  - `ui::helpers`: `layout_helper` (CJK-aware display width and triangular screen wireframe), `monitor_helper` (diegetic CRT monitor formatters).
  - `mcp`: `protocol`, `handler`, `stdio` event loop.
- **Docker Volume Isolation & Zero Host Pollution:** Migrated SpacetimeDB data storage and Cargo build caches to dedicated Docker named volumes (`spacetimedb_data:/stdb`, `cargo_target:/workspace/target`, `cargo_cache`, `cargo_git`), completely eliminating host disk pollution and intermediate target directories. Set `user: root` for SpacetimeDB to resolve Linux UID file permission conflicts (`os error 13`).
- **OpenRouter & Universal OpenAI-Compatible Multi-Model Deliberation:** Full native support for OpenRouter (`https://openrouter.ai/api/v1`), including smart `OPENROUTER_API_KEY` auto-detection and high-capacity free tier routers (`nvidia/nemotron-3-super-120b-a12b:free`, `cohere/north-mini-code:free`, `openrouter/free`). Added configurable HTTP timeout (`MAGI_TIMEOUT_SECONDS`, default 120s).
- **Multi-Tier Robust JSON Parser & Automatic Repair:** Enhanced `json_repair_helper` to recover unquoted keys, clean leading structural punctuation, and handle dot-prefixed keys (`.vote`, `.risk_score`, etc.). Added Serde deserialization aliases across `RawNodeOutput` and dynamic `serde_json::Value` fallback extraction to prevent crashes on non-standard model schema deviations.
- **Unified English Documentation:** Standardized all repository documentation (`docs/`, `README.md`, `CHANGELOG.md`, `CONFIGURATION.md`) in English per project development guidelines.
- **Comprehensive Test Suite Expansion:** Added unit tests for MCP initialization, tool listing, tool invocation, local consensus evaluation, agnostic environment resolution, and JSON parser repair, expanding test coverage to 30 passing tests (23 client + 7 server) with 0 clippy warnings.

### Changed
- **Provider-Agnostic Configuration:** Removed vendor bias, hardcoded priority chains, and dated version strings in `config.rs`. Added generic environment variable support (`MAGI_PROVIDER`, `LLM_PROVIDER`, `LLM_MODEL`, `LLM_ENDPOINT`, `LLM_API_KEY`) alongside per-node overrides.
- **Neutral Default Configuration:** `Default for MagiConfig` now initializes a neutral, offline-safe mock environment rather than hardcoding cloud vendor models.
- **Zero Hardcoded Stopwords / Strings:** Completely eliminated `is_spanish_text` and substring keyword heuristics in favor of clean catalog-driven internationalization (`client/i18n/`).

---

## [0.1.4] - 2026-10-02 — i18n Localization Engine, Zero-Config Provider Inference & Documentation Overhaul

### Added
- **i18n Localization Engine (`client/src/i18n/`):** Dedicated message catalogs (`client/i18n/en.json` and `client/i18n/es.json`) providing runtime internationalization for terminal telemetry, debate prompts, and Markdown reports.
- **Language Override Configuration (`MAGI_LANG`):** Dynamic language detection from prompt contents with optional explicit override via `MAGI_LANG=es|en`.
- **Language-Aware Markdown Reports:** Rewrote `report.rs` to generate 100% unified, consistent Markdown reports in either Spanish or English in `deliberations/` without language mixing.
- **Zero-Config Intelligent Provider Inference:** `MagiConfig::from_env()` automatically detects and configures all 3 nodes based on whichever single API key is present in `.env` (Gemini, OpenAI, Anthropic, Grok, DeepSeek, Ollama, or Mock), removing rigid per-node fallbacks.
- **Automated Manifest Synchronization:** Release workflow (`release.yml`) automatically extracts version from `CHANGELOG.md`, synchronizes `client/Cargo.toml` and `server/Cargo.toml` across Linux and Windows build matrices, and commits the version bump on release.
- **Dedicated CLI & TUI Manual:** Added comprehensive user guide in `docs/guides/CLI_AND_TUI.md`.

### Changed
- **Clap Dynamic Versioning:** Removed hardcoded `version = "0.1.3"` in `client/src/main.rs` in favor of automatic compile-time derivation from `client/Cargo.toml`.
- **Docker Toolchain & Compose Security Hardening:** Patched Debian OS CVEs via `apt-get upgrade -y` in `Dockerfile.dev` and removed unnecessary source workspace volume mounts from `spacetimedb` service in `docker-compose.yml`.
- **Entrypoint Tool Passthrough:** Enhanced `scripts/entrypoint.sh` to allow direct execution of developer commands (`cargo`, `rustc`, `git`, `bash`, `sh`).
- **Complete Documentation Suite Overhaul:** Restructured `docs/` into modular categories (`architecture/`, `guides/`, `rfcs/`), aligned all command examples, and realigned `ROADMAP.md` around realistic v0.1.4–v1.0.0 deliverables.

---

## [0.1.3] - 2026-10-01 — Evangelion Triangular Layout, Terminal Pacing & Decoupled Persona Prompts

### Added
- **Triangular Evangelion Layout:** Pixel-perfect NERV terminal presentation with Balthasar-2 positioned on top center and Casper-3 / Melchior-1 connected via horizontal data bus below.
- **Progressive Terminal Pacing:** Real-time typewriter pacing and deliberate 2.5s contemplative pause after animated consensus monitors before printing specialized findings.
- **In-Console TUI Command Deck:** System commands (`status`, `history`, `help`, `clear`) now render directly inside the Ratatui `NERV CONSOLE` buffer without screen flicker or exiting alternate screen.
- **Operator Review Prompt:** Interactive deliberations now pause after rendering the full Evangelion monitors with a clean operator prompt before returning to the TUI command deck.
- **Multi-Platform Release Matrix:** GitHub Actions workflow (`release.yml`) compiling standalone binaries for Linux (`magi-linux-amd64.tar.gz`) and Windows (`magi-windows-amd64.zip`).
- **CodeQL Security Analysis:** Automated security scanning workflow (`codeql.yml`).

### Changed
- **Decoupled Persona Prompts:** Removed all legacy hardcoded `PROMPT_*` string constants from `orchestrator.rs`; the three node personas are now driven 100% by the externalized Markdown definitions (`client/skills/magi-system/`).
- **Toolchain Updates:** Updated dev image to Rust 1.90 slim Bookworm for SpacetimeDB 1.12 compatibility with container resource reservations.
- **Eliminated Duplicate Console Output:** Removed redundant R1/R2 status text spam and conflicting `indicatif` spinner collisions that caused ghost terminal lines.

---

## [0.1.2] - 2026-09-30 — Real NERV terminal UI

### Changed
- Replaced the blocking line-oriented console with an event-driven `ratatui` and `crossterm` interface.
- Renamed the interactive UI module to `client/src/ui/tui.rs` and aligned module references and documentation.
- Added keyboard-driven input, alternate-screen rendering, and a visible two-round Trinity status panel.

---

## [0.1.1] - 2026-09-30 — Consensus integrity hardening

### Fixed
- Made deliberation ID lookup safe for concurrent users by correlating records with a unique request ID.
- Enforced the three canonical MAGI node identities and valid vote/risk ranges in the server reducer.
- Kept triage escalation separate from final Trinity votes and propagated persistence failures.
- Allowed universal deliberation from a prompt without requiring a file path.
- Cancelled in-flight provider futures when the orchestration timeout expires.

---

## [0.1.0] - 2026-09-30 — Initial release: MAGI System

### Added
- Multi-agent consensus engine architecture based on the MAGI System from Neon Genesis Evangelion.
- SpacetimeDB server module with in-memory relational tables: `deliberation`, `node_vote`, and `consensus_result`.
- Deterministic transaction reducers: `create_deliberation`, `submit_node_vote`, and atomic consensus evaluation.
- Security veto logic for Balthasar-2: automatic veto override when risk score is greater than or equal to 8 with a reject vote.
- Asynchronous multi-provider CLI client supporting Melchior-1 (Anthropic), Balthasar-2 (OpenAI), and Casper-3 (Ollama/Gemini).
- Neon Genesis Evangelion NERV-themed terminal user interface with colored outputs, spinners, and structured evaluation summaries.
- Containerized development and runtime environment using Docker and SpacetimeDB.
