# The Evangelion Trinity & Consensus Protocol 

In *Neon Genesis Evangelion*, the MAGI supercomputer governs decision-making through three biological computers embodying the three distinct facets of its creator, Dr. Naoko Akagi. In **MAGI System**, this cybernetic philosophy is translated into a deterministic, high-reliability software architecture.

```text
                     ┌───────────────────────┐
                     │      BALTHASAR-2      │
                     │   MOTHER · SECURITY   │
                     │RISK [███░░░░░░░]  3/10│
                     │ R2:  95%  LAT: 2500ms │
                     ├───────────────────────┤
                     │    ██ AGREEMENT ██    │
                     │     [  AGREEMENT  ]   │
                     └───────────┬───────────┘
                                 │
                ┌────────────────┴────────────────┐
                │                                 │
    ┌───────────┴───────────┐         ┌───────────┴───────────┐
    │       CASPER-3        │         │      MELCHIOR-1       │
    │  WOMAN · PRAGMATICS   │         │   SCIENTIST · ARCH    │
    │RISK [█████████░]  9/10│         │RISK [███░░░░░░░]  3/10│
    │ R2: 100%  LAT: 3200ms │         │ R2:  95%  LAT: 2700ms │
    ├───────────────────────┼─────────┼───────────────────────┤
    │     ██ DENIAL ██      │         │    ██ AGREEMENT ██    │
    │     [    DENIAL   ]   │         │     [  AGREEMENT  ]   │
    └───────────────────────┘         └───────────────────────┘
```

---

## 1. The Three Core Archetypes & Subagent Workers

### 1. Melchior-1 (`melchior.rs`) — The Scientist
* **Analytical Lens:** Logic, algorithmic correctness, clean architecture, cyclomatic complexity, performance bottlenecks, and modular separation of concerns.
* **Persona Prompt:** Rigorous, mathematically disciplined, and intolerant of structural antipatterns or memory unsafety.
* **Specialist Incident Domain:** Runtime panics, concurrency race conditions, memory leaks, algorithmic complexity, and architectural debt.
* **Delegated Subagent Worker (`systems_analyst`):** Rapidly analyzes algorithmic data flow, concurrency patterns, complexity ($O(N)$), and technical trade-offs mid-generation.
* **Prompt Source:** [`client/skills/magi-system/melchoir-1.md`](../../client/skills/magi-system/melchoir-1.md).

### 2. Balthasar-2 (`balthasar.rs`) — The Mother
* **Analytical Lens:** Defensive cybersecurity, zero-trust vulnerability auditing, secret leaks, privilege escalation, OWASP Top 10, and Common Weakness Enumeration (CWE).
* **Unilateral Security Veto Power:** If Balthasar evaluates a proposal with `risk_score >= 8` and casts a `REJECT` vote, the entire deliberation is immediately overridden with `VETO_BALTHASAR_SECURITY`, regardless of affirmative votes from Melchior and Casper.
* **Specialist Incident Domain:** Authentication bypass, secret exposure, authorization flaws, injection vulnerabilities, and TLS/SSL misconfigurations.
* **Delegated Subagent Worker (`security_scanner`):** Rapidly scans attack vectors, injection points, sanitization boundaries, and matches potential CVE/CWE vulnerabilities.
* **Prompt Source:** [`client/skills/magi-system/balthasar-2.md`](../../client/skills/magi-system/balthasar-2.md).

### 3. Casper-3 (`casper.rs`) — The Woman / Pragmatist
* **Analytical Lens:** Real-world pragmatism, developer ergonomics (DX), maintainability, team cognitive load, operational overhead, and anti-overengineering.
* **Strict Anti-Overengineering Mandate:** Casper-3 has a zero-tolerance policy against unnecessary external daemons, distributed databases for local CLIs, or premature complexity introduced for novelty.
* **Specialist Incident Domain:** Configuration errors, missing dependencies, build environment setup, excessive boilerplate, and DX friction.
* **Delegated Subagent Worker (`pragmatic_evaluator`):** Evaluates implementation complexity, operational overhead, maintainability, and delivery risk.
* **Prompt Source:** [`client/skills/magi-system/casper-3.md`](../../client/skills/magi-system/casper-3.md).

---

## 2. Multi-Round Trinity Deliberation Protocol

Rather than accepting a single LLM output, MAGI executes an asynchronous multi-round debate protocol:

```text
Payload / Proposal / Code Diff / Incident
        │
        ├──▶ [Round 1: Parallel Independent Evaluation]
        │     Melchior-1, Balthasar-2, Casper-3 evaluate concurrently
        │     without awareness of peer stances (eliminates anchoring bias).
        │
        ├──▶ [Intermediate Rounds: 2 .. N-1 (if --rounds > 2)]
        │     Previous peer arguments and risk scores are exposed as untrusted
        │     inputs. Nodes challenge weak logic, identify agreements and conflicts.
        │
        ├──▶ [Final Round: Definitive Irrevocable Vote]
        │     Nodes issue definitive final votes, risk scores, and post-debate
        │     conclusions explaining whether they maintain or change stance.
        │
        └──▶ [SpacetimeDB Atomic Consensus Reducer]
              Votes are collected in SpacetimeDB table `node_vote`.
              Reducer evaluates Balthasar Veto -> Unanimous -> Majority -> Split rules.
              Verdict sealed in `consensus_result`.
```

---

## 3. Comprehensive Decision Matrix

The atomic consensus reducer (`server/src/lib.rs`) and client consensus helper (`client/src/core/helpers/consensus_helper.rs`) implement this deterministic resolution matrix:

| Melchior-1 | Balthasar-2 | Casper-3 | Consensus Result | Classification & Meaning |
| :---: | :---: | :---: | :---: | :--- |
| `APPROVE` | `APPROVE` | `APPROVE` | `APPROVED_UNANIMOUS` | 3–0 Unanimous Approval |
| `APPROVE` | `APPROVE` | `REJECT` | `APPROVED_MAJORITY` | 2–1 Majority Approval (Casper dissent) |
| `APPROVE` | `REJECT` (Risk < 8) | `APPROVE` | `APPROVED_MAJORITY` | 2–1 Majority Approval (Balthasar non-veto reject) |
| `APPROVE` | `REJECT` (Risk $\ge$ 8) | `APPROVE` | `VETO_BALTHASAR_SECURITY` | **Unilateral Security Veto Triggered** |
| `APPROVE` | `APPROVE` | `[OFFLINE]` | `APPROVED_MAJORITY` | 2–0–1 Degraded Quorum Majority Approval |
| `REJECT` | `REJECT` | `APPROVE` | `REJECTED_MAJORITY` | 1–2 Majority Rejection |
| `REJECT` | `REJECT` | `REJECT` | `REJECTED_UNANIMOUS` | 0–3 Unanimous Rejection |
| `REJECT` | `REJECT` | `[OFFLINE]` | `REJECTED_MAJORITY` | 0–2–1 Degraded Quorum Majority Rejection |
| `APPROVE` | `REJECT` | `NEUTRAL` | `SPLIT_DECISION` | 1–1–1 Split Decision (Human operator review) |
| `NEUTRAL` | `NEUTRAL` | `NEUTRAL` | `SPLIT_DECISION` | Neutral deliberation requiring clarification |
| Any | Any | Any | `CONSENSUS_UNAVAILABLE` | Database or upstream network failure |

---

## 4. Failover & Quorum Resilience

### Hot-Standby Failover Brain (`MAGI_FALLBACK_MODEL`)
When a node's primary model fails (e.g. out of API credits, HTTP 402, persistent 429 rate limits, or provider outages), the node transparently engages the configured fallback model (e.g. `cohere/north-mini-code:free` or local Ollama) without aborting or triggering degraded quorum:
* Emits console notification: `[{node_id}] Primary model ({model}) failed: {err}. Engaging backup circuit ({backup})...`
* Tags resulting evaluation model as `<model> (backup)` in SpacetimeDB history and Markdown reports.

### Degraded Quorum Tolerance (2-of-3)
If 1 node drops offline or exhausts retries, deliberation does not fail:
* MAGI synthesizes an explicit `[OFFLINE/DEGRADED QUORUM]` neutral position.
* Proceeding with the remaining 2 nodes across rounds, majority decisions (2 Approves or 2 Rejects) are validly resolved.
* If 2 or more nodes fail, the session fails closed to protect consensus integrity.

---

## 5. Dynamic Language Adaptation (i18n)

MAGI automatically adapts all terminal telemetry, phosphor badges, debate prompts, and Markdown reports based on language detection or explicit configuration:
* **Detection:** Evaluates prompt tokens and context to select English (`en`) or Spanish (`es`).
* **Explicit Override (`MAGI_LANG`):** Set `MAGI_LANG=es` or `MAGI_LANG=en` in `.env`.
* **Catalogs (`client/i18n/`):** Loaded from [`en.json`](../../client/i18n/en.json) and [`es.json`](../../client/i18n/es.json).
* **Contextual Fallbacks:** If a node maintains its position in Round 2 without redundant explanation, MAGI synthesizes a localized contextual post-debate conclusion:
  * *Spanish:* `"Mantiene su postura inicial tras revisar los argumentos de los pares: <argumento>"`
  * *English:* `"Maintains initial position after reviewing peer arguments: <argument>"`
