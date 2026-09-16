use opencomp::compositor::render_frame;
use opencomp::project::Project;

fn project_with_bg(bg: &str) -> Project {
    let src = format!(
        r##"
        [project]
        name = "bg"
        fps = 24
        width = 64
        height = 48
        duration = 10
        bg_color = "{bg}"
    "##
    );
    toml::from_str(&src).unwrap()
}

#[test]
fn fills_background_color() {
    let proj = project_with_bg("#0a0b0c");
    let frame = render_frame(&proj, 0);

    assert_eq!(frame.width, 64);
    assert_eq!(frame.height, 48);
    assert_eq!(frame.pixels.len(), 64 * 48 * 4);

    // Every pixel is the background color, fully opaque.
    for px in frame.pixels.chunks_exact(4) {
        assert_eq!(px, &[10, 11, 12, 255]);
    }
}

#[test]
fn background_alpha_defaults_opaque() {
    // 3-char hex → alpha 255 (opaque), not transparent
    let proj = project_with_bg("#ffffff");
    let frame = render_frame(&proj, 0);
    for px in frame.pixels.chunks_exact(4) {
        assert_eq!(px, &[255, 255, 255, 255]);
    }
}

#[test]
fn zero_layers_frame_is_pure_background() {
    let proj = project_with_bg("#000000");
    let frame = render_frame(&proj, 0);
    assert_eq!(frame.pixels.len(), 64 * 48 * 4);
    // Background is opaque black: RGB 0, alpha 255 (solid fill).
    for px in frame.pixels.chunks_exact(4) {
        assert_eq!(px, &[0, 0, 0, 255]);
    }
}
