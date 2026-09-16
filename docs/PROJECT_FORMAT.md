# OpenComp Project Format — v0.2 spec

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
type    = "solid"          # v0.2: "solid" only
color   = "#1a2b3c"        # required for solid
blend   = "normal"         # optional, default "normal"
size    = [200, 100]        # optional, [w, h] pixels; [0,0] = full canvas

[layer.transform]          # optional; all fields default
position = [0.0, 0.0]      # px from top-left, center of layer footprint
scale    = [1.0, 1.0]      # multiplier on layer size
opacity  = 100.0           # 0.0–100.0
rotation = 0.0             # degrees clockwise
```

## Keyframes (v0.2)

Per-property keyframes override the static `[layer.transform]` at the specified
frame. Multiple `[[layer.keyframe]]` entries can target the same property.

```toml
[[layer.keyframe]]
property = "position"         # "position" | "scale" | "opacity" | "rotation"
time     = 0                  # integer (frame number) or "MM:SS:FF" timecode
value    = [100.0, 200.0]     # [x, y] for position/scale; scalar for opacity/rotation
easing   = "ease_in_out"      # optional, default "linear"
```

**Easing names:** `linear`, `ease_in`, `ease_out`, `ease_in_out`, `step`

**Extrapolation:** before the first key and after the last key, the value is
held constant (no motion, no fade).

**Step easing:** holds the left key's value until the next key's time.

**Timecode format:** `"MM:SS:FF"` — MM = minutes, SS = seconds, FF = frames.
Resolved via project `fps`. Example: `"00:01:12"` at 24 fps = 1 second + 12
frames = 36.

## Expressions (v0.2)

Arithmetic expressions that override keyframes on the same property.
If both keyframes and an expression target a property, the **expression wins**.

```toml
[[layer.expression]]
property = "rotation"
expr     = "frame * 2.0"       # rotate 2° per frame
```

**Available variables:**

| Name | Type | Description |
|------|------|-------------|
| `value` | f32 | The property's static value (or keyframe-resolved value if keyframes exist) |
| `frame` | u32 | Current frame number (0-based) |
| `time` | f32 | Current time in seconds (`frame / fps`) |
| `fps` | u32 | Project frames per second |
| `width` | u32 | Project canvas width |
| `height` | u32 | Project canvas height |
| `duration` | u32 | Total frames in the project |

**Supported operators:** `+`, `-`, `*`, `/`, parentheses `()`.

**Vector expressions:** `" [expr_x, expr_y] "` — the `[0]` and `[1]` elements
of `value` can be accessed via `value[0]` and `value[1]`; bare `value` in an
x-expression refers to the x component, in the y-expression to y.

**Errors:** invalid syntax or type errors panic with a clear message (v0.2;
proper `Result` error type deferred to Phase 3).

## CLI

```bash
opencomp render project.toml                     # render frame 0
opencomp render project.toml -f 42               # render frame 42
opencomp render project.toml -f 42 -o out.png    # custom output path
```

`-f` / `--frame`: 0-based frame number (default: 0).

## Conventions

- **Times** are frame numbers (0-based) or `"MM:SS:FF"` timecodes.
- **Colors** are hex strings: `#RRGGBB` or `#RRGGBBAA`.
- **Angles** in degrees (agents read degrees more easily than radians).
- Omitted fields take defaults — a minimal 1-layer project is 12 lines.

## Reference implementation

The serde structs in `src/project.rs` are the source of truth. Changes to the
format must land there with a test first (see `tests/`).
