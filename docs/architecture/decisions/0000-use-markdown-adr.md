# 0. Use of Architecture Decision Records (ADR)

Date: 2025-11-22
Status: Accepted

## Context
We need to record significant architectural decisions for the Tyrus project.
Since this project aims to be both a complex engineering tool (compiler) and an academic artifact, it is crucial to keep a history of the "why" and "how" behind decisions.
Lack of documentation about design decisions can lead to rework, loss of context, and difficulty defending the master's thesis.

## Decision
We will adopt the **Architecture Decision Records (ADR)** format.
We will use sequentially numbered Markdown files in the `docs/architecture/decisions` folder.

Each ADR must follow this structure:
1. **Title:** Short and descriptive.
2. **Status:** Proposed, Accepted, Deprecated, or Rejected.
3. **Context:** What problem are we trying to solve? What are the constraints?
4. **Decision:** What will we do? (Technology, Pattern, Algorithm).
5. **Consequences:** What we gain (pros) and what we lose/pay (cons) with this decision.

## Consequences
### Positive
- Clear history of the project's evolution.
- Eases onboarding of new contributors (Open Source).
- Ready-made material for writing the master's dissertation.

### Negative
- Requires discipline to write the document before or during implementation.
