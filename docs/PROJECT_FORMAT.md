# OpenComp Project Format — v0.1 spec

Projects are TOML. Human-readable, commentable, git-diffable, agent-editable.

## Top-level

```toml
[project]
name     = "demo"          # required, string
fps      = 24              # required, frames per second
width    = 640             # required, pixels
height   = 480             # required, pixels
duration = 120             # required, total frames
bg_color = "#0a0a0a"       # required, hex RGB or RGBA
```

## Layers

`[[layer]]` tables, composited **bottom-up** (first layer = furthest back).

```toml
[[layer]]
name    = "backdrop"       # required, unique
type    = "solid"          # v0.1: "solid" only
color   = "#1a2b3c"        # required for solid
blend   = "normal"         # optional, default "normal"; v0.1 supports only "normal"

[layer.transform]          # optional; all fields default
position = [0.0, 0.0]      # px from top-left, center of layer footprint
scale    = [1.0, 1.0]      # multiplier on layer size
opacity  = 100.0           # 0.0–100.0
rotation = 0.0             # degrees clockwise
```

## Conventions

- **Times** are frame numbers (0-based) in v0.1. Phase 2 keyframes add `time` as `"MM:SS:FF"` or frame int.
- **Colors** are hex strings: `#RRGGBB` or `#RRGGBBAA`.
- **Angles** in degrees (agents read degrees more easily than radians).
- Omitted fields take defaults — a minimal 1-layer project is 12 lines.

## Reference implementation

The serde structs in `src/project.rs` are the source of truth. Changes to the format must land there with a test first (see `tests/`).