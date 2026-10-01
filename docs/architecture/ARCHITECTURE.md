# MAGI System — Architecture Documentation

## 1. What is this project?

MAGI System is an event-driven, distributed terminal code auditing and architectural review engine that submits source code and technical proposals to the parallel scrutiny of three distinct Large Language Model (LLM) analytical personas inspired by the supercomputer MAGI from *Neon Genesis Evangelion*. The system uses SpacetimeDB as an in-memory transactional database and WebAssembly (WASM) consensus state machine to record deliberation inputs, collect individual node votes with Common Weakness Enumeration (CWE) detection, and deterministically enforce voting rules, including an authoritative security veto.

MAGI operates as a **hybrid developer tool** with two complementary operational modes:
1. **One-Shot Scriptable CLI:** Standalone subcommands (`magi idea`, `magi maintain`, `magi triage`, or free prompts) with diegetic NERV phosphor terminal monitors, real-time typewriter pacing, and exit codes.
2. **Interactive Full-Screen NERV TUI Console:** An alternate-screen terminal dashboard (`magi tui` / `magi console`) built on `ratatui` featuring an interactive REPL, SpacetimeDB history/status queries, and natural-language intent classification.

## 2. What is this project NOT?

* **Not an automatic code refactoring or auto-fixing bot:** MAGI System deliberates, audits, analyzes risk, and produces a structured verdict; it does not directly modify user source files without human intervention.
* **Not a GUI windowed desktop app or bloated IDE plugin:** It is engineered strictly for the developer's terminal—either as single-shot CLI commands or as a full-screen retro TUI console.
* **Not a single-model prompt wrapper:** It is an asynchronous consensus system requiring distinct analytical lenses (architecture, security, pragmatism) executed in parallel across independent models.
* **Not a locked proprietary cloud SaaS:** All consensus state runs self-hosted or locally in SpacetimeDB with zero cloud database lock-in. Meanwhile, the LLM intelligence layer is completely open and pluggable: users can import API keys from any cloud provider (OpenAI GPT, Google Gemini, Anthropic Claude, xAI Grok, DeepSeek) or run entirely offline with local models (Ollama).

## 3. The Three Primary Use Cases of MAGI

MAGI is designed to resolve three fundamental developer workflows:

### Case 1: Idea / Markdown Viability Review (`magi idea <file.md>`)
* **Scenario:** The user has an idea, RFC, or architecture proposal written in Markdown. They ask: *"Is this idea viable?"*
* **Deliberation:**
  * **Melchior-1:** Assesses theoretical soundness, system scalability, and architectural trade-offs.
  * **Balthasar-2:** Assesses attack surfaces, security threats, data privacy pitfalls, and triggers a veto if critical hazards exist.
  * **Casper-3:** Assesses practical implementation effort, time-to-market, compute costs, and overengineering.
* **Result:** Consensus verdict determining whether to proceed, revise, or abort.

### Case 2: Code Maintenance & Guidelines Adherence (`magi maintain <code.rs> --guidelines <rules.md>`)
* **Scenario:** The user must maintain or refactor existing code under strict team guidelines or standards. They ask: *"How should we maintain this code under these rules?"*
* **Deliberation:**
  * **Melchior-1:** Audits pattern adherence, technical debt, and modularity under the guidelines.
  * **Balthasar-2:** Audits regression hazards, security vulnerabilities, and backward compatibility.
  * **Casper-3:** Audits team DX friction, developer cognitive overhead, and unnecessary boilerplate.
* **Result:** Consensus verdict and maintenance strategy report.

### Case 3: Error / Incident Triage (`magi triage <error.log> [--code <context.rs>]`)
* **Scenario:** The user encounters a runtime error, stack trace, or incident.
* **Intelligent Specialist Routing:** MAGI classifies the nature of the error and assigns the opening analysis to the most qualified node:
  * Security / Auth / Permission / Secret leak $\rightarrow$ **Balthasar-2**
  * Architecture / Panics / Concurrency / Memory / Logic $\rightarrow$ **Melchior-1**
  * Environment / Tooling / Config / Dependencies $\rightarrow$ **Casper-3**
* **Mandatory Trinity Resolution:** The specialist provides the opening diagnosis, then all three nodes debate the incident and issue the final verdict.

## 4. Core Abstractions

### Server (SpacetimeDB WASM Module)
* **`Deliberation`**: Represents an audit session containing prompt instructions, author identity, context payload (code, Dockerfile, or specification), and lifecycle status (`PENDING`, `DEBATING`, `RESOLVED`, `FAILED`).
* **`NodeVote`**: Represents an atomic vote cast by an individual MAGI persona module (`Melchior-1`, `Balthasar-2`, or `Casper-3`), containing justification argument, detected CWE identifiers, stance (`APPROVE`, `REJECT`, `NEUTRAL`), risk rating (1 to 10), and execution duration.
* **`ConsensusResult`**: Immutable final verdict recording the tally of votes, summary of conclusions, timestamp, and resolved verdict classification.
* **Consensus Engine (`evaluate_consensus`)**: Deterministic reducer function executed when all three nodes have cast their votes. It checks Balthasar's veto threshold before evaluating unanimous or majority rules.

### Client (CLI Orchestrator & Persona Modules)
* **The Persona Modules (Decoupled from specific models):**
  * `Melchior-1` (Scientist — Logic & Architecture): Configured via `MELCHIOR_API_KEY`, `MELCHIOR_MODEL`, and `MELCHIOR_ENDPOINT`. Enforces software design, cyclomatic complexity, performance, and clean patterns. Can be driven by any model (e.g. Claude 3.5, GPT-4o, DeepSeek, Gemini).
  * `Balthasar-2` (Mother — Cybersecurity & Risk): Configured via `BALTHASAR_API_KEY`, `BALTHASAR_MODEL`, and `BALTHASAR_ENDPOINT`. Enforces threat modeling, OWASP Top 10 vulnerabilities, CWE detection, and possesses unilateral VETO authority (`risk_score >= 8` with `REJECT`). Can be driven by any model (e.g. Gemini 1.5 Pro, GPT-4o, Claude 3.5).
  * `Casper-3` (Person — Pragmatism & DX): Configured via `CASPER_API_KEY`, `CASPER_MODEL`, and `CASPER_ENDPOINT`. Enforces implementation simplicity, developer experience, compute cost, and team velocity. Can be driven by any model (e.g. Grok-2, Llama 3 via Ollama, Mistral).
* **`LlmProvider` (Trait)**: Unified asynchronous interface defining `evaluate(node_id, system_prompt, user_prompt, context_payload) -> Result<NodeEvaluation, MagiError>`.
* **MAGI Node Modules (`client/src/llm/`)**:
  * `MelchiorNode` (`melchior.rs`): Encapsulates Melchior-1 persona, logic/architecture criteria, and provider dispatch.
  * `BalthasarNode` (`balthasar.rs`): Encapsulates Balthasar-2 persona, cybersecurity/vulnerability scanning, risk scoring, and unilateral VETO criteria.
  * `CasperNode` (`casper.rs`): Encapsulates Casper-3 persona, DX, maintainability, and delivery practicality.
  * `MockProvider` (`mock.rs`): Offline simulation provider for headless CI/CD and testing.
  * Universal Dispatcher (`dispatch_llm_request`): Dispatches requests to Anthropic Claude, OpenAI-compatible APIs (OpenAI, Gemini, xAI Grok, DeepSeek), or Ollama local inference.
* **`MagiOrchestrator`**: Coordinates concurrent cancellable provider futures, collects responses, records execution latency, and interfaces with the SpacetimeDB connection.
* **`DbClient`**: Manages the SpacetimeDB connection, handles subscriptions to tables, dispatches reducers, and synchronizes real-time updates.
* **`NervTheme`**: Terminal presentation layer rendering stylized NERV computer diagnostics, progress indicators, tables, and colored alerts.
* **`Tui`**: Event-driven `ratatui`/`crossterm` command deck for keyboard input, status rendering, and interactive MAGI sessions.

## 4. Data Flow

```text
┌──────────────┐     CLI Invocation: magi audit <file> --prompt <text>
│ User / Shell │ ────────────────────────────────────────────────────────┐
└──────────────┘                                                         │
       ▲                                                                 │
       │ Terminal output (NERV TUI)                                      ▼
┌──────────────────┐                                          ┌──────────────────────┐
│  Presentation    │                                          │  CLI Orchestrator    │
│  (comfy-table /  │                                          │  (main.rs / config)  │
│   indicatif)     │                                          └──────────┬───────────┘
└────────┬─────────┘                                                     │
         ▲                                                               │
         │ WebSocket Events (ConsensusResult)                            │ 1. create_deliberation
         │                                                               ▼
┌──────────────────┐                                          ┌──────────────────────┐
│  SpacetimeDB     │ ◄────────────────────────────────────────┤  SpacetimeDB Client  │
│  Engine (WASM)   │   2. submit_node_vote                    │  (WebSocket SDK)     │
└────────▲─────────┘                                          └──────────▲───────────┘
         │                                                               │
         │ 3. Evaluate Consensus (Atomic Reducer)                        │ Concurrent
         │    - Balthasar Veto Rule (REJECT + risk >= 8)                 │ LLM evaluation
         │    - Unanimous / Majority Tallies                             │
         │                                                               │
         │                                                    ┌──────────┴───────────┐
        │                                                    │ concurrent futures (x3)│
         │                                                    ├──────────────────────┤
         │                                                    │ Melchior-1 (Claude)  │
         │                                                    │ Balthasar-2 (GPT-4o) │
         │                                                    │ Casper-3 (Ollama)    │
         │                                                    └──────────────────────┘
```

1. **Submission**: The user runs `magi audit <path> --prompt "..."`. The CLI reads the target file and connects to SpacetimeDB.
2. **Deliberation Creation**: The client calls reducer `create_deliberation`, persisting the request with status `PENDING`.
3. **Opening Round**: The orchestrator concurrently dispatches the audit payload to Melchior-1, Balthasar-2, and Casper-3.
4. **Debate Round**: Each node receives the first-round positions, challenges weak reasoning, and emits a final position.
5. **Atomic Consensus**: The client submits the three final positions; inside SpacetimeDB, the third final vote calculates the outcome, writes `consensus_result`, and marks `deliberation.status = "RESOLVED"`.
6. **Live Rendering**: The CLI receives the WebSocket update for `consensus_result` and renders the NERV diagnostic interface.

## 5. Error Handling Philosophy

* All errors must be strongly typed using `thiserror` in both the server and client crates.
* No `panic!` in production code paths; all fallible operations return `Result<T, MagiError>`.
* Reducer transactions in SpacetimeDB are atomic: any `Err` returned from a reducer cleanly rolls back all modifications in that transaction.
* Client network and LLM provider failures are trapped per node with clear diagnostics, preventing a single API outage from silently crashing the CLI.
* Configurable timeouts ensure the orchestrator never hangs indefinitely awaiting an unresponsive upstream provider.

## 6. V1 Scope

### In V1:
* Local SpacetimeDB server module in Rust compiling to WASM.
* Tables: `deliberation`, `node_vote`, `consensus_result`.
* Reducers: `create_deliberation`, `submit_node_vote`, `evaluate_consensus`.
* Balthasar security veto mechanism (`risk_score >= 8` with `REJECT`).
* Asynchronous client CLI supporting Anthropic, OpenAI, and Ollama providers.
* Configurable parameters (endpoints, keys, timeouts, models) via flags and environment variables.
* NERV terminal theme with progress spinners, colored badges, and evaluation summaries.
* Complete Docker and WSL support for containerized execution.

### Out of Scope for V1:
* Web-based browser frontend dashboard (planned for V2).
* Multi-user role-based authentication or organizational tenancy.
* Automatic pull request commenting bot for GitHub/GitLab (planned for V1.1).

## 7. Anti-Bloat Rule

Before adding any crate dependency, feature flag, or abstraction to MAGI System:

1. **Is it strictly required for multi-agent deliberation or SpacetimeDB state consensus?**
2. **Can it be solved using standard Rust primitives or existing dependencies?**
3. **Does it introduce unnecessary runtime overhead or break offline/local capabilities?**

If the answer to (1) is no or (3) is yes, the proposal is rejected.
