# OpenComp — Development Log

> Chronological session records. NEWEST LAST.

## [2026-09-16] Session 2 — Phase 1 complete

### Tasks completed (all TDD, verified by test AND pixel math)

**Task 3 — Color hex parsing** (lock contract)
- 7 edge-case tests: RGB, RGBA, lowercase, no-hash-prefix, invalid-length panic, invalid-digit panic, roundtrip.

**Task 4 — Background fill** + compositor.rs
- `Frame` struct, `render_frame()`, opaque background fill. Fixed: bg always forces alpha=255 (background is a solid fill, not a layer).

**Task 5 — Solid layer rasterization**
- Normal alpha blending: `out = src*α + dst*(1-α)`, 50% opacity verified: (128,0,0,255) over black.

**Task 6 — Transforms** (position, scale, opacity, rotation)
- Added `size: [f32; 2]` to `Layer` format ([0,0] = full canvas). Inverse-transform compositing: per-pixel un-rotate and check footprint bounds.
- Design decision (documented in format): `position` is the layer's center for sized layers; for full-canvas layers position is ignored (center is canvas center).

**Task 8 — PNG export** (`write_png`, `read_png`)
- PNG RGBA8 via the `png` crate. Roundtrip test proves pixel fidelity.

**Task 9 — CLI `render` command**
- `opencomp render project.toml -o out.png`, clap-based. Missing-file → non-zero exit.

**Task 10 — Demo project**
- `examples/demo.toml`: 640×480, four layers (backdrop, left card, rotated right card, semi-transparent strip). Pixel sampling verified compositing + rotation + alpha blending.

### Issues found and fixed during development
- **Raw strings + rustfmt**: `r#"` terminated by `#` in hex colors → switched to `r##"` delimiters.
- **serde `flatten` + defaults**: `#[serde(flatten)]` on `Transform` broke `Default` → switched to nested `[layer.transform]` with explicit serde defaults.
- **Layer test missing `[[layer]]`**: old test's TOML helper forgot the header → duplicate key panic → added to helper.
- **Full-canvas centering**: position offset added to already-centered footprint pushed it off-canvas → full-canvas layers now ignore position (center at canvas center).

### Test summary
27 tests passing: 3 lib + 7 color + 3 compositor + 2 layer + 3 transform + 5 transform-stacked + 2 export + 2 CLI.

### Git history (this session)
```
2a368ad feat: demo project + compositor centering fix; Phase 1 complete
88a6f46 feat: CLI render command (Task 9)
a655603 feat: PNG export + read roundtrip (Task 8)
4bd4387 feat: transforms (position/scale/opacity/rotation) + layer size field
843a5b5 feat: solid layer rasterization with alpha blending
d261a17 feat: background fill + compositor Frame
88dbc6c test: lock color hex parsing contract (7 edge-case tests)
```

### Next
- Task 11: GitHub remote (blocked on user auth)
- Phase 2: keyframe engine (time-value pairs, easing, Python expression evaluator)
- ADR-0002: full-canvas layer centering convention