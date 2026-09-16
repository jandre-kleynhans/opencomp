# OpenComp — Master Plan (AGREED — read this before any work)

> **This is the plan the owner and agents agreed on. It is not advisory. It is the**
> **contract. Every session, on every machine, any agent working on OpenComp must**
> **read this file first and follow it exactly. Do not invent phases. Do not jump**
> **ahead. If the owner asks for something outside the current phase, point to the**
> **phase it belongs to and keep the order.**

## The vision (why this exists)

OpenComp is an **open, minimal, agent-first compositing engine** — a from-the-ground-up
reimagining of After Effects. The compositing core is the product; everything else is
a plugin around it. Agents are first-class users.

## Language choices

| Layer | Choice | Why |
|-------|--------|-----|
| Core | Rust | performance, memory safety, no GC pauses mid-render |
| Scripting | Python | universal, LLMs know it cold, rich ecosystem |
| Plugins | WASM | language-agnostic, sandboxed, portable |
| UI | TypeScript/Canvas | web-based canvas, easy to extend |
| Project files | TOML | git-friendly, agent-readable, human-readable |

## Architecture layers

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
│   Project Model (JSON/TOML)          │  ← version-controllable
└─────────────────────────────────────┘
```

## THE ROADMAP (in order — no skipping, no inventing)

### Phase 1 — Compositing core ✅ DONE
Layer stack, transforms, blending modes, PNG/EXR I/O. The engine that takes a TOML file
and produces a frame.

### Phase 2 — Keyframe engine 🔨 CURRENT
Interpolation curves, easing functions, expression evaluator (**Python-based**, not
ExtendScript).

### Phase 3 — Python SDK + CLI
`opencomp render`, `opencomp preview`, `opencomp frame --n=42`. This is where it becomes
agent-usable. **The Python SDK + REST API is the key differentiator — the thing that
makes OpenComp *the* agent compositor.**

### Phase 4 — WASM plugin runtime
Blur, color correct, particles as sandboxed plugins.

### Phase 5 — Web UI (optional)
Canvas-based timeline, layer list, viewport. Runs headless for agents, visual for humans.

### Phase 6 — Media I/O
Video decode (ffmpeg/libav), audio sync, h264/h265/ProRes output.

## Why this beats AE for agent workflows

| AE | OpenComp |
|----|----------|
| ExtendScript (dead 1998 JS) | Python — first-class, full library |
| No API, drive UI clicks | REST API + Python SDK |
| Binary project files | Plain TOML — git, diff, patch |
| Single-threaded CPU render | GPU-first, parallel |
| Plugins need C++ SDK + signing | WASM — anyone can build |
| Can't run headless | Fully headless by default |
| Random crashes, memory leaks | Rust — no GC, no segfaults |
| No version control | Project files are text |

## The agent API (the key differentiator — Phase 3)

```python
from opencomp import Project, Composition, Layer, Keyframe

proj = Project.load("main.toml")
comp = proj.composition
layer = Layer(name="spark", type="particle", source="core:particle_fire")
layer.transform.position.animate([
    Keyframe(time=0, value=[100, 200]),
    Keyframe(time=1.5, value=[800, 400], easing="ease_in_out"),
])
comp.add(layer)
comp.render_frame(42, output="preview_f042.png")  # render a frame
comp.render(output="final.mp4", codec="h264", quality="high")  # full render
```

REST surface (Phase 3):
```
POST /project/load · GET /composition/layers · POST /layer/add
POST /layer/keyframe · POST /render/frame?n=42 · POST /render/all
GET /project/diff?since=abc · WS /events
```

## Hard rules for any agent session

1. **Follow the plan.** Do not invent phases. Do not jump ahead.
2. **TDD** — production code only after a failing test.
3. **Docs-first** — architecture changes land in `docs/ARCHITECTURE.md`; decisions get an ADR.
4. **Conventional commits** — `feat:` `fix:` `docs:` `chore:` `refactor:` `test:`.
5. **The repo is the memory** — read AGENTS.md / STATUS.md / DEVLOG.md / ADRs first.
6. **Session records** — DEVLOG entry + STATUS refresh at end of every session.
7. **Windows delivery** — test builds go to `C:\Users\Public\` on matrix (ssh hermes@matrix).

## Current status

See `docs/STATUS.md` — the live short-term memory. This file is the immutable plan.