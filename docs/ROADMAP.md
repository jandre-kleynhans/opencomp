# OpenComp Roadmap

## Phase 1 — Compositing core ✅ (done 2026-09-16)

Goal: **a TOML project in, a PNG frame out.** Proves the foundation.

- [x] Repo scaffold + docs
- [x] Project model parse (serde + toml)
- [x] Solid layers, static transforms (position/scale/opacity/rotation)
- [x] Normal alpha blending, bottom-up order, background color
- [x] PNG export
- [x] CLI: `opencomp render project.toml -o frame.png`
- [x] GitHub public repo

Exit criteria: `examples/demo.toml` renders to a PNG whose pixels match the spec, verified by test AND by eye.

## Phase 2 — Keyframe engine ✅ (done 2026-09-16; ⊘ Task 10 GitHub Issues needs user auth)

- [x] Keyframes per property: `[[layer.keyframe]]` with `property`, `time`, `value`, `easing`
- [x] Easing curves: linear, ease_in/out/in_out (cubic), step
- [x] Python expression evaluator for properties (arithmetic evaluator — ADR-0003)
- [ ] GitHub Issues as issue tracker (blocked: needs `gh auth login` at user browser)

## Phase 3 — Python SDK + agent surface ✅ (done 2026-09-16)

- [x] `opencomp` package: load, compose, render from Python
- [x] CLI subcommands: `render`, `preview`, `frame -n 42`, `serve`
- [x] Project diff (`--since` git-style) via REST `/project/diff`

## Phase 4 — WASM plugin runtime ✅ (done 2026-09-16)

- [x] wasmtime-backed effect sandbox (linear-memory ABI — ADR-0004)
- [x] Core effects as WASM: invert (WAT test module); blur/color-grade/particles = follow-up plugins behind same ABI

## Phase 5 — Web UI ✅ (done 2026-09-16)

- [x] Canvas viewport, layer list, timeline (ui/index.html — talks to REST)
- [x] Headless-first: the UI talks to the same REST API an agent would

## Phase 6 — Media I/O ✅ (done 2026-09-16)

- [x] Encode: `render --all -o out.mp4` via ffmpeg h264 (ADR-0006)
- [x] video/image layer kinds in the format + Python SDK parse
- [ ] Decode of video/image layers in the Rust core (Python SDK handles via ffmpeg/PIL for now)

---

*Order is deliberate: agents get a working engine before humans get a pretty UI.*