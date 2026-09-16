# OpenComp — Development Log

> Chronological session records. NEWEST LAST (append at bottom).
> Format: `## [YYYY-MM-DD] Session summary` + bullet details.

## [2026-09-16] Session 1 — Foundations: project model + meta-system

- User's vision: build an open, minimal, agent-first After Effects alternative (OpenComp); developed over many sessions with structural guidance; GitHub public once authorized.
- **Meta-system built** (repo-as-memory, ADR-0001): `AGENTS.md` bootstrap constitution, `docs/STATUS.md` short-term memory, `docs/DEVLOG.md` session log, `docs/ADRs/` decision records, session start/end rituals. Any agent/machine/model continues from these — no context handoff needed.
- **Phase 1 plan** (`docs/PLAN_PHASE1.md`) — 11 tasks, TDD, bite-sized commits.
- **Task 2 DONE**: Project model (`src/project.rs`) — TOML → serde structs:
  - `Project` → `ProjectInfo` (name/fps/width/height/duration/bg_color) + `Vec<Layer>`
  - `Layer` (name/type/color/transform/blend), `Transform` (position/scale/opacity/rotation), `BlendMode`, `LayerKind`
  - Hex color string ↔ `Color` via custom serde deserializers (`color_from_hex`, `opt_color_from_hex`)
  - `to_hex()` returns lowercase (format spec)
  - Format contract locked by tests: `tests/project_test.rs` 3 pass + unit tests 3 pass
- TDD disciplined this session: RED (no crate), RED (raw-string `#"` panic → `r##`), RED (string→Color), RED (Option<Color>) — each fixed by the failing test.
- **Toolchain**: Rust 1.98.1 via rustup (minimal profile + rustfmt). Binary split into lib + bin for integration tests.
- **Blocked**: GitHub remote (Task 11) — user to authorize later.
- **Next**: Task 3 (color hex parsing), Task 4 (background fill).