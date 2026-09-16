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

**Phase 1 — Compositing core (in progress).** A Rust engine that reads a TOML project and renders frames. No UI, no keyframes, no plugins — just the foundation proven end-to-end.

Current capability:
- TOML project parsing (composition + layers)
- Solid-color layers with static transforms (position, scale, opacity, rotation)
- Normal alpha blending, bottom-up layer ordering
- PNG frame output

## Quickstart

```bash
cargo build --release
./target/release/opencomp render examples/demo.toml -o out/frame_0000.png
```

## Roadmap

| Phase | What | Status |
|-------|------|--------|
| 1 | Rust compositing core: TOML project → PNG frame | 🔨 in progress |
| 2 | Keyframe engine: interpolation, easing, Python expression evaluator | pending |
| 3 | Python SDK + CLI (`opencomp render/preview/frame`) | pending |
| 4 | WASM plugin runtime: blur, color, particles as sandboxed effects | pending |
| 5 | Web UI (optional, headless-first) | pending |
| 6 | Media I/O: video decode/encode via ffmpeg, audio sync | pending |

## Remote / GitHub

- **Public repo**: https://github.com/jandre-kleynhans/opencomp
- Remote: `git@github-opencomp:jandre-kleynhans/opencomp.git` (SSH alias in `~/.ssh/config`)

## License

MIT