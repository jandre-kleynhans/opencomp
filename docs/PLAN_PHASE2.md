# Phase 2: Keyframe Engine + Python Expressions — Implementation Plan

> **For Hermes:** Use `test-driven-development` (RED → GREEN per task, verify failure before
> implementation, commit after each task). Full context: `docs/ARCHITECTURE.md`
> (§ Keyframe Engine), `docs/PROJECT_FORMAT.md` (v0.1 → v0.2 in Task 8), `docs/ROADMAP.md`.
> Current suite: 27 tests green, Phase 1 complete.

**Goal:** Layers animate. Per-property keyframes with easing curves (linear, cubic
in/out/in-out, step), `"MM:SS:FF"` or frame-int times, and a Python expression
evaluator — the AE-expressions killer. `opencomp render` gains `--frame N`.

**Architecture:** New `src/keyframe.rs` resolves any layer's transform at a given
frame from `[[layer.keyframe]]` / `[[layer.expression]]` tables; `compositor.rs`
becomes time-aware by consuming the resolved transform; new `src/expression.rs`
wraps an in-process Python interpreter (rustpython-vm). The TOML format contract
extends: `Layer` gains `keyframes: Vec<Keyframe>` + `expressions: Vec<Expression>`.

**Tech stack:** Rust (edition 2021). New dep: `rustpython-vm` (in-process Python;
pure-Rust, no C toolchain, matches the "Python is what agents write" principle).
Existing: serde / toml / png / clap.

---

## Design decisions (locked here, ADR-0003 in Task 9)

1. **Times**: keyframe `time` is either a frame int (`0`) or a timecode string
   `"MM:SS:FF"` resolved via project `fps` → frame int. Interpolation happens in
   **integer frame space** (no sub-frame until real-time needs it).
2. **Values**: `value` is a TOML float (`opacity`, `rotation`) or 2-float array
   (`position`, `scale`). One runtime enum `AnimValue { Number(f32), Vec2([f32; 2]) }`
   with a custom serde deserializer accepting int or float scalars.
3. **Easing** (per-keyframe, default `linear`; the *left* key's easing governs the
   segment, AE-style): `linear` (t), `ease_in` (t³), `ease_out` (1−(1−t)³),
   `ease_in_out` (3t²−2t³ smoothstep), `step` (hold — left value until the next key).
4. **Extrapolation**: before the first key and after the last key → hold constant.
5. **Expressions** (Task 6): a property may carry `[[layer.expression]]` with a
   Python expression string. If both keyframes and an expression target the same
   property, the **expression wins**. Available names: `value` (static transform
   value, or keyframe-resolved value if keyframes exist), `time` (seconds, float),
   `frame` (int), `fps`, `width`, `height`, `duration` (frames). Scalar properties
   get a Python number; vector properties get a 2-element list. Result coerced back
   to the property type (int → f32 accepted). Eval/parse errors **panic with a
   clear message** in v0.2 (proper `Result` error type is a later phase).
6. **Full-canvas layers**: position is ignored for full-canvas layers (Phase 1
   convention, ADR-0002 in Task 9) — so keyframing position of a full-canvas layer
   is a no-op by design. Sized layers animate normally.

---

## Task 1: Keyframe + expression model in project.rs (RED first)

**Files:**
- Modify: `src/project.rs` (new types + `Layer` fields)
- Create: `tests/keyframe_test.rs` (parse half of it; eval tests come in Task 4)

**Step 1 — failing tests** (`tests/keyframe_test.rs`):

```rust
use opencomp::project::{Project, AnimValue, AnimProp, Easing, KeyTime};

fn parse(src: &str) -> Project {
    toml::from_str(src).unwrap()
}

const MIN: &str = r#"
    [project]
    name = "demo"
    fps = 24
    width = 640
    height = 480
    duration = 120
    bg_color = "#000000"
"#;

#[test]
fn parses_keyframe_with_frame_int_time() {
    let proj = parse(&format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        size = [80.0, 80.0]
        [[layer.keyframe]]
        property = "position"
        time = 10
        value = [100.0, 200.0]
    "#));
    let kf = &proj.layers[0].keyframes[0];
    assert_eq!(kf.property, AnimProp::Position);
    assert_eq!(kf.time, KeyTime::Frame(10));
    assert_eq!(kf.value, AnimValue::Vec2([100.0, 200.0]));
    assert_eq!(kf.easing, Easing::Linear); // default
}

#[test]
fn parses_keyframe_with_timecode_string() {
    let proj = parse(&format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.keyframe]]
        property = "opacity"
        time = "00:01:00"
        value = 50.0
        easing = "ease_in_out"
    "#));
    let kf = &proj.layers[0].keyframes[0];
    assert_eq!(kf.time, KeyTime::Tc("00:01:00".to_string()));
    assert_eq!(kf.value, AnimValue::Number(50.0));
    assert_eq!(kf.easing, Easing::EaseInOut);
}

#[test]
fn parses_int_scalar_value() {
    // TOML `value = 50` (integer) must deserialize to Number(50.0)
    let proj = parse(&format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.keyframe]]
        property = "opacity"
        time = 0
        value = 50
    "#));
    assert_eq!(proj.layers[0].keyframes[0].value, AnimValue::Number(50.0));
}

#[test]
fn parses_all_easing_names() {
    let src = format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.keyframe]]
        property = "rotation"
        time = 0
        value = 0.0
        easing = "step"
    "#);
    assert_eq!(parse(&src).layers[0].keyframes[0].easing, Easing::Step);
    // sanity: the four curve names all parse via serde rename_all = "lowercase"
    for (name, want) in [("linear", Easing::Linear), ("ease_in", Easing::EaseIn),
                         ("ease_out", Easing::EaseOut), ("ease_in_out", Easing::EaseInOut),
                         ("step", Easing::Step)] {
        let src = format!(r#"{MIN}
            [[layer]]
            name = "card"
            type = "solid"
            color = "#ff0000"
            [[layer.keyframe]]
            property = "rotation"
            time = 0
            value = 0.0
            easing = "{name}"
        "#);
        assert_eq!(parse(&src).layers[0].keyframes[0].easing, want, "easing {name}");
    }
}

#[test]
fn parses_expression_table() {
    let proj = parse(&format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.expression]]
        property = "rotation"
        expr = "frame * 2.0"
    "#));
    let e = &proj.layers[0].expressions[0];
    assert_eq!(e.property, AnimProp::Rotation);
    assert_eq!(e.expr, "frame * 2.0");
}

#[test]
fn layer_without_keyframes_defaults_empty() {
    let proj = parse(MIN);
    assert!(proj.layers.is_empty()); // no layers at all
    let proj = parse(&format!(r#"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
    "#));
    assert!(proj.layers[0].keyframes.is_empty());
    assert!(proj.layers[0].expressions.is_empty());
}
```

**Step 2 — verify fail:** `cargo test --test keyframe_test` → compile error
(`no keyframes field / no AnimProp`). Expected RED.

**Step 3 — implement.** New types in `src/project.rs` (before `Layer`):

```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimProp { Position, Scale, Opacity, Rotation }

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Easing { Linear, EaseIn, EaseOut, EaseInOut, Step }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyTime { Frame(u32), Tc(String) }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AnimValue { Number(f32), Vec2([f32; 2]) }

// Custom deserializer: TOML int OR float → Number(f32); 2-array → Vec2.
// (Untagged f32 alone rejects bare ints — the test above locks the accept-both contract.)
fn anim_value_deser<'de, D>(d: D) -> Result<AnimValue, D::Error>
where D: serde::Deserializer<'de> {
    let v = toml::Value::deserialize(d)?;
    match v {
        toml::Value::Integer(i) => Ok(AnimValue::Number(i as f32)),
        toml::Value::Float(f)   => Ok(AnimValue::Number(f as f32)),
        toml::Value::Array(a) if a.len() == 2 => {
            let mut out = [0.0f32; 2];
            for (i, item) in a.into_iter().enumerate() {
                out[i] = match item {
                    toml::Value::Integer(x) => x as f32,
                    toml::Value::Float(x)   => x,
                    _ => return Err(serde::de::Error::custom("keyframe value must be a number or [x, y]")),
                };
            }
            Ok(AnimValue::Vec2(out))
        }
        _ => Err(serde::de::Error::custom("keyframe value must be a number or [x, y]")),
    }
}

fn default_easing() -> Easing { Easing::Linear }

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub property: AnimProp,
    pub time: KeyTime,
    #[serde(deserialize_with = "anim_value_deser")]
    pub value: AnimValue,
    #[serde(default = "default_easing")]
    pub easing: Easing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    pub property: AnimProp,
    pub expr: String,
}
```

Then on `Layer`:

```rust
#[serde(rename = "keyframe", default)]
pub keyframes: Vec<Keyframe>,
#[serde(rename = "expression", default)]
pub expressions: Vec<Expression>,
```

**Step 4 — verify pass:** `cargo test --test keyframe_test` → 6 passed.

**Step 5 — commit:** `feat: keyframe + expression model in project format`

---

## Task 2: Timecode → frame conversion (RED first)

**Files:**
- Create: `src/keyframe.rs` (time conversion + easing; interpolation lands Task 4)
- Modify: `src/lib.rs` (`pub mod keyframe;`)
- Create: `tests/anim_time_test.rs`

**Step 1 — failing tests:**

```rust
use opencomp::keyframe::time_to_frame;
use opencomp::project::KeyTime;

#[test]
fn frame_int_passes_through() {
    assert_eq!(time_to_frame(&KeyTime::Frame(42), 24), 42);
    assert_eq!(time_to_frame(&KeyTime::Frame(0), 60), 0);
}

#[test]
fn timecode_converts_via_fps() {
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:00".into()), 24), 0);
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:01".into()), 24), 24);
    assert_eq!(time_to_frame(&KeyTime::Tc("00:01:00".into()), 24), 24 * 60);
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:15".into()), 60), 15);
    assert_eq!(time_to_frame(&KeyTime::Tc("01:30:15".into()), 24), 90 * 24 + 15);
}

#[test]
#[should_panic(expected = "timecode")]
fn malformed_timecode_panics() {
    let _ = time_to_frame(&KeyTime::Tc("abc".into()), 24);
}

#[test]
#[should_panic(expected = "timecode")]
fn wrong_timecode_arity_panics() {
    let _ = time_to_frame(&KeyTime::Tc("1:2".into()), 24);
}
```

**Step 2 — verify fail:** `cargo test --test anim_time_test` → compile error
(no `opencomp::keyframe`). RED.

**Step 3 — implement** `src/keyframe.rs`:

```rust
use crate::project::KeyTime;

/// Resolve a keyframe time to a frame number. Timecode is "MM:SS:FF" resolved via fps.
pub fn time_to_frame(t: &KeyTime, fps: u32) -> u32 {
    match t {
        KeyTime::Frame(f) => *f,
        KeyTime::Tc(s) => {
            let parts: Vec<&str> = s.split(':').collect();
            assert!(parts.len() == 3, "timecode must be MM:SS:FF, got '{s}'");
            let nums: Result<Vec<u32>, _> = parts.iter().map(|p| p.parse::<u32>()).collect();
            let nums = nums.unwrap_or_else(|_| panic!("timecode must be MM:SS:FF, got '{s}'"));
            nums[0] * 60 * fps + nums[1] * fps + nums[2]
        }
    }
}
```

**Step 4 — verify pass:** 4 passed. **Step 5 — commit:** `feat: MM:SS:FF timecode parsing`

---

## Task 3: Easing curves (RED first)

**Files:** modify `src/keyframe.rs`, extend `tests/anim_time_test.rs`
(rename mentally to `anim_test.rs` — or create `tests/easing_test.rs`; either is fine, keep names explicit).

**Step 1 — failing tests** (`tests/easing_test.rs`):

```rust
use opencomp::keyframe::ease;
use opencomp::project::Easing;

const EPS: f32 = 1e-4;

#[test]
fn linear_is_identity() {
    for t in [0.0, 0.25, 0.5, 0.75, 1.0] {
        assert!((ease(Easing::Linear, t) - t).abs() < EPS);
    }
}

#[test]
fn ease_in_is_cubic() {
    assert!((ease(Easing::EaseIn, 0.5) - 0.125).abs() < EPS); // 0.5^3
    assert_eq!(ease(Easing::EaseIn, 0.0), 0.0);
    assert_eq!(ease(Easing::EaseIn, 1.0), 1.0);
}

#[test]
fn ease_out_is_inverse_cubic() {
    assert!((ease(Easing::EaseOut, 0.5) - 0.875).abs() < EPS); // 1 - 0.5^3
    assert_eq!(ease(Easing::EaseOut, 0.0), 0.0);
    assert_eq!(ease(Easing::EaseOut, 1.0), 1.0);
}

#[test]
fn ease_in_out_is_smoothstep() {
    assert!((ease(Easing::EaseInOut, 0.5) - 0.5).abs() < EPS); // 3t^2-2t^3
    assert_eq!(ease(Easing::EaseInOut, 0.0), 0.0);
    assert_eq!(ease(Easing::EaseInOut, 1.0), 1.0);
    // symmetric: f(t) = 1 - f(1-t)
    for t in [0.1, 0.3, 0.7, 0.9] {
        assert!((ease(Easing::EaseInOut, t) + ease(Easing::EaseInOut, 1.0 - t) - 1.0).abs() < EPS);
    }
}

#[test]
fn input_is_clamped() {
    assert_eq!(ease(Easing::EaseIn, 2.0), 1.0);
    assert_eq!(ease(Easing::Linear, -1.0), 0.0);
}

#[test]
fn step_returns_left_hold_marker() {
    // step is handled as a hold in interpolation; expose 0.0 so it's never used as a curve
    assert_eq!(ease(Easing::Step, 0.5), 0.0);
}
```

**Step 2 — verify fail:** `cargo test --test easing_test` → compile error (no `ease`). RED.

**Step 3 — implement:**

```rust
/// Apply an easing curve to a 0..1 progress value (clamped). `Step` is a hold:
/// interpolation must not move — see `interp_value`.
pub fn ease(e: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match e {
        Easing::Linear    => t,
        Easing::EaseIn    => t * t * t,
        Easing::EaseOut   => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => 3.0 * t * t - 2.0 * t * t * t,
        Easing::Step      => 0.0,
    }
}
```

**Step 4 — verify pass:** 6 passed. **Step 5 — commit:** `feat: easing curves (linear, cubic in/out/in-out, step)`

---

## Task 4: Interpolation + per-property resolution (RED first)

**Files:** modify `src/keyframe.rs`, create `tests/keyframe_eval_test.rs`.

**Step 1 — failing tests:**

```rust
use opencomp::keyframe::{resolve_value, AnimCtx};
use opencomp::project::{AnimProp, AnimValue, Easing, KeyTime, Keyframe};

fn kf(prop: AnimProp, time: u32, value: AnimValue, easing: Easing) -> Keyframe {
    Keyframe { property: prop, time: KeyTime::Frame(time), value, easing }
}

#[test]
fn no_keyframes_returns_static() {
    let v = resolve_value(&[], AnimProp::Opacity, 42, 24, AnimValue::Number(80.0));
    assert_eq!(v, AnimValue::Number(80.0));
}

#[test]
fn single_keyframe_holds_everywhere() {
    let keys = vec![kf(AnimProp::Opacity, 10, AnimValue::Number(50.0), Easing::Linear)];
    for f in [0, 5, 10, 999] {
        assert_eq!(resolve_value(&keys, AnimProp::Opacity, f, 24, AnimValue::Number(0.0)),
                   AnimValue::Number(50.0));
    }
}

#[test]
fn before_first_and_after_last_hold() {
    let keys = vec![
        kf(AnimProp::Position, 10, AnimValue::Vec2([100.0, 200.0]), Easing::Linear),
        kf(AnimProp::Position, 20, AnimValue::Vec2([300.0, 400.0]), Easing::Linear),
    ];
    assert_eq!(resolve_value(&keys, AnimProp::Position, 0, 24, AnimValue::Vec2([0.0, 0.0])),
               AnimValue::Vec2([100.0, 200.0]));
    assert_eq!(resolve_value(&keys, AnimProp::Position, 30, 24, AnimValue::Vec2([0.0, 0.0])),
               AnimValue::Vec2([300.0, 400.0]));
}

#[test]
fn linear_midpoint_interpolates_scalar() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(AnimProp::Opacity, 10, AnimValue::Number(100.0), Easing::Linear),
    ];
    assert_eq!(resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)),
               AnimValue::Number(50.0));
}

#[test]
fn linear_midpoint_interpolates_vec2() {
    let keys = vec![
        kf(AnimProp::Position, 0, AnimValue::Vec2([0.0, 0.0]), Easing::Linear),
        kf(AnimProp::Position, 10, AnimValue::Vec2([100.0, 200.0]), Easing::Linear),
    ];
    assert_eq!(resolve_value(&keys, AnimProp::Position, 5, 24, AnimValue::Vec2([0.0, 0.0])),
               AnimValue::Vec2([50.0, 100.0]));
}

#[test]
fn ease_in_curves_the_progress() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::EaseIn),
        kf(AnimProp::Opacity, 10, AnimValue::Number(100.0), Easing::EaseIn),
    ];
    // raw 0.5 → eased 0.125 → 12.5
    match resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)) {
        AnimValue::Number(v) => assert!((v - 12.5).abs() < 1e-3),
        _ => panic!("expected number"),
    }
}

#[test]
fn step_holds_until_next_key() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Step),
        kf(AnimProp::Opacity, 10, AnimValue::Number(100.0), Easing::Linear),
    ];
    assert_eq!(resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)),
               AnimValue::Number(0.0));   // held at left value
    assert_eq!(resolve_value(&keys, AnimProp::Opacity, 10, 24, AnimValue::Number(0.0)),
               AnimValue::Number(100.0)); // exact key time → its own value
}

#[test]
fn exact_key_time_returns_that_keys_value() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(AnimProp::Opacity, 10, AnimValue::Number(100.0), Easing::Linear),
    ];
    assert_eq!(resolve_value(&keys, AnimProp::Opacity, 10, 24, AnimValue::Number(0.0)),
               AnimValue::Number(100.0));
}

#[test]
fn ignores_other_properties_keyframes() {
    let keys = vec![
        kf(AnimProp::Position, 0, AnimValue::Vec2([0.0, 0.0]), Easing::Linear),
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(AnimProp::Opacity, 10, AnimValue::Number(100.0), Easing::Linear),
    ];
    // Position has a single key → holds it; opacity keyframes must not leak in.
    assert_eq!(resolve_value(&keys, AnimProp::Position, 5, 24, AnimValue::Vec2([9.0, 9.0])),
               AnimValue::Vec2([0.0, 0.0]));
}
```

**Step 2 — verify fail:** compile error. RED.

**Step 3 — implement** in `src/keyframe.rs`:

```rust
use crate::project::{AnimProp, AnimValue, Easing, Keyframe, KeyTime, Layer, Transform};

pub struct AnimCtx { pub frame: u32, pub fps: u32, pub width: u32, pub height: u32, pub duration: u32 }

/// Resolve one property at `frame`: static value if no keyframes, else interpolated.
pub fn resolve_value(keyframes: &[Keyframe], prop: AnimProp, frame: u32, fps: u32,
                     static_value: AnimValue) -> AnimValue {
    let mut keys: Vec<&Keyframe> = keyframes.iter().filter(|k| k.property == prop).collect();
    if keys.is_empty() { return static_value; }
    keys.sort_by_key(|k| time_to_frame(&k.time, fps));

    // Before first / after last → hold.
    let first = time_to_frame(&keys[0].time, fps);
    let last = time_to_frame(&keys[keys.len() - 1].time, fps);
    if frame <= first { return keys[0].value; }
    if frame >= last { return keys[keys.len() - 1].value; }

    // Find bracketing pair (left key strictly before frame).
    let mut i = keys.len() - 1;
    while i > 0 && time_to_frame(&keys[i].time, fps) > frame { i -= 1; }
    let (a, b) = (keys[i], keys[i + 1]);
    let (ta, tb) = (time_to_frame(&a.time, fps), time_to_frame(&b.time, fps));

    // Step (hold): left easing == Step → left value until the next key.
    if a.easing == Easing::Step { return a.value; }

    let raw = (frame - ta) as f32 / (tb - ta) as f32;
    let t = ease(a.easing, raw);
    lerp(a.value, b.value, t)
}

fn lerp(a: AnimValue, b: AnimValue, t: f32) -> AnimValue {
    match (a, b) {
        (AnimValue::Number(x), AnimValue::Number(y)) => AnimValue::Number(x + (y - x) * t),
        (AnimValue::Vec2(x), AnimValue::Vec2(y)) => AnimValue::Vec2([
            x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t,
        ]),
        _ => panic!("keyframe value type mismatch between adjacent keys"),
    }
}

/// Resolve a layer's whole transform at `frame`. Expression override added in Task 6.
pub fn resolve_transform(layer: &Layer, ctx: &AnimCtx) -> Transform {
    let t = layer.transform;
    Transform {
        position: match resolve_value(&layer.keyframes, AnimProp::Position, ctx.frame, ctx.fps,
                                      AnimValue::Vec2(t.position)) {
            AnimValue::Vec2(v) => v, _ => unreachable!(),
        },
        scale: match resolve_value(&layer.keyframes, AnimProp::Scale, ctx.frame, ctx.fps,
                                   AnimValue::Vec2(t.scale)) {
            AnimValue::Vec2(v) => v, _ => unreachable!(),
        },
        opacity: match resolve_value(&layer.keyframes, AnimProp::Opacity, ctx.frame, ctx.fps,
                                     AnimValue::Number(t.opacity)) {
            AnimValue::Number(v) => v, _ => unreachable!(),
        },
        rotation: match resolve_value(&layer.keyframes, AnimProp::Rotation, ctx.frame, ctx.fps,
                                      AnimValue::Number(t.rotation)) {
            AnimValue::Number(v) => v, _ => unreachable!(),
        },
    }
}
```

**Step 4 — verify pass:** 9 passed. **Step 5 — commit:** `feat: keyframe interpolation + per-property resolution`

---

## Task 5: Time-aware compositing (RED first — pixel-verified)

**Files:** modify `src/compositor.rs`, `src/lib.rs` (already done Task 2), create `tests/anim_composite_test.rs`.

**Step 1 — failing tests** (100×100 canvas, black bg, red solid):

```rust
use opencomp::compositor::render_frame;
use opencomp::project::Project;

fn px(frame: &opencomp::compositor::Frame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * frame.width + x) * 4) as usize;
    [frame.pixels[i], frame.pixels[i + 1], frame.pixels[i + 2], frame.pixels[i + 3]]
}

const ANIM: &str = r#"
    [project]
    name = "anim"
    fps = 24
    width = 100
    height = 100
    duration = 100
    bg_color = "#000000"
    [[layer]]
    name = "square"
    type = "solid"
    color = "#ff0000"
    size = [10.0, 10.0]
    [[layer.keyframe]]
    property = "position"
    time = 0
    value = [50.0, 50.0]
    [[layer.keyframe]]
    property = "position"
    time = 10
    value = [90.0, 50.0]
    [[layer.keyframe]]
    property = "opacity"
    time = 0
    value = 100.0
    [[layer.keyframe]]
    property = "opacity"
    time = 10
    value = 0.0
"#;

#[test]
fn layer_moves_across_frames() {
    let proj: Project = toml::from_str(ANIM).unwrap();
    let f0 = render_frame(&proj, 0);
    assert_eq!(px(&f0, 50, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f0, 90, 50), [0, 0, 0, 255]);
    let f10 = render_frame(&proj, 10);
    assert_eq!(px(&f10, 90, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f10, 50, 50), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // linear midpoint → center (70, 50)
    assert_eq!(px(&f5, 70, 50), [255, 0, 0, 255]);
}

#[test]
fn opacity_fades_across_frames() {
    let proj: Project = toml::from_str(ANIM).unwrap();
    assert_eq!(px(&render_frame(&proj, 0), 50, 50), [255, 0, 0, 255]);
    assert_eq!(px(&render_frame(&proj, 10), 50, 50), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // opacity 50 → 127.5 rounds to 128
    assert_eq!(px(&f5, 50, 50), [128, 0, 0, 255]);
}

#[test]
fn static_layers_unchanged_across_frames() {
    let src = r#"
        [project]
        name = "static"
        fps = 24
        width = 100
        height = 100
        duration = 100
        bg_color = "#000000"
        [[layer]]
        name = "bg2"
        type = "solid"
        color = "#00ff00"
        size = [20.0, 20.0]
    "#;
    let proj: Project = toml::from_str(src).unwrap();
    for f in [0, 5, 99] {
        assert_eq!(px(&render_frame(&proj, f), 50, 50), [0, 255, 0, 255]);
    }
}
```

**Step 2 — verify fail:** RED (render_frame ignores `frame` today — first test fails on pixel
at frame 5/10). Confirm the actual assertion failure, not a compile error.

**Step 3 — implement:** in `render_frame`, resolve each layer's transform before compositing:

```rust
for layer in &proj.layers {
    let mut eff = layer.clone();
    eff.transform = crate::keyframe::resolve_transform(layer, &ctx);
    composite_layer(&mut pixels, &eff, width, height);
}
```

where `ctx = AnimCtx { frame, fps: proj.project.fps, width, height, duration: proj.project.duration }`.
(The full-canvas centering path already ignores `position` — unchanged.)

**Step 4 — verify pass:** 3 passed + full suite still green. **Step 5 — commit:** `feat: time-aware compositing (keyframed transforms)`

---

## Task 6: Python expression evaluator (RED first)

**Files:** create `src/expression.rs`, modify `src/lib.rs` (`pub mod expression;`),
modify `src/keyframe.rs` (expression override in `resolve_transform`), create `tests/expression_test.rs`.

**Step 1 — failing tests:**

```rust
use opencomp::expression::evaluate;
use opencomp::keyframe::AnimCtx;
use opencomp::project::{AnimValue, AnimProp, Easing, KeyTime, Keyframe};

fn ctx(frame: u32) -> AnimCtx {
    AnimCtx { frame, fps: 24, width: 640, height: 480, duration: 120 }
}

#[test]
fn evaluates_scalar_arithmetic() {
    let v = evaluate("value * 2", AnimValue::Number(25.0), &ctx(0)).unwrap();
    assert_eq!(v, AnimValue::Number(50.0));
}

#[test]
fn evaluates_vec2_list_ops() {
    let v = evaluate("[value[0] + 10, value[1] - 10]", AnimValue::Vec2([100.0, 200.0]), &ctx(0)).unwrap();
    assert_eq!(v, AnimValue::Vec2([110.0, 190.0]));
}

#[test]
fn sees_time_and_frame() {
    // time = frame/fps; time * fps == frame
    let v = evaluate("time * fps", AnimValue::Number(1.0), &ctx(48)).unwrap();
    assert_eq!(v, AnimValue::Number(48.0));
    let v = evaluate("frame", AnimValue::Number(1.0), &ctx(7)).unwrap();
    assert_eq!(v, AnimValue::Number(7.0));
}

#[test]
fn sees_width_height_duration() {
    let v = evaluate("width * height / duration", AnimValue::Number(1.0), &ctx(0)).unwrap();
    assert_eq!(v, AnimValue::Number(640.0 * 480.0 / 120.0));
}

#[test]
fn int_results_coerce_to_float() {
    let v = evaluate("2 + 3", AnimValue::Number(0.0), &ctx(0)).unwrap();
    assert_eq!(v, AnimValue::Number(5.0));
}

#[test]
fn invalid_expression_errors() {
    assert!(evaluate("value +", AnimValue::Number(1.0), &ctx(0)).is_err());
    assert!(evaluate("[1, 2, 3]", AnimValue::Vec2([0.0, 0.0]), &ctx(0)).is_err()); // wrong arity
}

#[test]
fn wrong_result_type_errors() {
    assert!(evaluate("'text'", AnimValue::Number(1.0), &ctx(0)).is_err());
}

#[test]
fn expression_overrides_keyframes() {
    use opencomp::keyframe::resolve_value;
    let keys = vec![
        Keyframe { property: AnimProp::Opacity, time: KeyTime::Frame(0), value: AnimValue::Number(0.0), easing: Easing::Linear },
        Keyframe { property: AnimProp::Opacity, time: KeyTime::Frame(10), value: AnimValue::Number(100.0), easing: Easing::Linear },
    ];
    // value passed to the expression = keyframe-resolved value at this frame
    let resolved = resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0));
    let v = evaluate("value * 0.5", resolved, &ctx(5)).unwrap();
    assert_eq!(v, AnimValue::Number(25.0));
}
```

**Step 2 — verify fail:** compile error (no `opencomp::expression`). RED.

**Step 3 — implement.**

`Cargo.toml`: add

```toml
rustpython-vm = { version = "0.4", features = ["stdlib"] }
```

(*If the `stdlib` feature name differs in the resolved version, drop it and keep
expressions arithmetic-only — the test contract above uses no imports.*)

`src/expression.rs` (skeleton — adjust to the resolved rustpython-vm API; the
tests are the contract):

```rust
use crate::keyframe::AnimCtx;
use crate::project::AnimValue;
use rustpython_vm::{Interpreter, Settings};

/// Evaluate a Python expression to a number or 2-element list.
/// Available names: value, time, frame, fps, width, height, duration.
pub fn evaluate(expr: &str, value: AnimValue, ctx: &AnimCtx) -> Result<AnimValue, String> {
    let interp = Interpreter::with_config(Settings::default());
    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();
        let g = &scope.globals;
        let time_s = ctx.frame as f64 / ctx.fps as f64;
        g.set_item("value", to_py(vm, value), vm);
        g.set_item("time", vm.ctx.new_float(time_s), vm);
        g.set_item("frame", vm.ctx.new_int(ctx.frame), vm);
        g.set_item("fps", vm.ctx.new_int(ctx.fps), vm);
        g.set_item("width", vm.ctx.new_int(ctx.width), vm);
        g.set_item("height", vm.ctx.new_int(ctx.height), vm);
        g.set_item("duration", vm.ctx.new_int(ctx.duration), vm);

        let code = vm.compile(expr, rustpython_vm::compiler::Mode::Eval, "<expr>".to_owned())
            .map_err(|e| format!("expression syntax error: {e}"))?;
        let result = vm.run_code_obj(code, scope).map_err(|e| format!("expression error: {}", vm.to_repr(&e)?))?;
        from_py(vm, &result)
    })
}

fn to_py(vm: &rustpython_vm::VirtualMachine, v: AnimValue) -> rustpython_vm::PyObjectRef {
    match v {
        AnimValue::Number(n) => vm.ctx.new_float(n as f64).into(),
        AnimValue::Vec2(p) => {
            let list = vm.ctx.new_list(vec![vm.ctx.new_float(p[0] as f64).into(), vm.ctx.new_float(p[1] as f64).into()]);
            list.into()
        }
    }
}

fn from_py(vm: &rustpython_vm::VirtualMachine, obj: &rustpython_vm::PyObjectRef) -> Result<AnimValue, String> {
    if let Ok(n) = vm.extract::<f64>(obj) { return Ok(AnimValue::Number(n as f32)); }
    if let Ok(v) = vm.extract::<Vec<f64>>(obj) {
        if v.len() == 2 { return Ok(AnimValue::Vec2([v[0] as f32, v[1] as f32])); }
        return Err(format!("expression must return a number or [x, y], got list of len {}", v.len()));
    }
    Err("expression must return a number or [x, y]".to_string())
}
```

Wire the override into `src/keyframe.rs::resolve_transform` (after each keyframe
resolve, before building the Transform):

```rust
fn expression_value(layer: &Layer, prop: AnimProp, resolved: AnimValue, ctx: &AnimCtx) -> AnimValue {
    match layer.expressions.iter().find(|e| e.property == prop) {
        Some(e) => crate::expression::evaluate(&e.expr, resolved, ctx)
            .unwrap_or_else(|err| panic!("layer '{}' property '{:?}' expression failed: {err}", layer.name, prop)),
        None => resolved,
    }
}
```

apply it to all four properties in `resolve_transform` (expression sees the
keyframe-resolved value — or the static value when no keyframes exist).

**Step 4 — verify pass:** 8 passed. Then `cargo test` full suite green.

**Step 5 — commit:** `feat: Python expression evaluator (rustpython-vm)`

---

## Task 7: CLI `--frame` (RED first)

**Files:** modify `src/main.rs`, extend `tests/cli_test.rs`.

**Step 1 — failing tests** (mirror existing CLI test pattern — run the built binary via `env!("CARGO_BIN_EXE_opencomp")`):

```rust
#[test]
fn render_accepts_frame_flag() {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_opencomp"))
        .args(["render", "tests/fixtures/anim.toml", "-f", "10", "-o", "out/cli_f10.png"])
        .output().unwrap();
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    assert!(std::path::Path::new("out/cli_f10.png").exists());
}

#[test]
fn different_frames_differ() {
    // frame 0 vs frame 10 of an animated project must produce different PNG bytes
    let a = std::fs::read("out/cli_f0.png").unwrap();
    let b = std::fs::read("out/cli_f10.png").unwrap();
    assert_ne!(a, b);
}
```

(Add `tests/fixtures/anim.toml` — the same animated project as Task 5, or reuse a
checked-in fixture. Pre-render `-f 0` in the second test.)

**Step 2 — verify fail:** `-f` is rejected today (unknown arg) → RED.

**Step 3 — implement:** add to the `Render` subcommand:

```rust
/// Frame number to render (0-based)
#[arg(short, long, default_value_t = 0)]
frame: u32,
```

and pass `frame` instead of hardcoded `0` into `render_frame`. Update the success
message to include the frame.

**Step 4 — verify pass:** `cargo test --test cli_test` → existing 2 + new 2 pass.

**Step 5 — commit:** `feat: CLI --frame for time-aware renders`

---

## Task 8: Animated demo + docs (v0.2 format spec)

**Files:**
- Modify: `examples/demo.toml` (add keyframes + one expression)
- Modify: `docs/PROJECT_FORMAT.md` (bump to v0.2: keyframes, expressions, time format; keep v0.1 fields)
- Modify: `README.md` (mention `--frame`, link keyframe spec)
- Modify: `src/main.rs` only if the demo reveals a gap

**Demo additions** (keep the four existing layers; animate the right card and the
semi-transparent strip):

```toml
[[layer.keyframe]]          # inside the right-card layer
property = "position"
time = 0
value = [480.0, 140.0]
easing = "ease_in_out"

[[layer.keyframe]]
property = "position"
time = "00:02:00"           # 2 s at 60 fps → frame 120
value = [480.0, 340.0]

[[layer.keyframe]]          # inside the strip layer (opacity pulse)
property = "opacity"
time = 0
value = 40.0

[[layer.keyframe]]
property = "opacity"
time = 60
value = 90.0
easing = "ease_in_out"

[[layer.keyframe]]
property = "opacity"
time = 120
value = 40.0
easing = "ease_out"

[[layer.expression]]        # spinning badge on the backdrop layer
property = "rotation"
expr = "frame * 2.0"        # 2°/frame → full turn in 180 frames
```

**Verification:**
1. `cargo run -- render examples/demo.toml -f 0 -o out/demo_f000.png`
2. `cargo run -- render examples/demo.toml -f 60 -o out/demo_f060.png`
3. `cargo run -- render examples/demo.toml -f 120 -o out/demo_f120.png`
4. **Verify by eye** (vision_analyze): card moved, strip opacity changed, badge rotated.
5. **Verify by pixel math** where cheap (e.g. strip opacity at frame 60 for a known
   pixel ≈ mid blend).
6. `cargo test` — full suite green.
7. `cargo fmt`.

**Commits:** `feat: animated demo project (keyframes + expression)` then
`docs: project format v0.2 (keyframes, expressions, timecode)`.

---

## Task 9: ADRs (docs-first)

**Files:** create `docs/ADRs/0002-full-canvas-centering.md`, `docs/ADRs/0003-python-expressions.md`.

- **ADR-0002** (carried over from Phase 1's pending list): full-canvas layer
  centering convention — `position` is the center for sized layers; for
  full-canvas layers it is ignored and the center is the canvas center.
  Consequence: keyframing position on a full-canvas layer is a no-op by design;
  give a layer a `size` to move it.
- **ADR-0003**: expressions are Python via rustpython-vm (in-process, pure Rust);
  expression overrides keyframes on the same property; the variable contract
  (`value`, `time`, `frame`, `fps`, `width`, `height`, `duration`); errors panic
  with a clear message in v0.2, proper `Result` type later.

**Commit:** `docs: ADR-0002 (full-canvas centering) + ADR-0003 (python expressions)`

Also update `docs/ARCHITECTURE.md` § Keyframe Engine if any design point changed
(it should now match: integer frame space, hold extrapolation, step easing).

---

## Task 10: GitHub Issues as task tracker

**Prereq (blocked on user/credentials — see `github-auth` skill):** `gh` is NOT
installed on this box and there is no API token. SSH key for git push exists
(`github-opencomp` alias) but GitHub Issues requires an API credential.

**Steps (once creds exist):**
1. Install `gh` (Debian: `curl -fsSL https://cli.github.com/packages/githubcli-archive-keyring.gpg | ...` or `gh` .deb — follow `github-auth` skill).
2. `gh auth login` (device flow needs the user at a browser) or use the user's fine-grained PAT.
3. `gh repo view jandre-kleynhans/opencomp` — verify access.
4. Create one issue per Phase 2 task (mirror `docs/PLAN_PHASE2.md` tasks) + a
   `phase-2` label. Verify with `gh issue list`.
5. Commit `chore: set up GitHub Issues tracker` only if repo files change (Issues themselves are API-side).

**If creds stay unavailable:** note the blocker in STATUS.md and keep the plan as
the tracker (repo-as-memory already covers it).

---

## End-of-phase verification checklist

- [ ] `cargo test` — all green (27 existing + ~38 new)
- [ ] `cargo build --release` — clean
- [ ] Demo renders at `-f 0 / 60 / 120`; animation verified by eye AND pixel math
- [ ] Expression-driven property verified end-to-end via CLI
- [ ] `docs/PROJECT_FORMAT.md` v0.2 matches `src/project.rs` (tests are the contract)
- [ ] ADR-0002 + ADR-0003 land
- [ ] STATUS.md + DEVLOG.md refreshed, conventional commit per task

## Risks / open questions

- **rustpython-vm** adds ~1–3 min build time and has API churn between versions;
  the plan pins the contract in tests and keeps the glue in one module so the
  interpreter can be swapped without touching the format or compositor.
  Fallback if `stdlib`/`extract` features misbehave: arithmetic-only expressions
  (all Task 6 tests pass without stdlib) or hand-bound `sin`/`cos`/`pi` helpers.
- **serde untagged `KeyTime`** — `Frame(u32)`/`Tc(String)` is unambiguous (int vs string).
- **Float pixel asserts** — Task 5 uses exact values that round cleanly (128, not 127/129);
  if a rounding edge appears, use a ±1 tolerance helper.
- **Expression panic-on-error** — deliberate for v0.2 (matches `Color::from_hex`
  panic contract); a typed error path is a Phase 3 concern.
- **Times are integer frames** — sub-frame motion (e.g. 120 fps keyframes on a
  24 fps project) is out of scope until real-time playback needs it.