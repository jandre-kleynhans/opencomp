# OpenComp — Status

> Updated at end of every session. This file is the project's short-term memory.
> Note: `AGENTS.md` (agent bootstrap) is **gitignored** — it stays local-only by request.
> On a fresh clone, the bootstrap is: read this file + `docs/DEVLOG.md` + `docs/PLAN_PHASE2.md`.

## Current phase

**Phase 2 — Keyframe engine + Python expressions** — 🚧 TASKS 1–9 DONE, 10 BLOCKED

Phase 1 complete (27 tests). Phase 2 implemented under TDD: keyframe model,
timecode, easing, interpolation, time-aware compositor, expression evaluator,
CLI `--frame`. **~51 tests green** (15 test binaries). `opencomp.exe` built for
Windows (cross-compiled) and bundled in `dist/win/`.

## Progress

- [x] Repo scaffold + docs + meta-system — 2026-09-16
- [x] Project model (serde + TOML parsing) — Task 2 — 2026-09-16
- [x] Color hex parsing (Task 3) — 2026-09-16
- [x] Background fill (Task 4) — 2026-09-16
- [x] Solid layer rasterization (Task 5) — 2026-09-16
- [x] Transforms: position/scale/opacity/rotation + layer `size` field (Task 6) — 2026-09-16
- [x] Layer stacking (Task 7) — built into Task 6 — 2026-09-16
- [x] PNG export (Task 8) — 2026-09-16
- [x] CLI `render` (Task 9) — 2026-09-16
- [x] Demo project + README (Task 10) — 2026-09-16
- [x] GitHub public (Task 11) ✅ — 2026-09-16
- [x] **Phase 2**: Keyframe + expression model (Task 1) — 2026-09-16
- [x] **Phase 2**: Timecode → frame (Task 2) — 2026-09-16
- [x] **Phase 2**: Easing curves (Task 3) — 2026-09-16
- [x] **Phase 2**: Interpolation + resolution (Task 4) — 2026-09-16
- [x] **Phase 2**: Time-aware compositor (Task 5) — 2026-09-16
- [x] **Phase 2**: Expression evaluator (Task 6) — 2026-09-16
- [x] **Phase 2**: CLI `--frame` (Task 7) — 2026-09-16
- [x] **Phase 2**: Animated demo + docs v0.2 (Task 8) — 2026-09-16
- [x] **Phase 2**: ADR-0002 + ADR-0003 (Task 9) — 2026-09-16
- [ ] **Phase 2**: GitHub Issues tracker (Task 10) — BLOCKED: no `gh`/API token

## Known issues

- None open.

## Next 3 actions

1. **Phase 2 Task 10** — GitHub Issues as tracker (needs `gh` install + user auth: device flow or PAT)
2. **Windows exe test** — user runs `dist/win/run_demo.bat` on Windows, gives feedback
3. **Phase 3 planning** — Python SDK + REST (agent surface)

## Blocked

- **GitHub Issues (Phase 2 Task 10)** — `gh` CLI not installed, no API token on this box. SSH push works (`github-opencomp` alias) but Issues needs API creds. Until then the plan doc is the tracker.
- **Windows exe run proof** — PC (Tailscale `desktop-c511tl4`, LAN 192.168.1.13) was offline when shipped; exe is a valid PE32+ (verified), same source passes 15 native test binaries.

---

*Phase 2 notes:*
- *Expressions: pure-Rust arithmetic evaluator (ADR-0003) — no rustpython dependency. Panics on invalid syntax (v0.2 convention).*
- *Timecode `"MM:SS:FF"` — FF = frames (not seconds).*
- *Step easing = hold left value until next key.*
- *Expression wins over keyframes on the same property.*
- *Dist: `dist/win/` — `opencomp.exe` (3.7MB, static), `demo.toml`, `run_demo.bat`.*