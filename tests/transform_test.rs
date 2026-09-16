use opencomp::compositor::render_frame;
use opencomp::project::Project;

/// Helper: build a project with one solid layer. Optional extra layer TOML appended after `[[layer]]`.
fn project(layer: &str) -> Project {
    let src = format!(
        r##"
        [project]
        name = "xform"
        fps = 24
        width = 64
        height = 48
        duration = 10
        bg_color = "#000000"

        [[layer]]
        {layer}
    "##
    );
    toml::from_str(&src).unwrap()
}

fn pixel(frame: &opencomp::compositor::Frame, x: u32, y: u32) -> [u8; 4] {
    let idx = ((y * frame.width + x) * 4) as usize;
    [
        frame.pixels[idx],
        frame.pixels[idx + 1],
        frame.pixels[idx + 2],
        frame.pixels[idx + 3],
    ]
}

// ------- Task 6 tests: position, scale, opacity, rotation -------

#[test]
fn sized_layer_centered_at_position() {
    // 20×20 red layer, center at (32, 24) → covers [22..42, 14..34]
    let p = project(
        r##"
        name = "c"
        type = "solid"
        color = "#ff0000"
        size = [20.0, 20.0]

        [layer.transform]
        position = [32.0, 24.0]
        "##,
    );
    let f = render_frame(&p, 0);

    // Pixel inside → red
    assert_eq!(pixel(&f, 32, 24), [255, 0, 0, 255]);
    // Pixel outside → black background
    assert_eq!(pixel(&f, 0, 0), [0, 0, 0, 255]);
    // Pixel at left edge of layer (22) → red
    assert_eq!(pixel(&f, 22, 24), [255, 0, 0, 255]);
    // Pixel just outside left edge (21) → black
    assert_eq!(pixel(&f, 21, 24), [0, 0, 0, 255]);
}

#[test]
fn scale_doubles_footprint() {
    // 10×10 layer, scale 2×2 → actual 20×20 at center (32, 24)
    let p = project(
        r##"
        name = "s"
        type = "solid"
        color = "#00ff00"
        size = [10.0, 10.0]

        [layer.transform]
        position = [32.0, 24.0]
        scale = [2.0, 2.0]
        "##,
    );
    let f = render_frame(&p, 0);

    // Inside scaled footprint → green
    assert_eq!(pixel(&f, 32, 24), [0, 255, 0, 255]);
    // Edge at 22 (= 32 - 10) → green
    assert_eq!(pixel(&f, 22, 24), [0, 255, 0, 255]);
    // Outside at 21 → black
    assert_eq!(pixel(&f, 21, 24), [0, 0, 0, 255]);
}

#[test]
fn opacity_fifty_percent_blend() {
    // 20×20 red layer, opacity 50 → (128, 0, 0, 255) over black
    let p = project(
        r##"
        name = "a"
        type = "solid"
        color = "#ff0000"
        size = [20.0, 20.0]

        [layer.transform]
        position = [32.0, 24.0]
        opacity = 50.0
        "##,
    );
    let f = render_frame(&p, 0);
    assert_eq!(pixel(&f, 32, 24), [128, 0, 0, 255]);
}

#[test]
fn rotation_clips_corners() {
    // 40×40 red layer, rotation 45° → diamond, corners clipped.
    // At center (32, 24): still inside → red.
    // At top-right corner of unrotated rect: clipped → black.
    let p = project(
        r##"
        name = "r"
        type = "solid"
        color = "#ff0000"
        size = [40.0, 40.0]

        [layer.transform]
        position = [32.0, 24.0]
        rotation = 45.0
        "##,
    );
    let f = render_frame(&p, 0);

    // Center → red
    assert_eq!(pixel(&f, 32, 24), [255, 0, 0, 255]);
    // Original unrotated corner (52, 44) → outside diamond → black
    assert_eq!(pixel(&f, 52, 44), [0, 0, 0, 255]);
}

#[test]
fn multiple_sized_layers_stacked() {
    // Two layers: blue (bottom, 30×30, center 32,24), red (top, 10×10, center 32,24)
    // Overlap region (10×10 at center) → red.
    // Non-overlap (blue ring) → blue.
    let src = r##"
        [project]
        name = "stack"
        fps = 24
        width = 64
        height = 48
        duration = 10
        bg_color = "#000000"

        [[layer]]
        name = "blue"
        type = "solid"
        color = "#0000ff"
        size = [30.0, 30.0]

        [layer.transform]
        position = [32.0, 24.0]

        [[layer]]
        name = "red"
        type = "solid"
        color = "#ff0000"
        size = [10.0, 10.0]

        [layer.transform]
        position = [32.0, 24.0]
    "##;
    let p: Project = toml::from_str(src).unwrap();
    let f = render_frame(&p, 0);

    // Center → red (on top)
    assert_eq!(pixel(&f, 32, 24), [255, 0, 0, 255]);
    // In the blue ring (20, 24) → blue
    assert_eq!(pixel(&f, 20, 24), [0, 0, 255, 255]);
    // Outside both → black bg
    assert_eq!(pixel(&f, 0, 0), [0, 0, 0, 255]);
}
