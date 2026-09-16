// Phase 2 Task 4 — keyframe interpolation + per-property resolution (RED first).
use opencomp::keyframe::resolve_value;
use opencomp::project::{AnimProp, AnimValue, Easing, KeyTime, Keyframe};

fn kf(prop: AnimProp, time: u32, value: AnimValue, easing: Easing) -> Keyframe {
    Keyframe {
        property: prop,
        time: KeyTime::Frame(time),
        value,
        easing,
    }
}

#[test]
fn no_keyframes_returns_static() {
    let v = resolve_value(&[], AnimProp::Opacity, 42, 24, AnimValue::Number(80.0));
    assert_eq!(v, AnimValue::Number(80.0));
}

#[test]
fn single_keyframe_holds_everywhere() {
    let keys = vec![kf(
        AnimProp::Opacity,
        10,
        AnimValue::Number(50.0),
        Easing::Linear,
    )];
    for f in [0, 5, 10, 999] {
        assert_eq!(
            resolve_value(&keys, AnimProp::Opacity, f, 24, AnimValue::Number(0.0)),
            AnimValue::Number(50.0)
        );
    }
}

#[test]
fn before_first_and_after_last_hold() {
    let keys = vec![
        kf(
            AnimProp::Position,
            10,
            AnimValue::Vec2([100.0, 200.0]),
            Easing::Linear,
        ),
        kf(
            AnimProp::Position,
            20,
            AnimValue::Vec2([300.0, 400.0]),
            Easing::Linear,
        ),
    ];
    assert_eq!(
        resolve_value(
            &keys,
            AnimProp::Position,
            0,
            24,
            AnimValue::Vec2([0.0, 0.0])
        ),
        AnimValue::Vec2([100.0, 200.0])
    );
    assert_eq!(
        resolve_value(
            &keys,
            AnimProp::Position,
            30,
            24,
            AnimValue::Vec2([0.0, 0.0])
        ),
        AnimValue::Vec2([300.0, 400.0])
    );
}

#[test]
fn linear_midpoint_interpolates_scalar() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(
            AnimProp::Opacity,
            10,
            AnimValue::Number(100.0),
            Easing::Linear,
        ),
    ];
    assert_eq!(
        resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)),
        AnimValue::Number(50.0)
    );
}

#[test]
fn linear_midpoint_interpolates_vec2() {
    let keys = vec![
        kf(
            AnimProp::Position,
            0,
            AnimValue::Vec2([0.0, 0.0]),
            Easing::Linear,
        ),
        kf(
            AnimProp::Position,
            10,
            AnimValue::Vec2([100.0, 200.0]),
            Easing::Linear,
        ),
    ];
    assert_eq!(
        resolve_value(
            &keys,
            AnimProp::Position,
            5,
            24,
            AnimValue::Vec2([0.0, 0.0])
        ),
        AnimValue::Vec2([50.0, 100.0])
    );
}

#[test]
fn ease_in_curves_the_progress() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::EaseIn),
        kf(
            AnimProp::Opacity,
            10,
            AnimValue::Number(100.0),
            Easing::EaseIn,
        ),
    ];
    // raw 0.5 -> eased 0.125 -> 12.5
    match resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)) {
        AnimValue::Number(v) => assert!((v - 12.5).abs() < 1e-3),
        _ => panic!("expected number"),
    }
}

#[test]
fn step_holds_until_next_key() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Step),
        kf(
            AnimProp::Opacity,
            10,
            AnimValue::Number(100.0),
            Easing::Linear,
        ),
    ];
    assert_eq!(
        resolve_value(&keys, AnimProp::Opacity, 5, 24, AnimValue::Number(0.0)),
        AnimValue::Number(0.0) // held at left value
    );
    assert_eq!(
        resolve_value(&keys, AnimProp::Opacity, 10, 24, AnimValue::Number(0.0)),
        AnimValue::Number(100.0) // exact key time -> its own value
    );
}

#[test]
fn exact_key_time_returns_that_keys_value() {
    let keys = vec![
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(
            AnimProp::Opacity,
            10,
            AnimValue::Number(100.0),
            Easing::Linear,
        ),
    ];
    assert_eq!(
        resolve_value(&keys, AnimProp::Opacity, 10, 24, AnimValue::Number(0.0)),
        AnimValue::Number(100.0)
    );
}

#[test]
fn ignores_other_properties_keyframes() {
    let keys = vec![
        kf(
            AnimProp::Position,
            0,
            AnimValue::Vec2([0.0, 0.0]),
            Easing::Linear,
        ),
        kf(AnimProp::Opacity, 0, AnimValue::Number(0.0), Easing::Linear),
        kf(
            AnimProp::Opacity,
            10,
            AnimValue::Number(100.0),
            Easing::Linear,
        ),
    ];
    // Position has a single key -> holds it; opacity keyframes must not leak in.
    assert_eq!(
        resolve_value(
            &keys,
            AnimProp::Position,
            5,
            24,
            AnimValue::Vec2([9.0, 9.0])
        ),
        AnimValue::Vec2([0.0, 0.0])
    );
}
