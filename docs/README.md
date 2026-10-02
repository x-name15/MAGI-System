# MAGI System Documentation 📚

Welcome to the technical documentation for **MAGI System**. The documentation is structured into modular categories:

```text
docs/
├── README.md                 # Documentation index (this file)
├── IDEA.md                   # Foundational architectural design thesis
├── architecture/
│   ├── ARCHITECTURE.md       # SpacetimeDB engine, WASM reducers, client architecture
│   └── TRINITY.md            # The Evangelion Trinity, archetypes, and 2-round debate protocol
├── guides/
│   ├── GETTING_STARTED.md    # Quickstart (2 minutes) and core workflows
│   ├── CLI_AND_TUI.md        # Complete manual for CLI subcommands and interactive TUI
│   └── OPERATIONS.md         # Docker runbook, container lifecycle, and troubleshooting
└── rfcs/
    ├── ROADMAP.md            # Realistic and high-impact milestone roadmap (v0.1.3 to v1.0.0)
    └── LOG.md                # Chronological architectural decision records (ADRs)
```

---

## 🏛️ Architecture & Core Concepts
* **[System Architecture](architecture/ARCHITECTURE.md):** Deep dive into the SpacetimeDB in-memory transactional database, WASM reducers, atomic consensus rules, and client-server WebSocket transport.
* **[The Evangelion Trinity](architecture/TRINITY.md):** Detailed breakdown of Melchior-1 (Scientist), Balthasar-2 (Mother), and Casper-3 (Woman), including Balthasar's unilateral veto and Casper's anti-overengineering mandate.
* **[Foundational Vision (IDEA.md)](IDEA.md):** The original architectural thesis outlining the motivations, theoretical framework, and system design.

---

## 🚀 Guides & Manuals
* **[Getting Started](guides/GETTING_STARTED.md):** Step-by-step setup guide to install and execute your first deliberation in under 2 minutes.
* **[CLI & TUI Manual](guides/CLI_AND_TUI.md):** In-depth reference for `idea`, `maintain`, `triage`, free prompts, custom `--skill` injection, and the interactive NERV command deck.
* **[Operations Runbook](guides/OPERATIONS.md):** Docker container management, PowerShell automation (`magi.ps1`), memory limits, volume persistence, and verification checklist.

---

## 📋 Roadmaps & Engineering History
* **[Project Roadmap](rfcs/ROADMAP.md):** Milestone tracker from v0.1.3 core consensus to v0.2.0 (streaming/local-first), v0.3.0 (codebase awareness), and v1.0.0 (production distribution).
* **[Engineering Log](rfcs/LOG.md):** Chronological log of architecture decisions (ADRs), testing results, and system migrations.
