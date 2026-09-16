# OpenComp Architecture

> The design document. Everything here is the target; the repo implements it bottom-up.

## Principles

1. **The core is small.** Compositing, transforms, keyframes, blending. Everything else is a plugin.
2. **Text is the interchange format.** Projects are TOML. An agent with a text editor can do everything the GUI can.
3. **Agents are first-class.** The Python SDK and REST API are designed in from the start, not retrofitted.
4. **No proprietary anything.** WASM plugins, MIT license, open formats.
5. **Headless by default.** The render core never needs a window. UI is an optional thin layer.

## Layer Stack

```
┌─────────────────────────────────────┐
│         UI (TypeScript/Canvas)       │  optional — web-tech canvas timeline
├─────────────────────────────────────┤
│      Agent API (Python + REST)       │  PyPI package + REST server + WebSocket events
├─────────────────────────────────────┤
│      Plugin Runtime (WASM)           │  wasmtime sandbox; effects, generators, particle systems
├─────────────────────────────────────┤
│   Animation / Keyframe Engine        │  time-value curves, easing, Python expression evaluator
├─────────────────────────────────────┤
│   Compositing Engine (Rust/GPU)      │  layer stack → blended frame; wgpu abstraction (Vulkan/Metal/DX12)
├─────────────────────────────────────┤
│   Project Model (TOML)               │  serde-defined structs; git-diffable
└─────────────────────────────────────┘
```

### Compositing Engine (Rust)

- Owns the frame buffer: `Vec<u8>` RGBA8, width×height×4
- Consumes the resolved layer stack (already transformed & opacity-applied) and produces the final frame
- Bottom-up compositing: layer N is drawn over layers 0..N-1
- Blend modes operate per-pixel; Phase 1 ships `normal` only, the enum is ready for the rest (screen, multiply, add, ...)

### Project Model (TOML + serde)

```rust
struct Project { name: String, fps: u32, width: u32, height: u32, duration: u32, bg_color: Color, layers: Vec<Layer> }
struct Layer { name: String, kind: LayerKind, color: Option<Color>, transform: Transform, blend: BlendMode }
struct Transform { position: [f32; 2], scale: [f32; 2], opacity: f32, rotation: f32 }
```

The structs ARE the file format contract. `serde` gives us parse + serialize for free, and tests lock the contract.

## Agent API (future phases)

The property that makes OpenComp *the* agent compositor:

```python
from opencomp import Project, Layer, Keyframe

proj = Project.load("main.toml")
layer = Layer(name="spark", kind="solid", color="#ff8800")
layer.transform.position.animate([
    Keyframe(time=0, value=[100, 200]),
    Keyframe(time=1.5, value=[800, 400], easing="ease_in_out"),
])
proj.composition.add(layer)
proj.render_frame(42, output="preview_f042.png")
```

Plus a REST surface so agents (and humans) can drive a long-running engine:

```
POST /project/load · GET /composition/layers · POST /layer/add
POST /layer/keyframe · POST /render/frame?n=42 · POST /render/all
GET /project/diff?since=abc · WS /events
```

## Plugin Runtime (future phases)

WASM sandbox. Any language that compiles to WASM (Rust, C, Go, AssemblyScript) can ship an effect:

```rust
use opencomp_sdk::{Effect, FrameBuffer, Param};

pub struct BlurEffect { radius: Param<f32> }

impl Effect for BlurEffect {
    fn process(&self, input: &FrameBuffer, output: &mut FrameBuffer) { /* ... */ }
}
```

## Keyframe Engine (Phase 2)

Time-value pairs per property, interpolation between them, named easing curves, and a **Python** expression evaluator (replacing the ExtendScript-era `expression` language — Python is what agents actually write).

## Tech Choices

| Layer | Choice | Why |
|-------|--------|-----|
| Core | Rust | memory safety, no GC stalls, wgpu GPU access |
| Scripting | Python | agents know it, rich ecosystem |
| Plugins | WASM | sandboxed, portable, language-agnostic |
| UI | TypeScript/Canvas | web tech, easy to extend, runs anywhere |
| Project files | TOML | readable, diffable, commentable |
| Media I/O | ffmpeg/libav (Phase 6) | don't reinvent codecs |

## Existing OSS to stand on

- **wgpu** — Rust GPU abstraction (Vulkan/Metal/DX12/WebGPU)
- **ffmpeg** — media decode/encode
- **wasmtime** — WASM runtime
- **Natron** — reference for node compositor UX (aging, but prior art)