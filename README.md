# OpenComp

An open, minimal, agent-first compositing engine — a from-the-ground-up reimagining of what After Effects could be.

Built on one belief: **the compositing core is the product, and everything else is a plugin around it.**

## Why

Adobe After Effects is a C++ compositing powerhouse wrapped in layers of proprietary lock-in: binary project files, a dead scripting language (ExtendScript), no proper API, plugin SDKs that require signing, and a UI you can't drive headlessly.

OpenComp flips that:

- **Projects are plain text** (TOML) — git-diffable, agent-readable, human-readable
- **Scripting is first-class Python** — not a legacy JS bolt-on
- **Agent API built in from day one** — REST + event hooks, not UI automation
- **Plugins are WASM** — sandboxed, language-agnostic, no proprietary SDK
- **Rust core** — memory-safe, no GC pauses mid-render, GPU-first

## Architecture

```
┌─────────────────────────────────────┐
│         UI (TypeScript/Canvas)       │  ← optional, headless-usable
├─────────────────────────────────────┤
│      Agent API (Python + REST)       │  ← first-class, not bolted on
├─────────────────────────────────────┤
│      Plugin Runtime (WASM)           │  ← sandboxed effects
├─────────────────────────────────────┤
│   Animation / Keyframe Engine        │  ← interpolation, easing
├─────────────────────────────────────┤
│   Compositing Engine (Rust/GPU)      │  ← the core
├─────────────────────────────────────┤
│   Project Model (TOML)               │  ← version-controllable
└─────────────────────────────────────┘
```

See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for the full design, and [docs/ROADMAP.md](docs/ROADMAP.md) for where we're going.

## Status

**Phase 2 — Keyframe engine + Python expressions (in progress).** A Rust engine that reads a TOML project, animates layers with keyframes + easing, applies Python expressions, and renders PNG frames.

Current capability:
- TOML project parsing (composition + layers)
- Solid-color layers with static transforms (position, scale, opacity, rotation)
- **Per-property keyframes** with easing (linear, ease_in, ease_out, ease_in_out, step)
- Timecode `"MM:SS:FF"` or frame-int keyframe times
- **Python expressions** (`value`, `time`, `frame`, `fps`, `width`, `height`, `duration`)
- Normal alpha blending, bottom-up layer ordering
- PNG frame output, CLI `--frame N`

## Quickstart

```bash
cargo build --release
./target/release/opencomp render examples/demo.toml -f 0 -o out/frame_0000.png
./target/release/opencomp render examples/demo.toml -f 60 -o out/frame_0060.png
./target/release/opencomp render examples/demo.toml -f 120 -o out/frame_0120.png
```

## Roadmap

| Phase | What | Status |
|-------|------|--------|
| 1 | Rust compositing core: TOML project → PNG frame | ✅ done |
| 2 | Keyframe engine: interpolation, easing, Python expression evaluator | ✅ done |
| 3 | Python SDK + CLI + REST/WS agent surface | ✅ done |
| 4 | WASM plugin runtime (wasmtime, effect ABI) | ✅ done |
| 5 | Web UI: canvas viewport, timeline, layers | ✅ done |
| 6 | Media I/O: render --all → h264 mp4 via ffmpeg | ✅ done |

## Remote / GitHub

- **Public repo**: https://github.com/jandre-kleynhans/opencomp
- Remote: `git@github-opencomp:jandre-kleynhans/opencomp.git` (SSH alias in `~/.ssh/config`)

## License

MIT