# OpenComp — Agent Bootstrap Guide

> Read this FIRST, every session. It is the project constitution.
> Any agent, any machine, any model (OmniRoute included) continues from these files — nothing else.

## What this is

OpenComp is an open, minimal, **agent-first** compositing engine — a from-the-ground-up
reimagining of After Effects. Projects are TOML (git-diffable), the core is Rust,
agents are first-class users (Python SDK + REST planned). The compositing core is the
product; everything else is a plugin around it.

## Non-negotiable rules

1. **TDD** — production code only after a failing test (test-driven-development skill).
2. **Docs-first** — architecture changes land in `docs/ARCHITECTURE.md`; decisions get an ADR (`docs/ADRs/`).
3. **Conventional commits** — `feat:` `fix:` `docs:` `chore:` `refactor:` `test:`.
4. **The repo is the memory** — never assume context from a previous session. Read the files.
5. **Session records are mandatory** — DEVLOG entry + STATUS refresh at end of every session.

## Session start ritual

1. Read this file.
2. Read `docs/STATUS.md` — where we are, next 3 actions.
3. Read the tail of `docs/DEVLOG.md` — what the last session did.
4. Check blocked items (STATUS.md; GitHub Issues once remote exists).
5. Run `cargo test` — the suite must be green before starting new work.

## Session end ritual

1. `cargo test` green.
2. Append one `docs/DEVLOG.md` entry (date, what changed, decisions, what's next).
3. Refresh `docs/STATUS.md` (progress, blocked, next 3 actions).
4. Commit with a conventional message.

## Commands

- Build: `cargo build`
- Test: `cargo test`
- Run: `cargo run -- render examples/demo.toml -o out/demo.png`
- Format: `cargo fmt`

## Architecture map

- `docs/ARCHITECTURE.md` — the design; read before touching structure
- `docs/PLAN_PHASE1.md` — current phase plan (task-by-task, TDD)
- `docs/PROJECT_FORMAT.md` — the TOML project format spec
- `src/project.rs` — project model (the format contract)
- `src/compositor.rs` — rasterization (Phase 1 target)