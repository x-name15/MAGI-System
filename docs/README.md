# MAGI System Documentation 📚

Welcome to the technical documentation for **MAGI System v0.2.6**.

MAGI System is an asynchronous, event-driven terminal multi-agent consensus engine inspired by the supercomputer from *Neon Genesis Evangelion*. It audits code changes, architectural proposals, technical dilemmas, and production incidents through a parallel **Two-Round Trinity Deliberation Protocol** backed by an in-memory transactional database ([SpacetimeDB](https://spacetimedb.com)).

---

## 🗺️ Documentation Directory Map

```text
docs/
├── README.md                 # Documentation portal (this file)
├── architecture/
│   ├── ARCHITECTURE.md       # Core architecture: SpacetimeDB, WASM reducers, resilience & MCP
│   └── TRINITY.md            # The Evangelion Trinity: archetypes, veto rules, consensus matrix & subagents
├── guides/
│   ├── GETTING_STARTED.md    # Quickstart guide (in under 2 minutes) & core workflows
│   ├── CONFIGURATION.md      # Vendor-agnostic LLM config, failover models, subagents & limits
│   ├── CLI_AND_TUI.md        # Complete manual for CLI subcommands, history TUI browser & MCP suite
│   └── OPERATIONS.md         # Docker runbook, container lifecycle, purging & troubleshooting
└── rfcs/
    ├── IDEA-ES.md            # Foundational architectural design thesis (Spanish)
    ├── ROADMAP.md            # Milestone roadmap (v0.1.3 through upcoming v0.3.0)
    └── LOG.md                # Architectural Decision Records (ADRs) and engineering log
```

---

## 🏛️ Architecture & Core Concepts

* **[System Architecture](architecture/ARCHITECTURE.md):** Deep dive into the SpacetimeDB transactional engine, WASM reducers, dual-archive hybrid persistence, fault-tolerant degraded quorum, smart payload budgeting, and Model Context Protocol (MCP) server.
* **[The Evangelion Trinity & Consensus Protocol](architecture/TRINITY.md):** Detailed breakdown of Melchior-1 (Scientist), Balthasar-2 (Mother), and Casper-3 (Woman), unilateral security veto thresholds, persona-specialized subagents (`openrouter:subagent`), hot-standby failover, and the complete consensus matrix.
* **[Foundational Vision (IDEA-ES.md)](rfcs/IDEA-ES.md):** The original architectural thesis (in Spanish) outlining the design principles, motivation, and theoretical framework.

---

## 🚀 Guides & Manuals

* **[Getting Started](guides/GETTING_STARTED.md):** Rapid 2-minute setup guide covering Windows PowerShell (`magi.ps1`), Linux/macOS Docker Compose, and pre-compiled standalone binaries.
* **[Configuration Guide (API Keys & Providers)](guides/CONFIGURATION.md):** Vendor-agnostic configuration guide for unified global vs. granular per-node settings, OpenRouter server tools, hot-standby fallback models, generation token budgets, and local inference (Ollama, LM Studio, vLLM).
* **[CLI & TUI Manual](guides/CLI_AND_TUI.md):** Complete operational manual covering all 5 core subcommands (`idea`, `maintain`, `triage`, `diff`, `debate`), the interactive dual-pane History Browser (`magi history`), database purging (`magi purge`), the full-screen NERV TUI console (`magi tui`), and the 5-tool MCP server suite.
* **[Operations Runbook](guides/OPERATIONS.md):** Docker container lifecycle, volume persistence, cache management, test verification, SpacetimeDB maintenance, and operational troubleshooting.

---

## 📋 Roadmaps & Engineering History

* **[Project Roadmap](rfcs/ROADMAP.md):** Milestone tracker from v0.1.3 foundation through v0.2.6 (subagents & server tools) and upcoming v0.3.0 (codebase AST awareness).
* **[Engineering Log (ADRs)](rfcs/LOG.md):** Chronological Architectural Decision Records detailing every major technical pivot, resilience feature, and integration decision.
