# MAGI System — Architecture Documentation 

## 1. Executive Summary

**MAGI System** is an event-driven, distributed terminal code auditing and architectural review engine that submits source code, technical proposals, and production incidents to the parallel scrutiny of three distinct Large Language Model (LLM) analytical personas inspired by the supercomputer MAGI from *Neon Genesis Evangelion*. 

The system uses **SpacetimeDB** as an in-memory transactional database and WebAssembly (WASM) consensus state machine to record deliberation inputs, collect individual node votes with Common Weakness Enumeration (CWE) detection, and deterministically enforce voting rules, including an authoritative security veto.

MAGI operates as a **hybrid developer tool** with complementary operational modes:
1. **One-Shot Scriptable CLI:** Standalone subcommands (`diff`, `debate`, `idea`, `maintain`, `triage`, `history`, `show`, `purge`, `status`) with diegetic NERV phosphor terminal monitors, typewriter pacing, and deterministic exit codes.
2. **Interactive Full-Screen NERV TUI Console:** An alternate-screen terminal dashboard (`magi tui`) built on `ratatui` and `crossterm` featuring an interactive REPL, SpacetimeDB history/status queries, and natural-language intent classification.
3. **Interactive Dual-Pane History Browser:** A standalone full-screen terminal explorer (`magi history`) with fuzzy filtering, live node evaluation badges, and a full-report modal viewer.
4. **Model Context Protocol (MCP) Server:** A native stdio server (`magi mcp`) exposing 5 specialized consensus tools to IDE agents (Antigravity, Claude Desktop, Cursor).

---

## 2. Architectural Boundaries (What MAGI Is NOT)

* **Not an auto-fixing or silent code refactor bot:** MAGI deliberates, audits, analyzes risk, and produces structured verdicts; it never modifies user source files without human intervention.
* **Not a GUI windowed desktop app or bloated IDE plugin:** It is engineered strictly for the developer's terminal.
* **Not a single-model prompt wrapper:** It is an asynchronous multi-agent consensus system requiring distinct analytical lenses executed in parallel across independent models.
* **Not a proprietary cloud SaaS:** All consensus state runs locally in SpacetimeDB with zero cloud database lock-in. The LLM layer is 100% vendor-agnostic: powered by standard OpenAI Chat Completions REST API, supporting any cloud endpoint, local model runner (Ollama, LM Studio, vLLM), or proxy gateway (LiteLLM, OpenRouter).

---

## 3. Core Entry Points & Operational Subcommands

```text
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                  MAGI SYSTEM ENTRY POINTS                                   │
├──────────────┬──────────────┬──────────────┬────────────────────────┬───────────────────────┤
│ 1. GIT DIFF  │ 2. DEBATE    │ 3. IDEA RFC  │ 4. CODE MAINTENANCE    │ 5. INCIDENT TRIAGE    │
│ `magi diff`  │ `magi debate`│ `magi idea`  │ `magi maintain`        │ `magi triage`         │
├──────────────┼──────────────┼──────────────┼────────────────────────┼───────────────────────┤
│ • Working    │ • Tech trade-│ • RFC viabi- │ • Enforces team rules  │ • Auto specialist     │
│   tree &     │   offs       │   lity       │ • Refactor review      │   routing             │
│   staged     │ • Dilemmas   │ • Feasibi-   │ • Adherence to         │ • Root cause & fix    │
│ • Lockfile   │ • Algorith-  │   lity       │   `lineamientos-pw`    │ • Mandatory Trinity   │
│   filtering  │   mic choice │ • Attack     │ • Technical debt       │   consensus           │
│ • Branch diff│ • Pragmatism │   surface    │   detection            │   debate              │
└──────────────┴──────────────┴──────────────┴────────────────────────┴───────────────────────┘
```

---

## 4. Core System Abstractions

### Server (SpacetimeDB WASM Module — `server/src/lib.rs`)
* **`Deliberation`**: Transactional record representing an audit session containing prompt instructions, author identity, context payload (code, RFC, diff, or log), and lifecycle status (`PENDING`, `DEBATING`, `RESOLVED`, `FAILED`).
* **`NodeVote`**: Atomic vote cast by a MAGI persona module (`Melchior-1`, `Balthasar-2`, or `Casper-3`), containing justification argument, detected CWE identifiers, stance (`APPROVE`, `REJECT`, `NEUTRAL`), risk rating (1 to 10), and execution latency.
* **`ConsensusResult`**: Immutable final verdict recording the tally of votes, summary of conclusions, timestamp, and resolved verdict classification (`APPROVED_UNANIMOUS`, `APPROVED_MAJORITY`, `REJECTED_UNANIMOUS`, `REJECTED_MAJORITY`, `VETO_BALTHASAR_SECURITY`, `SPLIT_DECISION`).
* **Atomic Consensus Reducer (`eval_consensus`)**: Deterministic reducer function executed when all active nodes have cast their votes. It validates Balthasar's veto threshold before evaluating unanimous, majority, or split rules.

### Client (CLI Orchestrator & Persona Modules)
* **Decoupled Persona Modules:**
  * `Melchior-1` (`client/src/llm/melchior.rs` — The Scientist): Analyzes logic, architecture, algorithmic complexity, and scalability.
  * `Balthasar-2` (`client/src/llm/balthasar.rs` — The Mother): Analyzes cybersecurity, threat models, CWE detection, and holds unilateral VETO power (`risk_score >= 8` with `REJECT`).
  * `Casper-3` (`client/src/llm/casper.rs` — The Woman): Analyzes DX, pragmatism, delivery reality, and anti-overengineering.
* **`PromptLoader` (`client/src/skills/`):** Decouples persona prompts to Markdown files (`client/skills/magi-system/`) and injects custom operator skills (`--skill`).
* **`MagiOrchestrator` (`client/src/core/orchestrator.rs`):** Orchestrates multi-round debates, degraded quorum tolerances, peer summary synthesis, specialist incident routing, latency tracking, and SpacetimeDB synchronization.
* **Universal Dispatcher (`client/src/llm/helpers/dispatch_helper.rs`):** Protocol-agnostic HTTP engine speaking standard OpenAI Chat Completions REST API, managing retries, payload budgeting, failover circuits, and server tools.
* **`DbClient` (`client/src/db/client.rs`):** Manages SpacetimeDB WebSocket/HTTP subscriptions, polling fallbacks, and reducer invocations.
* **Dual-Archive Hybrid History Loader (`client/src/ui/helpers/history_loader.rs`):** Merges records from SpacetimeDB tables with local Markdown reports (`deliberations/`), guaranteeing full history accessibility even when SpacetimeDB is offline or purged.
* **`NervTheme` (`client/src/ui/nerv_theme.rs`):** Diegetic CRT phosphor terminal theme rendering animations, monitors, and evaluation tables.
* **`Tui` (`client/src/ui/tui.rs`) & `HistoryBrowser` (`client/src/ui/history_browser.rs`):** Full-screen interactive terminal user interfaces powered by `ratatui` and `crossterm`.

---

## 5. Deliberation Data Flow

```text
┌──────────────┐     CLI Invocation: magi diff --staged / magi debate "..."
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
         │    - Unanimous / Majority / Split Tallies                     │
         │                                                               │
         │                                                    ┌──────────┴───────────┐
         │                                                    │ concurrent futures   │
         │                                                    ├──────────────────────┤
         │                                                    │ Melchior-1 (Arch)    │
         │                                                    │ Balthasar-2 (Sec)    │
         │                                                    │ Casper-3 (DX)        │
         └────────────────────────────────────────────────────┴──────────────────────┘
```

---

## 6. OpenRouter Server Tools & Subagent Worker Ecosystem

Starting in **v0.2.6**, Trinity nodes can delegate micro-level analytical sub-tasks to faster or free worker models mid-generation using OpenRouter's native server tools:

```text
[Melchior-1 (deepseek)] ──▶ OpenRouter (openrouter:subagent) ──▶ [Worker: systems_analyst (haiku/free)]
                                       │
                                       ▼ (Tool Outcome)
[Melchior-1] ◀─────────────── Integrated analytical data ◀────────────────┘
```

* **Persona Specialization:**
  * **Melchior-1** delegates to `systems_analyst`: Algorithmic structures, concurrency race conditions, and computational complexity ($O(N)$).
  * **Balthasar-2** delegates to `security_scanner`: Vulnerability attack surface, sanitization boundaries, and CWE threat catalogs.
  * **Casper-3** delegates to `pragmatic_evaluator`: Ergonomics, developer cognitive load, and deliverability trade-offs.
* **Anti-Recursion Safeguard:** The engine automatically detects and disables subagent tools if the configured worker model matches the node's primary model, preventing self-referential loops.
* **Live Telemetry:** The client logs tool execution notifications (`[{node_id}] Subagent worker invoked: {worker} (task: {task_name})`) in real time.
* **Web Search Tooling:** Nodes can leverage `openrouter:web_search` (`MAGI_WEB_SEARCH=true` or `MAGI_SUBAGENT_WEB_SEARCH=true`) to verify live library documentation and CVE announcements.

---

## 7. Resilience, Fault Tolerance & Failover Subsystems

### 1. Hot-Standby Failover Brain (`MAGI_FALLBACK_MODEL`)
When a node's primary model exhausts API credits (HTTP 402), encounters sustained 429 rate limits, or suffers upstream outages, the dispatcher transparently switches to the configured hot-standby fallback model (e.g. `cohere/north-mini-code:free` or local Ollama). The resulting vote is tagged as `<model> (backup)` in SpacetimeDB and Markdown reports.

### 2. Degraded Quorum Resilience (`MAGI_ALLOW_DEGRADED_QUORUM`)
If 1 of the 3 nodes drops offline or fails all retries, deliberation does not abort. MAGI synthesizes an explicit `[OFFLINE/DEGRADED QUORUM]` neutral position and continues multi-round debate with the remaining 2 active nodes, resolving via 2-of-3 majority. Deliberation strictly halts if 2 or more nodes fail.

### 3. Smart Retries with Exponential Backoff & Jitter
Transient HTTP errors (429 Rate Limits, 500, 502, 503, 504, connection resets) trigger automatic retries with exponential backoff (`delay * 2^attempt`). The dispatcher parses `Retry-After` headers and inspects JSON error payloads for quota exhaustion details.

### 4. Dynamic Schema Mode Fallback & JSON Repair
When endpoints reject strict JSON mode (`response_format: { type: "json_object" }`), the dispatcher automatically retries in freeform Markdown text mode. The `json_repair_helper` balances unclosed braces/brackets and missing quotes caused by token truncation.

### 5. Smart Payload Budgeting
Large context files and git diffs are safeguarded against LLM token overflow by a 60/40 head-tail preservation algorithm (`MAGI_MAX_CONTEXT_CHARS`, default: `60,000`).

### 6. Automatic Project Ecosystem Discovery
The discovery engine inspects workspace markers (`Cargo.toml`, `package.json`, `go.mod`, `pyproject.toml`) and injects detected traits and dependencies into system prompt headers.

---

## 8. Model Context Protocol (MCP) Server Suite

MAGI implements the **Model Context Protocol (MCP)** specification over `stdio` (JSON-RPC 2.0), providing 5 specialized tools for AI coding assistants:

| MCP Tool | Purpose | Key Parameters |
| :--- | :--- | :--- |
| `deliberate_with_magi` | Universal code & proposal deliberation | `context`, `prompt`, `guidelines`, `rounds`, `mock` |
| `audit_git_changes` | Audits git working tree, staged deltas, or branch comparisons | `staged`, `branch`, `rounds`, `mock` |
| `triage_incident_with_magi` | Triages errors/logs with lead specialist routing | `error_log`, `code_context`, `rounds`, `mock` |
| `check_security_veto` | Rapid Balthasar-2 security veto scan | `context`, `instructions`, `mock` |
| `debate_technical_dilemma` | Debates architectural dilemmas and technical trade-offs | `dilemma`, `context`, `rounds`, `mock` |
