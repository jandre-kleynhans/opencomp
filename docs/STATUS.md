# OpenComp — Status

> Updated at end of every session. This file is the project's short-term memory.
> Note: `AGENTS.md` (agent bootstrap) is **gitignored** — it stays local-only by request.
> On a fresh clone, the bootstrap is: read this file + `docs/DEVLOG.md` + `docs/PLAN.md`.

## Current phase

**FULL PLAN COMPLETE** — Phases 1–6 implemented (Task 10 of Phase 2 blocked on user auth).

| Phase | What | Status |
|-------|------|--------|
| 1 | Compositing core (Rust, TOML→PNG) | ✅ done |
| 2 | Keyframe engine + expressions + easing | ✅ done (⊘ Task 10: GitHub Issues needs gh auth) |
| 3 | Python SDK + CLI + REST + WS | ✅ done |
| 4 | WASM plugin runtime | ✅ done (ABI + wasmtime + tests) |
| 5 | Web UI (canvas/timeline/layers) | ✅ done (ui/index.html) |
| 6 | Media I/O (render --all → mp4) | ✅ done |

## Progress

- [x] Phases 1–2 all tasks (see DEVLOG). ~51 Rust tests, 26 Python tests.
- [x] Phase 3: `python/opencomp/` SDK + `opencomp render|frame|preview|serve` + REST (`/project/load`, `/composition/layers`, `/layer/add`, `/layer/keyframe`, `/render/frame`, `/render/all`, `/project/diff`) + WS `/events`
- [x] Phase 4: `src/effect.rs` (wasmtime), linear-memory ABI (alloc/process/free), WAT invert plugin test — ADR-0004
- [x] Phase 5: `ui/index.html` (canvas viewport, layer list, timeline, keyframe edit, play/scrub, save TOML) — ADR-0005
- [x] Phase 6: `render --all` → ffmpeg h264 mp4 (verified 120f→5.0s 640×480) — ADR-0006

## Known issues

- None open.

## Next 3 actions

1. **Phase 2 Task 10** — GitHub Issues tracker: `gh` installed, needs `gh auth login` (device flow, user at browser) or a PAT
2. **User review** — UI on matrix (`C:\Users\Public\Desktop\opencomp_ui.html`) + `dist/win/` exe + `demo.mp4`
3. **Follow-ups** — Rust `serve` subcommand (from the CLI), video/image layer *decode* in core, codec flags (`--codec h264|hevc|prores|webm`)

## Blocked

- **GitHub Issues (Phase 2 Task 10)** — device-flow auth needs the user at a browser. `gh` CLI installed 2026-09-16.
- Video/image layer *decode* (not just parse) — Python SDK handles via ffmpeg/PIL; Rust core stores kind+source.

---

*Repo layout: `src/` (Rust core) · `python/opencomp/` (SDK+CLI+server) · `ui/index.html` (Phase 5 UI) · `docs/` (plan, format, ADRs) · `tests/`+`python/tests/` · `dist/win/` (Windows exe bundle). Delivery to matrix: ssh hermes@matrix → `C:\Users\Public\`.*