# MAGI System — CLI & TUI Manual 

MAGI operates as a **hybrid developer tool** designed for your terminal, combining scriptable single-shot CLI commands, an interactive dual-pane History Browser, a full-screen NERV command deck, and a native Model Context Protocol (MCP) stdio server.

---

## 1. Single-Shot CLI Subcommands

Every audit subcommand executes the **Two-Round Trinity Consensus Protocol**, prints diegetic phosphor CRT diagnostics, persists a Markdown report to `./deliberations/deliberation_XXXXXX_<slug>.md`, records the transaction in SpacetimeDB, and exits with semantic exit codes.

### Case 1: Git Diff Auditing (`diff`)
Audits repository changes (working tree, staged changes, or comparisons against a target branch) before committing or creating a pull request:

```powershell
# Windows
.\magi.ps1 diff                        # Working tree unstaged changes
.\magi.ps1 diff --staged               # Staged git index changes
.\magi.ps1 diff --branch origin/main   # Changes relative to a target branch

# Linux / Docker
docker compose run --rm magi diff --staged
```

* **Smart Noise & Lockfile Filtering:** Automatically filters noise files (`Cargo.lock`, `package-lock.json`, `pnpm-lock.yaml`, minified bundles, `.map` files) to conserve LLM context tokens.
* **Specialist Analysis:** Melchior audits correctness and logic regressions; Balthasar audits secret leakage, permissions, and security veto; Casper audits anti-overengineering and diff ergonomics.

---

### Case 2: Architectural Debate & Dilemmas (`debate`)
Submits a technical dilemma, architectural choice, or technology trade-off to the Trinity:

```powershell
# Windows
.\magi.ps1 debate "WebSockets vs Server-Sent Events (SSE) for real-time notifications"

# Linux / Docker
docker compose run --rm magi debate "WebSockets vs Server-Sent Events (SSE) for real-time notifications"
```

* **Melchior-1:** Debates algorithmic efficiency, distributed systems trade-offs, and scalability.
* **Balthasar-2:** Debates threat surface, proxy/firewall traversal, connection exhaustion, and DOS resilience.
* **Casper-3:** Debates implementation simplicity, team DX, maintainability, and delivery overhead.

---

### Case 3: Idea & RFC Viability Review (`idea`)
Audits an architectural proposal, RFC, or product idea for theoretical soundness, security threats, and overengineering:

```powershell
# Windows
.\magi.ps1 idea docs/rfcs/ROADMAP.md

# Linux / Docker
docker compose run --rm magi idea docs/rfcs/ROADMAP.md
```

---

### Case 4: Code Maintenance under Guidelines (`maintain`)
Audits existing code or refactor proposals against team rules, coding conventions, or clean code standards:

```powershell
# Windows
.\magi.ps1 maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md

# Linux / Docker
docker compose run --rm magi maintain client/src/config.rs --guidelines docs/guides/OPERATIONS.md
```

---

### Case 5: Error & Incident Triage (`triage`)
Diagnoses a runtime error, panic, or log file. MAGI automatically routes the opening analysis to the relevant specialist node, followed by a mandatory full-Trinity debate:

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
* **Mandatory Trinity Consensus:** The full Trinity debates the specialist's initial finding to deliver the final resolution.

---

### Case 6: Free-Form Deliberation (`prompt`)
Submit any custom technical question directly to the Trinity without a subcommand:

```powershell
.\magi.ps1 "Should we store JWTs in HttpOnly cookies or localStorage for an internal SPA?"
```

---

## 2. History & Archive Inspection Subcommands

### Interactive History Browser (`history`)
Launches an interactive dual-pane TUI terminal explorer powered by `ratatui` and `crossterm`:

```powershell
.\magi.ps1 history
```

* **Keyboard Navigation:** Scroll through past deliberations with `↑`/`↓`.
* **Live Node Breakdown:** Inspect individual votes and risk ratings for Melchior, Balthasar, and Casper in the right pane.
* **Live Query Filtering:** Press `/` to filter records by title, verdict, or node arguments.
* **Full Report Modal:** Press `Enter` or `Space` to open the full Markdown audit report in a scrollable terminal modal.
* **Non-Interactive ANSI Table Mode:** Pass `--table` or pipe output to render an instant formatted table:
  ```powershell
  .\magi.ps1 history --table
  .\magi.ps1 history --query "Argon2id" --table
  .\magi.ps1 history | Select-String "APPROVED"
  ```

### Inspect Single Deliberation (`show`)
Retrieves and displays a past deliberation by its numeric ID. Uses a hybrid loader that queries SpacetimeDB first and transparently falls back to local Markdown reports on disk:

```powershell
.\magi.ps1 show 1
```

### Database & Archive Purge (`purge`)
Cleans local state, Markdown reports, and SpacetimeDB tables:

```powershell
.\magi.ps1 purge                # Soft wipe: wipes SpacetimeDB & local deliberation markdown files
.\magi.ps1 purge --keep-files   # Wipes SpacetimeDB database while preserving markdown reports on disk
.\magi.ps1 purge --hard         # Hard reset: stops containers, removes volumes, and resets state
```

### Status Probe (`status`)
Pings the SpacetimeDB transactional engine and displays configured node models, fallback circuits, and active endpoints:

```powershell
.\magi.ps1 status
```

---

## 3. Deliberation Flags & Options

### Multi-Round Deep Debate (`--rounds N`)
By default, MAGI executes a **2-round deliberation** (Round 1 opening evaluations followed by Round 2 peer debate). For critical architectural crossroads or strict RFC audits, increase deliberation depth up to $N$ rounds:

```powershell
.\magi.ps1 debate "REST vs gRPC for internal microservices" --rounds 3
```

### Custom Injected Skills (`--skill <path>`)
Inject specialized guidelines or team rules into persona system prompts without recompiling source code:

```powershell
.\magi.ps1 idea docs/rfcs/ROADMAP.md --skill skills/custom/team-rules.md
```

### Machine-Readable JSON Export (`--output json`)
Emits a structured JSON envelope for CI/CD gates, automated linters, and external integrations:

```powershell
.\magi.ps1 diff --staged --output json
```

Envelope structure:
```json
{
  "deliberation_id": 1,
  "title": "Debate: Argon2id vs BCrypt",
  "category": "TECHNICAL_DEBATE",
  "context_type": "DILEMMA",
  "verdict": "APPROVED_MAJORITY",
  "summary": "Trinity Consensus: APPROVED_MAJORITY. Tallies -> [APPROVE: 2, REJECT: 0, NEUTRAL: 1]",
  "rounds": 2,
  "nodes": [
    {
      "node_id": "Melchior-1",
      "final_vote": "APPROVE",
      "initial_vote": "APPROVE",
      "risk_score": 5,
      "initial_risk_score": 5,
      "confidence": 0.75,
      "model": "deepseek/deepseek-v4.1-flash",
      "execution_time_ms": 23190,
      "cwe_flags": [],
      "rationale": "MANTENGO mi voto APPROVE..."
    }
  ]
}
```

### Offline Mock Simulations (`--mock`, `--simulate-veto`)
Execute deterministic simulations without invoking external LLM APIs:
```powershell
.\magi.ps1 debate "Test proposal" --mock
.\magi.ps1 debate "Veto test" --mock --simulate-veto
```

---

## 4. Semantic Process Exit Codes

All CLI subcommands return standard semantic process exit codes for automated pipelines and git pre-commit hooks:

| Exit Code | Meaning | Condition |
| :---: | :--- | :--- |
| `0` | **APPROVED** | Majority or unanimous `APPROVE` verdict reached |
| `1` | **REJECTED** | Majority `REJECT` or Balthasar-2 security veto triggered |
| `2` | **SPLIT / NEUTRAL** | Tie, neutral consensus, or consensus unavailable |
| `3` | **ERROR** | Runtime failure, config error, or missing file |

---

## 5. Model Context Protocol (MCP) Server Suite

MAGI implements the **Model Context Protocol (MCP)** specification over `stdio` (JSON-RPC 2.0, supporting protocol discovery and ping), providing 5 specialized tools for autonomous AI coding assistants (Antigravity, Claude Desktop, Cursor):

### Registered MCP Tools:
1. **`deliberate_with_magi`**: Universal code and architectural proposal deliberation.
2. **`audit_git_changes`**: Direct git changes auditing (unstaged, staged, or vs branch).
3. **`triage_incident_with_magi`**: Error diagnosis with lead specialist routing.
4. **`check_security_veto`**: Fast single-node Balthasar-2 security veto check.
5. **`debate_technical_dilemma`**: Multi-agent architectural dilemma debate.

### Running the MCP Server:
```powershell
.\magi.ps1 mcp
```

### Antigravity / Claude Desktop Configuration (`mcp_config.json`):
```json
{
  "mcpServers": {
    "magi": {
      "command": "powershell",
      "args": ["-NoProfile", "-ExecutionPolicy", "Bypass", "-File", ".\\magi.ps1", "mcp"]
    }
  }
}
```

---

## 6. Interactive NERV Command Deck (TUI Console)

Launch the full-screen terminal dashboard:

```powershell
.\magi.ps1 tui       # (or .\magi.ps1 console)
```

Inside the console:
* `status` — Inspect SpacetimeDB connection and active node models.
* `history` — Query past deliberations directly inside the REPL.
* `clear` — Clear the terminal buffer.
* `exit` / `quit` — Safely exit.
* **Natural Language Queries:** Type any technical question (e.g. `revisa docs/rfcs/ROADMAP.md` or `evaluar migración de base de datos`) to trigger a live 2-round Trinity debate directly inside the console.
