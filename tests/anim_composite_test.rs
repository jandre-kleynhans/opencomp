// Phase 2 Task 5 — time-aware compositing (RED first, pixel-verified).
// NOTE: position and opacity animations tested with SEPARATE projects to avoid
// one animating property contaminating the other's pixel expectation.
use opencomp::compositor::{render_frame, Frame};
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

const MOVE: &str = r##"
    [project]
    name = "move"
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
"##;

#[test]
fn layer_moves_across_frames() {
    let proj: Project = toml::from_str(MOVE).unwrap();
    let f0 = render_frame(&proj, 0);
    assert_eq!(px(&f0, 50, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f0, 90, 50), [0, 0, 0, 255]);
    let f10 = render_frame(&proj, 10);
    assert_eq!(px(&f10, 90, 50), [255, 0, 0, 255]);
    assert_eq!(px(&f10, 50, 50), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // linear midpoint -> center (70, 50)
    assert_eq!(px(&f5, 70, 50), [255, 0, 0, 255]);
}

const FADE: &str = r##"
    [project]
    name = "fade"
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
    property = "opacity"
    time = 0
    value = 100.0

    [[layer.keyframe]]
    property = "opacity"
    time = 10
    value = 0.0
"##;

#[test]
fn opacity_fades_across_frames() {
    let proj: Project = toml::from_str(FADE).unwrap();
    // Static position keeps the square at its default center: position [0,0] for a
    // sized layer centers it? No — position [0,0] = top-left corner. Give it a position.
    // The square is centered at (50,50) via... let me set position default: [0,0].
    // Actually for a sized layer, position [0,0] means center at (0,0) — the square
    // would be at top-left, only pixels 0..10,0..10. So check the fade at a pixel
    // that's inside the square for ALL frames of the fade.
    // Square at [0,0] size 10x10 -> covers (0..10, 0..10). Pixel (5,5) inside.
    assert_eq!(px(&render_frame(&proj, 0), 5, 5), [255, 0, 0, 255]);
    assert_eq!(px(&render_frame(&proj, 10), 5, 5), [0, 0, 0, 255]);
    let f5 = render_frame(&proj, 5); // opacity 50 -> 127.5 rounds to 128
    assert_eq!(px(&f5, 5, 5), [128, 0, 0, 255]);
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
        size = [40.0, 40.0]

        [layer.transform]
        position = [50.0, 50.0]
    "##;
    let proj: Project = toml::from_str(src).unwrap();
    for f in [0, 5, 99] {
        assert_eq!(px(&render_frame(&proj, f), 50, 50), [0, 255, 0, 255]);
    }
}
