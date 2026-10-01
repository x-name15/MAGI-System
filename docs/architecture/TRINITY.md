# The Evangelion Trinity & Consensus Protocol 🧠

In *Neon Genesis Evangelion*, the MAGI supercomputer governs decision-making through three biological computers embodying the three distinct facets of its creator, Dr. Naoko Akagi. In **MAGI System**, this cybernetic philosophy is translated into high-reliability software architecture.

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

## 🎭 The Three Core Archetypes

### 1. 🔬 Melchior-1 (`melchior.rs`) — The Scientist
* **Analytical Lens:** Logic, algorithmic correctness, clean architecture, cyclomatic complexity, performance bottlenecks, and modular separation of concerns.
* **Persona Prompt:** Rigorous, mathematically disciplined, and intolerant of structural antipatterns or memory unsafety.
* **Specialist Domain:** Runtime panics, concurrency race conditions, memory leaks, algorithmic complexity, architectural debt.

### 2. 🛡️ Balthasar-2 (`balthasar.rs`) — The Mother
* **Analytical Lens:** Defensive security, zero-trust vulnerability auditing, secret leaks, privilege escalation, OWASP Top 10, and Common Weakness Enumeration (CWE).
* **Unilateral Security Veto:** If Balthasar evaluates a proposal with `risk_score >= 8` and casts a `REJECT` vote, the entire deliberation is immediately overridden with `VETO_BALTHASAR_SECURITY`, regardless of affirmative votes from Melchior and Casper.
* **Specialist Domain:** Authentication bypass, secret exposure, authorization flaws, injection vulnerabilities, TLS/SSL misconfigurations.

### 3. ⚡ Casper-3 (`casper.rs`) — The Woman / Pragmatist
* **Analytical Lens:** Real-world pragmatism, developer ergonomics (DX), maintainability, team cognitive load, operational overhead, and anti-overengineering.
* **Strict Anti-Overengineering Rule:** Casper-3 has a zero-tolerance policy against unnecessary external daemons, distributed databases for local CLIs, or premature complexity introduced for novelty or "architectural fun".
* **Specialist Domain:** Configuration errors, missing dependencies, build environment setup, excessive boilerplate, DX friction.

---

## 🔄 The 2-Round Trinity Deliberation Protocol

Rather than accepting a single LLM output, MAGI executes an asynchronous two-round debate protocol:

```text
Payload / Proposal
        │
        ├──▶ [Round 1: Parallel Independent Evaluation]
        │     Melchior-1, Balthasar-2, Casper-3 evaluate concurrently
        │     without awareness of peer stances.
        │
        ├──▶ [Round 2: Cross-Peer Inter-Node Debate]
        │     First-round stances, risks, and arguments are crossed
        │     between peers as untrusted input. Nodes challenge weak logic.
        │
        └──▶ [SpacetimeDB Atomic Consensus Reducer]
              Votes are collected in SpacetimeDB table `node_votes`.
              Reducer evaluates Balthasar Veto -> Unanimous -> Majority rules.
              Verdict sealed in `consensus_results`.
```

---

## 🚨 Decision Matrix

| Melchior-1 | Balthasar-2 | Casper-3 | Consensus Result | Classification |
| :---: | :---: | :---: | :---: | :--- |
| `APPROVE` | `APPROVE` | `APPROVE` | `APPROVED_UNANIMOUS` | 3–0 Unanimous Approval |
| `APPROVE` | `APPROVE` | `REJECT` | `APPROVED_MAJORITY` | 2–1 Majority Approval |
| `APPROVE` | `REJECT` (Risk < 8) | `APPROVE` | `APPROVED_MAJORITY` | 2–1 Majority Approval |
| `APPROVE` | `REJECT` (Risk $\ge$ 8) | `APPROVE` | `VETO_BALTHASAR_SECURITY` | **Unilateral Security Veto** |
| `REJECT` | `REJECT` | `APPROVE` | `REJECTED_MAJORITY` | 1–2 Majority Rejection |
| `REJECT` | `REJECT` | `REJECT` | `REJECTED_UNANIMOUS` | 0–3 Unanimous Rejection |
| Any | Any | Any | `CONSENSUS_UNAVAILABLE` | Database / Network Failure |

---

## 🌐 Dynamic Language Adaptation

MAGI automatically adapts its entire terminal telemetry, monitor badges, and consensus synthesis to the language of the prompt:
* **Spanish Prompt:** Renders terminal telemetry in Spanish (`[RONDA 1: EVALUACIÓN INDEPENDIENTE]`, `SÍNTESIS`, `CONSENSO`).
* **English Prompt:** Renders terminal telemetry in English (`[ROUND 1: INDEPENDENT EVALUATION]`, `SYNTHESIS`, `CONSENSUS`).
