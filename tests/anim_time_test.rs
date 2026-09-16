// Phase 2 Task 2 — timecode -> frame conversion (RED first).
use opencomp::keyframe::{ease, time_to_frame};
use opencomp::project::{Easing, KeyTime};

#[test]
fn frame_int_passes_through() {
    assert_eq!(time_to_frame(&KeyTime::Frame(42), 24), 42);
    assert_eq!(time_to_frame(&KeyTime::Frame(0), 60), 0);
}

#[test]
fn timecode_converts_via_fps() {
    // MM:SS:FF — FF is *frames* (not seconds)
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:00".into()), 24), 0);
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:01".into()), 24), 1); // 1 frame
    assert_eq!(time_to_frame(&KeyTime::Tc("00:01:00".into()), 24), 24); // 1 second
    assert_eq!(time_to_frame(&KeyTime::Tc("00:01:12".into()), 24), 36); // 1s12f
    assert_eq!(time_to_frame(&KeyTime::Tc("00:00:15".into()), 60), 15);
    assert_eq!(
        time_to_frame(&KeyTime::Tc("01:30:15".into()), 24),
        90 * 24 + 15 // 1m30s15f
    );
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

// Task 3 easing curves live here too (same module).

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
