# ADR-0001: Repo-as-memory for multi-session development

Status: Accepted
Date: 2026-09-16

## Context

OpenComp will be developed over many sessions — possibly hundreds — across machines
and models (local, OmniRoute gateways, different Hermes profiles). Agents have no
persistent context between sessions; naive handoffs cause rework and scope drift.
We need cohesion without a central "agent brain".

## Decision

**The repository is the memory.** Every session starts by reading `AGENTS.md` +
`docs/STATUS.md` + the tail of `docs/DEVLOG.md`, and ends by updating the records.
Architecture decisions are recorded as numbered ADRs. Tests are the executable
contract. Git history is the audit trail. A fresh agent with zero context must be
able to answer "where are we, what's next?" from the repo alone.

## Consequences

- Any agent / any machine / any model can continue with no handoff.
- Settled decisions are not re-litigated (ADRs).
- Status is never lost (STATUS.md + DEVLOG.md stay current).
- The session-end ritual is mandatory discipline; a skipped ritual degrades the next session.
- Records can drift if not enforced — enforced via AGENTS.md rules and review.