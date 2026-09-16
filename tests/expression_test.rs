// Phase 2 Task 6 — Python expression evaluator (RED first).
use opencomp::keyframe::{evaluate_expression, evaluate_expression_vec, resolve_value};
use opencomp::project::{AnimProp, AnimValue, Easing, KeyTime, Keyframe};

fn ctx() -> (u32, u32, u32, u32) {
    // frame, fps, width, height, duration
    (0, 24, 640, 480)
}

#[test]
fn evaluates_scalar_arithmetic() {
    let (fr, fps, w, h) = ctx();
    let v = evaluate_expression("value * 2", AnimValue::Number(25.0), fr, fps, w, h, 120);
    assert_eq!(v, AnimValue::Number(50.0));
}

#[test]
fn evaluates_vector_x_operation() {
    let (fr, fps, w, h) = ctx();
    let v = evaluate_expression_vec(
        "[value + 10, value - 10]",
        [100.0, 200.0],
        fr,
        fps,
        w,
        h,
        120,
    );
    assert_eq!(v, AnimValue::Vec2([110.0, 190.0]));
}

#[test]
fn sees_time_and_frame() {
    let (_, fps, w, h) = ctx();
    // frame 48, fps 24 -> time = 2.0
    let v = evaluate_expression("time * fps", AnimValue::Number(1.0), 48, fps, w, h, 120);
    assert_eq!(v, AnimValue::Number(48.0));
    let v = evaluate_expression("frame", AnimValue::Number(1.0), 7, fps, w, h, 120);
    assert_eq!(v, AnimValue::Number(7.0));
}

#[test]
fn sees_width_height_duration() {
    let (fr, fps, w, h) = ctx();
    let v = evaluate_expression(
        "width * height / duration",
        AnimValue::Number(1.0),
        fr,
        fps,
        w,
        h,
        120,
    );
    assert_eq!(v, AnimValue::Number((640 * 480 / 120) as f32));
}

#[test]
fn int_results_coerce_to_float() {
    let (fr, fps, w, h) = ctx();
    let v = evaluate_expression("2 + 3", AnimValue::Number(0.0), fr, fps, w, h, 120);
    assert_eq!(v, AnimValue::Number(5.0));
}

#[test]
#[should_panic(expected = "syntax error")]
fn invalid_expression_panics() {
    let (fr, fps, w, h) = ctx();
    let _ = evaluate_expression("value +", AnimValue::Number(1.0), fr, fps, w, h, 120);
}

#[test]
#[should_panic(expected = "must be a number")]
fn non_numeric_result_panics() {
    let (fr, fps, w, h) = ctx();
    let _ = evaluate_expression("'text'", AnimValue::Number(1.0), fr, fps, w, h, 120);
}

#[test]
fn expression_overrides_keyframes() {
    let fps = 24;
    let keys = vec![
        Keyframe {
            property: AnimProp::Opacity,
            time: KeyTime::Frame(0),
            value: AnimValue::Number(0.0),
            easing: Easing::Linear,
        },
        Keyframe {
            property: AnimProp::Opacity,
            time: KeyTime::Frame(10),
            value: AnimValue::Number(100.0),
            easing: Easing::Linear,
        },
    ];
    // value passed to expression = keyframe-resolved value at this frame
    let resolved = resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0));
    let v = evaluate_expression("value * 0.5", resolved, 5, fps, 640, 480, 120);
    assert_eq!(v, AnimValue::Number(25.0));
}

#[test]
fn vec2_two_element_list_result() {
    let (fr, fps, w, h) = ctx();
    let v = evaluate_expression_vec(
        "[value[0] * 2, value[1] / 2]",
        [100.0, 200.0],
        fr,
        fps,
        w,
        h,
        120,
    );
    assert_eq!(v, AnimValue::Vec2([200.0, 100.0]));
}
