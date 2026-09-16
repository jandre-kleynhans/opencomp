use opencomp::compositor::render_frame;
use opencomp::project::Project;

fn project_with_layer(layer_toml: &str) -> Project {
    let src = format!(
        r##"
        [project]
        name = "layer"
        fps = 24
        width = 64
        height = 48
        duration = 10
        bg_color = "#000000"

        [[layer]]
        {layer_toml}
    "##
    );
    toml::from_str(&src).unwrap()
}

#[test]
fn full_canvas_solid_layer_over_background() {
    // Default transform → layer covers the whole frame.
    let proj = project_with_layer(
        r##"
        name = "red"
        type = "solid"
        color = "#ff0000"
        "##,
    );
    let frame = render_frame(&proj, 0);
    for px in frame.pixels.chunks_exact(4) {
        assert_eq!(px, &[255, 0, 0, 255]);
    }
}

#[test]
fn transparent_layer_shows_background_through() {
    // Alpha 128 → 50% blend of red over black bg: (128, 0, 0, 255).
    let proj = project_with_layer(
        r##"
        name = "red"
        type = "solid"
        color = "#ff000080"
        "##,
    );
    let frame = render_frame(&proj, 0);
    for px in frame.pixels.chunks_exact(4) {
        assert_eq!(px, &[128, 0, 0, 255]);
    }
}
