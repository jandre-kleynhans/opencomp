use crate::project::{AnimProp, AnimValue, Easing, KeyTime, Keyframe, Layer, Transform};
use std::collections::HashMap;

/// Resolve a keyframe time to an integer frame number.
/// Frame ints pass through unchanged; `"MM:SS:FF"` is resolved via `fps`.
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

/// Apply an easing curve to a 0..1 progress value (clamped).
/// `Step` returns 0.0 — the interpolation caller must treat it as a hold instead of using this.
pub fn ease(e: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match e {
        Easing::Linear => t,
        Easing::EaseIn => t * t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => 3.0 * t * t - 2.0 * t * t * t,
        Easing::Step => 0.0,
    }
}

/// Resolve one animated property at the given frame.
/// If no keyframes target this property, `static_value` is returned unchanged.
/// Extrapolation: hold constant before the first key and after the last key.
/// Step easing = hold the left key's value until the next key.
pub fn resolve_value(
    keyframes: &[Keyframe],
    prop: AnimProp,
    frame: u32,
    fps: u32,
    static_value: AnimValue,
) -> AnimValue {
    let mut keys: Vec<&Keyframe> = keyframes.iter().filter(|k| k.property == prop).collect();
    if keys.is_empty() {
        return static_value;
    }
    keys.sort_by_key(|k| time_to_frame(&k.time, fps));

    let first = time_to_frame(&keys[0].time, fps);
    let last = time_to_frame(&keys[keys.len() - 1].time, fps);

    if frame <= first {
        return keys[0].value;
    }
    if frame >= last {
        return keys[keys.len() - 1].value;
    }

    // Find the bracketing pair: `i` is the last key whose time <= frame.
    let mut i = keys.len() - 1;
    while i > 0 && time_to_frame(&keys[i].time, fps) > frame {
        i -= 1;
    }
    let a = keys[i];
    let b = keys[i + 1];
    let ta = time_to_frame(&a.time, fps);
    let tb = time_to_frame(&b.time, fps);

    if a.easing == Easing::Step {
        return a.value;
    }

    let raw = (frame - ta) as f32 / (tb - ta) as f32;
    let t = ease(a.easing, raw);
    lerp(a.value, b.value, t)
}

fn lerp(a: AnimValue, b: AnimValue, t: f32) -> AnimValue {
    match (a, b) {
        (AnimValue::Number(x), AnimValue::Number(y)) => AnimValue::Number(x + (y - x) * t),
        (AnimValue::Vec2(x), AnimValue::Vec2(y)) => {
            AnimValue::Vec2([x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t])
        }
        _ => panic!("keyframe value type mismatch between adjacent keys"),
    }
}

/// Resolve a full layer transform at a given frame.
/// Keyframes override the static transform; expression override added in Task 6.
pub fn resolve_transform(layer: &Layer, frame: u32, fps: u32) -> Transform {
    let t = layer.transform;
    let position = resolve_vec2(&layer.keyframes, AnimProp::Position, frame, fps, t.position);
    let scale = resolve_vec2(&layer.keyframes, AnimProp::Scale, frame, fps, t.scale);
    let opacity = resolve_f32(&layer.keyframes, AnimProp::Opacity, frame, fps, t.opacity);
    let rotation = resolve_f32(&layer.keyframes, AnimProp::Rotation, frame, fps, t.rotation);
    Transform {
        position,
        scale,
        opacity,
        rotation,
    }
}

fn resolve_vec2(
    keys: &[Keyframe],
    prop: AnimProp,
    frame: u32,
    fps: u32,
    fallback: [f32; 2],
) -> [f32; 2] {
    match resolve_value(keys, prop, frame, fps, AnimValue::Vec2(fallback)) {
        AnimValue::Vec2(v) => v,
        _ => unreachable!(),
    }
}

fn resolve_f32(keys: &[Keyframe], prop: AnimProp, frame: u32, fps: u32, fallback: f32) -> f32 {
    match resolve_value(keys, prop, frame, fps, AnimValue::Number(fallback)) {
        AnimValue::Number(v) => v,
        _ => unreachable!(),
    }
}
