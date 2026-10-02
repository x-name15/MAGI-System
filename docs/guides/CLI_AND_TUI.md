# MAGI System — CLI & TUI Manual 

MAGI operates as a **hybrid developer tool** designed for your terminal, combining scriptable single-shot CLI commands with a full-screen interactive NERV command console.

---

## 1. Single-Shot CLI Subcommands

Every subcommand executes the **Two-Round Trinity Consensus Protocol**, prints real-time phosphor CRT diagnostics, saves a persistent Markdown report to `./deliberations/`, and exits with meaningful exit codes.

### Case 1: Idea & RFC Viability Review (`idea`)
Audits an architectural proposal, RFC, or product idea for theoretical soundness, security threats, and overengineering.

```powershell
# Windows
.\magi.ps1 idea docs/rfcs/ROADMAP.md

# Linux / Docker
docker compose run --rm magi idea docs/rfcs/ROADMAP.md
```

* **Melchior-1:** Audits theoretical soundness, system trade-offs, and scalability.
* **Balthasar-2:** Audits security attack surfaces, data privacy, and enforces veto power.
* **Casper-3:** Audits practical delivery effort, operational overhead, and anti-overengineering.

---

### Case 2: Code Maintenance under Guidelines (`maintain`)
Audits existing code or refactor proposals against team rules, coding conventions, or clean code standards.

```powershell
# Windows
.\magi.ps1 maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md

# Linux / Docker
docker compose run --rm magi maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md
```

* **Melchior-1:** Audits pattern adherence, technical debt, and modularity.
* **Balthasar-2:** Audits regression hazards, backward compatibility, and security vulnerabilities.
* **Casper-3:** Audits developer cognitive load, boilerplate, and team DX.

---

### Case 3: Error & Incident Triage (`triage`)
Diagnoses a runtime error, panic, or log file. MAGI automatically routes the opening analysis to the relevant specialist node, followed by a mandatory full-Trinity debate.

```powershell
# Windows
.\magi.ps1 triage logs/panic.log --code client/src/main.rs

# Linux / Docker
docker compose run --rm magi triage logs/panic.log --code client/src/main.rs
```

* **Specialist Routing:**
  * Security / Auth / Token / Secret leak $\rightarrow$ **Balthasar-2**
  * Panic / Concurrency / Logic / Architecture $\rightarrow$ **Melchior-1**
  * Config / Env / Dependencies / Missing file $\rightarrow$ **Casper-3**
* **Mandatory Trinity Verdict:** The full Trinity debates the specialist's initial finding to deliver the final resolution.

---

### Case 4: Free-Form Deliberation (`prompt`)
Submit any custom query, architecture dilemma, or question directly to the Trinity:

```powershell
# Windows
.\magi.ps1 prompt "Should we use an embedded in-memory database or a separate daemon for local developer tools?"

# Linux / Docker
docker compose run --rm magi "Should we use an embedded in-memory database or a separate daemon for local developer tools?"
```

---

## 2. Injected Custom Skills (`--skill`)

You can inject specialized rules or guidelines into the MAGI persona system prompts without editing source code:

```powershell
.\magi.ps1 idea docs/rfcs/ROADMAP.md --skill client/skills/magi-system/SKILL.md
```

---

## 3. Language & Internationalization (`MAGI_LANG`)

MAGI dynamically detects prompt language (Spanish or English) and outputs terminal monitors, peer debate prompts, and Markdown reports in that language. You can enforce a language globally via `.env` or flags:

```bash
# Force Spanish output
MAGI_LANG=es

# Force English output
MAGI_LANG=en
```

---

## 4. Interactive NERV Terminal Console (TUI)

Launch the full-screen interactive dashboard built with `ratatui` and `crossterm`:

```powershell
# Windows
.\magi.ps1 tui

# Linux / Docker
docker compose run --rm magi tui
```

```text
┌STATUS───────────────────────────────────────────────────────────────────────┐
│MAGI SYSTEM // NERV COMMAND DECK                                             │
│MELCHIOR-1: gemini-flash-lite    BALTHASAR-2: gemini-flash-lite   CASPER-3: ..│
│Two-round deliberation: opening positions -> peer debate -> Trinity verdict  │
└─────────────────────────────────────────────────────────────────────────────┘
┌NERV CONSOLE─────────────────────────────────────────────────────────────────┐
│ > status                                                                    │
│ System Online. Connected to SpacetimeDB at http://spacetimedb:3000          │
│ > history                                                                   │
│ [MAGI-000028] APPROVED_UNANIMOUS: Deliberation on Roadmap                   │
└─────────────────────────────────────────────────────────────────────────────┘
┌COMMAND INPUT────────────────────────────────────────────────────────────────┐
│ > Should we migrate our backend to Rust?                                    │
└─────────────────────────────────────────────────────────────────────────────┘
```

### In-Console Commands:
* `status` — Check live connection to SpacetimeDB and node model configurations.
* `history` — Query past deliberations and consensus verdicts stored in SpacetimeDB.
* `help` — Show console navigation and available commands.
* `clear` — Clear the console buffer.
* `exit` / `quit` / `Ctrl+C` — Safely exit the TUI.
* **Any natural text query:** Automatically classified and submitted to the full 2-round Trinity debate directly inside the console!
