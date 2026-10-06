# MAGI System — Architecture Documentation 

## 1. What is MAGI System?

MAGI System is an event-driven, distributed terminal code auditing and architectural review engine that submits source code and technical proposals to the parallel scrutiny of three distinct Large Language Model (LLM) analytical personas inspired by the supercomputer MAGI from *Neon Genesis Evangelion*. 

The system uses **SpacetimeDB** as an in-memory transactional database and WebAssembly (WASM) consensus state machine to record deliberation inputs, collect individual node votes with Common Weakness Enumeration (CWE) detection, and deterministically enforce voting rules, including an authoritative security veto.

MAGI operates as a **hybrid developer tool** with two complementary operational modes:
1. **One-Shot Scriptable CLI:** Standalone subcommands (`magi idea`, `magi maintain`, `magi triage`, or direct prompts) with diegetic NERV phosphor terminal monitors, real-time typewriter pacing, and deterministic exit codes.
2. **Interactive Full-Screen NERV TUI Console:** An alternate-screen terminal dashboard (`magi tui`) built on `ratatui` and `crossterm` featuring an interactive REPL, SpacetimeDB history/status queries, and natural-language intent classification.

---

## 2. What is this project NOT?

* **Not an auto-fixing or silent code refactor bot:** MAGI deliberates, audits, analyzes risk, and produces structured verdicts; it never modifies user source files without human intervention.
* **Not a GUI windowed desktop app or bloated IDE plugin:** It is engineered strictly for the developer's terminal.
* **Not a single-model prompt wrapper:** It is an asynchronous multi-agent consensus system requiring distinct analytical lenses executed in parallel across independent models.
* **Not a proprietary cloud SaaS:** All consensus state runs locally in SpacetimeDB with zero cloud database lock-in. The LLM layer is 100% open and vendor-agnostic: powered by standard OpenAI Chat Completions REST API, supporting any cloud endpoint, local model runner (Ollama, LM Studio, vLLM), or proxy gateway (LiteLLM, OpenRouter).

---

## 3. The Three Primary Use Cases

```text
┌─────────────────────────────────────────────────────────────────────────────┐
│                          MAGI SYSTEM ENTRY POINTS                           │
├──────────────────────────┬──────────────────────────┬───────────────────────┤
│ 1. IDEA VIABILITY        │ 2. CODE MAINTENANCE      │ 3. INCIDENT TRIAGE    │
│ `magi idea <file.md>`    │ `magi maintain <file>    │ `magi triage <log>    │
│                          │   --guidelines <rules>`  │   [--code <file>]`    │
│                          │                          │                       │
│ • Melchior: Feasibility  │ • Melchior: Clean Code   │ • Specialist Routing: │
│ • Balthasar: Sec & Veto  │ • Balthasar: Regressions │   Balthasar (Sec)     │
│ • Casper: Anti-bloat & DX│ • Casper: Team DX        │   Melchior (Logic)    │
│                          │                          │   Casper (Config)     │
│                          │                          │ • Mandatory Trinity   │
└──────────────────────────┴──────────────────────────┴───────────────────────┘
```

---

## 4. Core System Abstractions

### Server (SpacetimeDB WASM Module)
* **`Deliberation`**: Represents an audit session containing prompt instructions, author identity, context payload (code, RFC, or log), and lifecycle status (`PENDING`, `DEBATING`, `RESOLVED`, `FAILED`).
* **`NodeVote`**: Represents an atomic vote cast by a MAGI persona module (`Melchior-1`, `Balthasar-2`, or `Casper-3`), containing justification argument, detected CWE identifiers, stance (`APPROVE`, `REJECT`, `NEUTRAL`), risk rating (1 to 10), and execution latency.
* **`ConsensusResult`**: Immutable final verdict recording the tally of votes, summary of conclusions, timestamp, and resolved verdict classification.
* **Consensus Reducer (`eval_consensus`)**: Deterministic reducer function executed when all three nodes have cast their votes. It checks Balthasar's veto threshold before evaluating unanimous or majority rules.

### Client (CLI Orchestrator & Persona Modules)
* **Decoupled Persona Modules:**
  * `Melchior-1` (`melchior.rs` — The Scientist): Analyzes logic, architecture, and complexity.
  * `Balthasar-2` (`balthasar.rs` — The Mother): Analyzes cybersecurity, threat models, CWE detection, and holds unilateral VETO power (`risk_score >= 8` with `REJECT`).
  * `Casper-3` (`casper.rs` — The Woman): Analyzes DX, pragmatism, delivery reality, and anti-overengineering.
* **`PromptLoader` (`client/src/skills/`):** Decouples persona prompts to Markdown files (`client/skills/magi-system/`) and injects custom operator skills (`--skill`).
* **Universal Dispatcher (`client/src/llm/helpers/dispatch_helper.rs`):** Protocol-agnostic HTTP engine speaking standard OpenAI Chat Completions REST API, with prompt schemas externalized into i18n catalogs.
* **`MagiOrchestrator` (`client/src/core/orchestrator.rs`):** Orchestrates the 2-round deliberation lifecycle, specialist incident routing, latency tracking, and SpacetimeDB synchronization.
* **`DbClient` (`client/src/db/client.rs`):** Manages SpacetimeDB WebSocket/HTTP subscriptions and reducer invocations.
* **`NervTheme` (`client/src/ui/nerv_theme.rs`):** Diegetic CRT phosphor terminal theme rendering animations, monitors, and evaluation tables.
* **`Tui` (`client/src/ui/tui.rs`):** Interactive `ratatui` command console.

---

## 5. Deliberation Data Flow

```text
┌──────────────┐     CLI Invocation: magi idea docs/rfcs/ROADMAP.md
│ User / Shell │ ────────────────────────────────────────────────────────┐
└──────────────┘                                                         │
       ▲                                                                 │
       │ Terminal output (NERV TUI / CRT Monitors)                       ▼
┌──────────────────┐                                          ┌──────────────────────┐
│  Presentation    │                                          │  CLI Orchestrator    │
│  (NervTheme /    │                                          │  (main.rs / config)  │
│   Ratatui TUI)   │                                          └──────────┬───────────┘
└────────┬─────────┘                                                     │
         ▲                                                               │
         │ WebSocket Events (ConsensusResult)                            │ 1. create_deliberation
         │                                                               ▼
┌──────────────────┐                                          ┌──────────────────────┐
│  SpacetimeDB     │ ◄────────────────────────────────────────┤  SpacetimeDB Client  │
│  Engine (WASM)   │   2. submit_node_vote                    │  (WebSocket / HTTP)  │
└────────▲─────────┘                                          └──────────▲───────────┘
         │                                                               │
         │ 3. Evaluate Consensus (Atomic Reducer)                        │ 2-Round Trinity
         │    - Balthasar Veto Rule (REJECT + risk >= 8)                 │ Deliberation
         │    - Unanimous / Majority Tallies                             │
         │                                                               │
         │                                                    ┌──────────┴───────────┐
         │                                                    │ concurrent futures (x3)│
         │                                                    ├──────────────────────┤
         │                                                    │ Melchior-1 (Arch)    │
         │                                                    │ Balthasar-2 (Sec)    │
         │                                                    │ Casper-3 (DX)        │
         └────────────────────────────────────────────────────┴──────────────────────┘
```

---

## 6. Error Handling & Fail-Closed Guarantee

* All domain errors are strongly typed with `thiserror`.
* No `panic!` in production code paths; all fallible operations return `Result<T, MagiError>`.
* Reducer transactions in SpacetimeDB are atomic: any error rolls back state cleanly.
* If SpacetimeDB or upstream networks fail, MAGI reports `CONSENSUS_UNAVAILABLE` rather than fabricating unverified local consensus.

---

## 7. Anti-Bloat Architecture Principles

1. **Is it strictly required for multi-agent deliberation or consensus state?**
2. **Can it be solved using standard Rust primitives or existing crates?**
3. **Does it introduce unnecessary runtime overhead or break offline/local capabilities?**
