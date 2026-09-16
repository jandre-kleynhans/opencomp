# OpenComp Roadmap

## Phase 1 — Compositing core 🔨 (in progress)

Goal: **a TOML project in, a PNG frame out.** Proves the foundation.

- [x] Repo scaffold + docs
- [ ] Project model parse (serde + toml)
- [ ] Solid layers, static transforms (position/scale/opacity/rotation)
- [ ] Normal alpha blending, bottom-up order, background color
- [ ] PNG export
- [ ] CLI: `opencomp render project.toml -o frame.png`
- [ ] GitHub public repo

Exit criteria: `examples/demo.toml` renders to a PNG whose pixels match the spec, verified by test AND by eye.

## Phase 2 — Keyframe engine

- Keyframes per property: `[[layer.keyframe]]` with `property`, `time`, `value`, `easing`
- Easing curves: linear, ease_in/out/in_out (cubic), step
- Python expression evaluator for properties (the AE expressions killer)

## Phase 3 — Python SDK + agent surface

- `opencomp` package: load, compose, render from Python
- CLI subcommands: `render`, `preview`, `frame -n 42`, `info`
- Project diff (`--since` git-style)

## Phase 4 — WASM plugin runtime

- wasmtime-backed effect sandbox
- Core effects as WASM: gaussian blur, color grade, text? (text is bigger — Phase 5)

## Phase 5 — Web UI

- Canvas viewport, layer list, timeline
- Headless-first: the UI talks to the same REST API an agent would

## Phase 6 — Media I/O

- ffmpeg decode (video/image sequences as layer sources)
- Encode: h264/h265/ProRes/WebM
- Audio track sync, render queue

---

*Order is deliberate: agents get a working engine before humans get a pretty UI.*