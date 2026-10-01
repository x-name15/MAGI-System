# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.3] - 2026-10-01 — Evangelion Triangular Layout, Terminal Pacing, and TUI Refinements

### Added
- **Triangular Evangelion Layout:** Pixel-perfect NERV terminal presentation with Balthasar-2 positioned on top center and Casper-3 / Melchior-1 connected via horizontal data bus below.
- **Progressive Terminal Pacing:** Real-time typewriter pacing and deliberate 2.5s contemplative pause after animated consensus monitors before printing specialized findings.
- **Bilingual Deliberation Support:** Automatic language detection (Spanish/English) adapting node prompts, rationales, and NERV status headers dynamically.
- **In-Console TUI Command Deck:** System commands (`status`, `history`, `help`, `clear`) now render directly inside the Ratatui `NERV CONSOLE` buffer without screen flicker or exiting alternate screen.
- **Operator Review Prompt:** Interactive deliberations now pause after rendering the full Evangelion monitors with a clean operator prompt before returning to the TUI command deck.
- **Interactive Subcommand Aliases:** Added `tui` and `ui` aliases to `Interactive` command (`.\magi.ps1 tui`).
- **Multi-Platform Release Matrix:** GitHub Actions workflow (`release.yml`) compiling standalone binaries for Linux (`magi-linux-amd64.tar.gz`) and Windows (`magi-windows-amd64.zip`).
- **CodeQL Security Analysis:** Automated security scanning workflow (`codeql.yml`).

### Changed
- **Eliminated Duplicate Console Output:** Removed redundant R1/R2 status text spam and conflicting `indicatif` spinner collisions that caused ghost terminal lines.
- **Modularized Documentation:** Restructured documentation into dedicated directories: `docs/architecture/` (`ARCHITECTURE.md`, `TRINITY.md`), `docs/guides/` (`GETTING_STARTED.md`, `OPERATIONS.md`), and `docs/rfcs/` (`IDEA.md`, `LOG.md`, `ROADMAP.md`).
- **Streamlined Root README:** Compacted root `README.md` to ~110 lines with an exact ASCII diagram and explicit documentation of Hybrid (One-Shot CLI + Interactive TUI) architecture.
- **Roadmap Clarification:** Pruned external CI/CD action and pre-commit hook publishing from `ROADMAP.md` to focus on local workstation developer experience.
- **Removed Bloat:** Deleted dead static sample test fixtures (`fixtures/`).
- **Decoupled Persona Prompts:** Removed all legacy hardcoded `PROMPT_*` string constants from `orchestrator.rs`; the three node personas are now driven 100% by the externalized Markdown definitions (`client/skills/magi-system/`).
- **Toolchain Updates:** Updated dev image to Rust 1.90 slim Bookworm for SpacetimeDB 1.12 compatibility with container resource reservations.

## [0.1.2] - 2026-09-30 — Real NERV terminal UI

### Changed
- Replaced the blocking line-oriented console with an event-driven `ratatui` and `crossterm` interface.
- Renamed the interactive UI module to `client/src/ui/tui.rs` and aligned module references and documentation.
- Added keyboard-driven input, alternate-screen rendering, and a visible two-round Trinity status panel.

## [0.1.1] - 2026-09-30 — Consensus integrity hardening

### Fixed
- Made deliberation ID lookup safe for concurrent users by correlating records with a unique request ID.
- Enforced the three canonical MAGI node identities and valid vote/risk ranges in the server reducer.
- Kept triage escalation separate from final Trinity votes and propagated persistence failures.
- Allowed universal deliberation from a prompt without requiring a file path.
- Cancelled in-flight provider futures when the orchestration timeout expires.

## [0.1.0] - 2026-09-30 — Initial release: MAGI System

### Added
- Multi-agent consensus engine architecture based on the MAGI System from Neon Genesis Evangelion.
- SpacetimeDB server module with in-memory relational tables: `deliberation`, `node_vote`, and `consensus_result`.
- Deterministic transaction reducers: `create_deliberation`, `submit_node_vote`, and atomic consensus evaluation.
- Security veto logic for Balthasar-2: automatic veto override when risk score is greater than or equal to 8 with a reject vote.
- Asynchronous multi-provider CLI client supporting Melchior-1 (Anthropic), Balthasar-2 (OpenAI), and Casper-3 (Ollama/Gemini).
- Neon Genesis Evangelion NERV-themed terminal user interface with colored outputs, spinners, and structured evaluation summaries.
- Containerized development and runtime environment using Docker and SpacetimeDB.
