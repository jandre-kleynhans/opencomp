use opencomp::project::{BlendMode, Project};

#[test]
fn parses_minimal_project() {
    let src = r##"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#0a0a0a"
    "##;
    let proj: Project = toml::from_str(src).unwrap();
    assert_eq!(proj.project.name, "demo");
    assert_eq!(proj.project.width, 640);
    assert_eq!(proj.project.height, 480);
    assert_eq!(proj.project.bg_color.to_hex(), "#0a0a0a");
}

#[test]
fn parses_layer_with_defaults() {
    let src = r##"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#000000"

        [[layer]]
        name = "backdrop"
        type = "solid"
        color = "#ff0000"
    "##;
    let proj: Project = toml::from_str(src).unwrap();
    let l = &proj.layers[0];
    assert_eq!(l.name, "backdrop");
    assert_eq!(l.transform.position, [0.0, 0.0]); // default
    assert_eq!(l.transform.opacity, 100.0); // default
    assert_eq!(l.transform.scale, [1.0, 1.0]); // default
    assert_eq!(l.blend, BlendMode::Normal);
}

#[test]
fn parses_full_layer() {
    let src = r##"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#000000"

        [[layer]]
        name = "card"
        type = "solid"
        color = "#00ff00"
        blend = "normal"

        [layer.transform]
        position = [10.0, 20.0]
        scale = [2.0, 2.0]
        opacity = 50.0
        rotation = 45.0
    "##;
    let proj: Project = toml::from_str(src).unwrap();
    let l = &proj.layers[0];
    assert_eq!(l.name, "card");
    assert_eq!(l.transform.position, [10.0, 20.0]);
    assert_eq!(l.transform.scale, [2.0, 2.0]);
    assert_eq!(l.transform.opacity, 50.0);
    assert_eq!(l.transform.rotation, 45.0);
}
