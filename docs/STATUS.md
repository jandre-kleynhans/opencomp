# OpenComp — Status

> Updated at end of every session. This file is the project's short-term memory.
> Note: `AGENTS.md` (agent bootstrap) is **gitignored** — it stays local-only by request.
> On a fresh clone, the bootstrap is: read this file + `docs/DEVLOG.md` + `docs/PLAN_PHASE1.md`.

## Current phase

**Phase 2 — Keyframe engine + Python expressions** — 🚧 IN PROGRESS (plan drafted)

Phase 1 complete: TOML project in → RGBA8 frame out. 27 tests green.
Phase 2 plan: `docs/PLAN_PHASE2.md` (10 tasks, TDD, tasks 1–3 ready to start).

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

1. **Phase 2 Task 1** — keyframe + expression model in `project.rs` (test-first: parse `[[layer.keyframe]]`, `AnimProp`, `Easing`, `KeyTime`, `AnimValue` int-or-float)
2. **Phase 2 Task 2** — timecode `"MM:SS:FF"` → frame conversion (`src/keyframe.rs`)
3. **Phase 2 Task 3** — easing curves (linear / ease_in / ease_out / ease_in_out / step), then Task 4 interpolation → `compositor.rs` time-awareness

## Blocked

- **GitHub Issues (Phase 2 Task 10)** — `gh` CLI not installed, no API token on this box. SSH push works (`github-opencomp` alias) but Issues needs API creds. Options: install `gh` + device-flow login (needs user at browser), or user provides PAT. Until then the plan doc is the tracker.

---
*Task 11 (GitHub public) ✅ done 2026-09-16: key `mothership-opencomp`, alias `github-opencomp`, remote `git@github-opencomp:jandre-kleynhans/opencomp.git`.*