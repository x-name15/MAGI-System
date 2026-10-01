# MAGI System Documentation 📚

Welcome to the technical documentation for **MAGI System**. The documentation is organized into modular categories:

```text
docs/
├── architecture/
│   ├── ARCHITECTURE.md       # SpacetimeDB engine, WASM reducers, concurrency model
│   └── TRINITY.md            # The Evangelion Trinity, archetypes, and 2-round debate protocol
├── guides/
│   ├── GETTING_STARTED.md    # Quickstart, zero-cost free AI keys, and CLI reference
│   └── OPERATIONS.md         # Docker runbook, container lifecycle, and troubleshooting
├── rfcs/
│   ├── IDEA.md               # Original foundational proposal and design thesis
│   ├── ROADMAP.md            # Future milestones and release plans
│   └── LOG.md                # Engineering diary and audit history
└── README.md                 # This index
```

---

## 🏛️ Architecture & Core Concepts
* **[System Architecture](architecture/ARCHITECTURE.md):** Deep dive into the SpacetimeDB in-memory transactional database, WASM reducers, atomic consensus rules, and client-server WebSocket transport.
* **[The Evangelion Trinity](architecture/TRINITY.md):** Detailed breakdown of Melchior-1 (Scientist), Balthasar-2 (Mother), and Casper-3 (Woman), including Balthasar's unilateral veto and Casper's anti-overengineering mandate.

---

## 🚀 Guides & Operations
* **[Getting Started & Free AI Keys](guides/GETTING_STARTED.md):** Step-by-step setup guide with 100% free API key configurations (Google Gemini, Groq Cloud, OpenRouter, and offline `MOCK` mode).
* **[Operations Runbook](guides/OPERATIONS.md):** Docker container management, PowerShell automation (`magi.ps1`), memory limits, volume persistence, and verification checklist.

---

## 📋 Specifications & RFCs
* **[Original Vision (IDEA.md)](rfcs/IDEA.md):** The foundational document outlining the motivations, theoretical framework, and initial system design.
* **[Project Roadmap](rfcs/ROADMAP.md):** Milestone tracker from V0.1 core consensus to future V1.1 and V2 enhancements.
* **[Engineering Log](rfcs/LOG.md):** Chronological log of architecture changes, testing results, and system migrations.
