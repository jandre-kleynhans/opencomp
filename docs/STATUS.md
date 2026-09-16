# OpenComp — Status

> Updated at end of every session. This file is the project's short-term memory.

## Current phase

**Phase 1 — Compositing core** — ✅ COMPLETE

TOML project in → RGBA8 frame out. Tests verified by eye and by pixel math.
27 tests, all passing. Demo renders correctly.

## Progress

- [x] Repo scaffold + docs + meta-system (AGENTS.md, ADRs, repo-as-memory) — 2026-09-16
- [x] Project model (serde + TOML parsing) — Task 2 — 2026-09-16
- [x] Color hex parsing (Task 3) — 2026-09-16
- [x] Background fill (Task 4) — 2026-09-16
- [x] Solid layer rasterization (Task 5) — 2026-09-16
- [x] Transforms: position/scale/opacity/rotation + layer `size` field (Task 6) — 2026-09-16
- [x] Layer stacking (Task 7) — built into Task 6 — 2026-09-16
- [x] PNG export (Task 8) — 2026-09-16
- [x] CLI `render` (Task 9) — 2026-09-16
- [x] Demo project + README (Task 10) — 2026-09-16
- [x] GitHub public (Task 11) ✅ — https://github.com/jandre-kleynhans/opencomp — 2026-09-16

## Known issues

- None open.

## Next 3 actions

1. **Phase 2** — keyframe engine: time-value pairs per property, easing curves, Python expression evaluator
2. **ADR-0002** — document the full-canvas layer centering convention (position ignored; center at canvas center)
3. **Setup GitHub Issues** as the task tracker once Phase 2 plan is drafted (repo-as-memory → issues-as-work)

## Blocked

- Nothing.

---
*Task 11 (GitHub public) ✅ done 2026-09-16: key `mothership-opencomp`, alias `github-opencomp`, remote `git@github-opencomp:jandre-kleynhans/opencomp.git`.*