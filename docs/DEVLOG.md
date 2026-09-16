# OpenComp — Development Log

> Chronological session records. NEWEST LAST.

## [2026-09-16] Session 3 — Phase 2 started (plan drafted)

### What changed
- **`docs/PLAN_PHASE2.md`**: 10-task TDD plan for the keyframe engine:
  1. Keyframe + expression model in `project.rs` (`AnimProp`, `Easing`, `KeyTime` (frame int or `"MM:SS:FF"`), `AnimValue` (number or vec2, int-or-float), `[[layer.keyframe]]` + `[[layer.expression]]` tables)
  2. Timecode → frame conversion (`src/keyframe.rs`)
  3. Easing curves: linear, ease_in/out/in_out (cubic), step
  4. Interpolation + per-property resolution (hold extrapolation, left-key easing governs segment)
  5. Time-aware compositor (`render_frame` resolves transform per frame; pixel-verified)
  6. Python expression evaluator (rustpython-vm; expression overrides keyframes; vars: value/time/frame/fps/width/height/duration)
  7. CLI `-f/--frame N`
  8. Animated demo + PROJECT_FORMAT v0.2 + README
  9. ADR-0002 (full-canvas centering convention) + ADR-0003 (Python expressions)
  10. GitHub Issues as tracker (BLOCKED: no `gh`/API token)
- **`docs/STATUS.md`**: phase → Phase 2 in progress; next-3-actions + blockers updated.
- No source code changed — the session's deliverable is the plan (docs-first; STATUS's next-3-actions explicitly gated Phase 2 on the plan).

### Design decisions locked in the plan
- Interpolation in **integer frame space** (no sub-frame until real-time needs it)
- **Hold extrapolation** before first / after last key
- **`step` easing** = hold left value until next key (AE-style, matches "all = queued" honesty of UI states)
- **Expression wins** when both keyframes and an expression target the same property
- Eval/parse errors **panic with a clear message** in v0.2 (matches `Color::from_hex` contract); typed `Result` error path is Phase 3
- `rustpython-vm` (pure-Rust, in-process, no C toolchain) + fallback to arithmetic-only if stdlib/extract misbehaves

### Test targets (next session)
- `cargo test` (27 existing green); new: `tests/keyframe_test.rs` (6), `anim_time_test.rs` (4), `easing_test.rs` (6), `keyframe_eval_test.rs` (9), `anim_composite_test.rs` (3), `expression_test.rs` (8), `cli_test.rs` (+2) ≈ 38 new

### Next session
- Start Task 1: RED test for `[[layer.keyframe]]` parse → GREEN in `project.rs`
- Task 2–5 straight after (model → time → easing → interp → compositor)
- Task 10 (GitHub Issues): needs `gh` install + user auth (device flow or PAT); SSH push already works

### Git history (this session)
```
(committed after this entry)
```

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