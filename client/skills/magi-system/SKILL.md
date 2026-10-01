---
name: magi-system
description: Core persona system prompts and evaluation protocols for the MAGI Trinity (Melchior-1, Balthasar-2, Casper-3).
---

# MAGI System Skill & Persona Prompts

This directory contains the externalized system prompts and analytical instructions for the MAGI supercomputer Trinity:

- `melchior.md`: Melchior-1 (The Scientist — Logic, Architecture, Complexity, Scalability)
- `balthasar.md`: Balthasar-2 (The Mother — Cybersecurity, Defensive Boundary, VETO Authority)
- `casper.md`: Casper-3 (The Woman — Pragmatism, Developer Experience, Simplicity, Cost)

## Custom Skills & Pre-Debate Instructions
Developers can inject project-specific skills, team guidelines, or security policies into the Trinity before deliberation starts:

1. **Via CLI Flag**:
   ```bash
   magi deliberate src/auth.rs --skill skills/team-rules.md
   ```
2. **Via Skills Directory**:
   Place markdown files in `skills/<skill-name>.md` or `skills/<skill-name>/SKILL.md`.

When loaded, custom skills are prepended to the task context of all three nodes, ensuring every node debates with full situational awareness of the project's unique constraints.
