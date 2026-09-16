// Phase 2 Task 1 — keyframe + expression model parse tests (RED first).
// NOTE: raw strings use r##"..."## because hex colors (#...) truncate r#"..."#.
use opencomp::project::{AnimProp, AnimValue, Easing, KeyTime, Project};

fn parse(src: &str) -> Project {
    toml::from_str(src).unwrap()
}

const MIN: &str = r##"
    [project]
    name = "demo"
    fps = 24
    width = 640
    height = 480
    duration = 120
    bg_color = "#000000"
"##;

#[test]
fn parses_keyframe_with_frame_int_time() {
    let proj = parse(&format!(
        r##"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        size = [80.0, 80.0]
        [[layer.keyframe]]
        property = "position"
        time = 10
        value = [100.0, 200.0]
    "##
    ));
    let kf = &proj.layers[0].keyframes[0];
    assert_eq!(kf.property, AnimProp::Position);
    assert_eq!(kf.time, KeyTime::Frame(10));
    assert_eq!(kf.value, AnimValue::Vec2([100.0, 200.0]));
    assert_eq!(kf.easing, Easing::Linear); // default
}

#[test]
fn parses_keyframe_with_timecode_string() {
    let proj = parse(&format!(
        r##"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.keyframe]]
        property = "opacity"
        time = "00:01:00"
        value = 50.0
        easing = "ease_in_out"
    "##
    ));
    let kf = &proj.layers[0].keyframes[0];
    assert_eq!(kf.time, KeyTime::Tc("00:01:00".to_string()));
    assert_eq!(kf.value, AnimValue::Number(50.0));
    assert_eq!(kf.easing, Easing::EaseInOut);
}

#[test]
fn parses_int_scalar_value() {
    // TOML `value = 50` (integer) must deserialize to Number(50.0)
    let proj = parse(&format!(
        r##"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.keyframe]]
        property = "opacity"
        time = 0
        value = 50
    "##
    ));
    assert_eq!(proj.layers[0].keyframes[0].value, AnimValue::Number(50.0));
}

#[test]
fn parses_all_easing_names() {
    for (name, want) in [
        ("linear", Easing::Linear),
        ("ease_in", Easing::EaseIn),
        ("ease_out", Easing::EaseOut),
        ("ease_in_out", Easing::EaseInOut),
        ("step", Easing::Step),
    ] {
        let src = format!(
            r##"{MIN}
            [[layer]]
            name = "card"
            type = "solid"
            color = "#ff0000"
            [[layer.keyframe]]
            property = "rotation"
            time = 0
            value = 0.0
            easing = "{name}"
        "##
        );
        assert_eq!(
            parse(&src).layers[0].keyframes[0].easing,
            want,
            "easing {name}"
        );
    }
}

#[test]
fn parses_expression_table() {
    let proj = parse(&format!(
        r##"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
        [[layer.expression]]
        property = "rotation"
        expr = "frame * 2.0"
    "##
    ));
    let e = &proj.layers[0].expressions[0];
    assert_eq!(e.property, AnimProp::Rotation);
    assert_eq!(e.expr, "frame * 2.0");
}

#[test]
fn layer_without_keyframes_defaults_empty() {
    let proj = parse(&format!(
        r##"{MIN}
        [[layer]]
        name = "card"
        type = "solid"
        color = "#ff0000"
    "##
    ));
    assert!(proj.layers[0].keyframes.is_empty());
    assert!(proj.layers[0].expressions.is_empty());
}
