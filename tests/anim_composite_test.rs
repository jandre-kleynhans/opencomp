// Phase 2 Task 5 — time-aware compositing (RED first, pixel-verified).
use opencomp::compositor::{Frame, render_frame};
use opencomp::project::Project;

fn px(frame: &Frame, x: u32, y: u32) -> [u8; 4] {
    let i = ((y * frame.width + x) * 4) as usize;
    [
        frame.pixels[i],
        frame.pixels[i + 1],
        frame.pixels[i + 2],
        frame.pixels[i + 3],
    ]
}

const ANIM: &str = r##"
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
"##;

#[test]
fn layer_moves_across_frames() {
    let proj: Project = toml::from_str(ANIM).unwrap();
    let f0 = render_frame(&proj, 0);
    assert_eq!(px(&f0, 50, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f0, 90, 50), [0, 0, 0, 255]);
    let f10 = render_frame(&proj, 10);
    assert_eq!(px(&f10, 90, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f10, 50, 50), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // linear midpoint -> center (70, 50)
    assert_eq!(px(&f5, 70, 50), [255, 0, 0, 255]);
}

#[test]
fn opacity_fades_across_frames() {
    let proj: Project = toml::from_str(ANIM).unwrap();
    assert_eq!(px(&render_frame(&proj, 0), 50, 50), [255, 0, 0, 255]);
    assert_eq!(px(&render_frame(&proj, 10), 50, 50), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // opacity 50 -> 127.5 rounds to 128
    assert_eq!(px(&f5, 50, 50), [128, 0, 0, 255]);
}

#[test]
fn static_layers_unchanged_across_frames() {
    let src = r##"
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
    "##;
    let proj: Project = toml::from_str(src).unwrap();
    for f in [0, 5, 99] {
        assert_eq!(px(&render_frame(&proj, f), 50, 50), [0, 255, 0, 255]);
    }
}