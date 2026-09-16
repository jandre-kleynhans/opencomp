use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    // Cargo sets CARGO_BIN_EXE_opencomp for integration tests.
    PathBuf::from(env!("CARGO_BIN_EXE_opencomp"))
}

fn write_temp_project(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(name);
    let src = r##"
        [project]
        name = "cli"
        fps = 24
        width = 64
        height = 48
        duration = 10
        bg_color = "#112233"

        [[layer]]
        name = "red"
        type = "solid"
        color = "#ff0000"
        size = [20.0, 20.0]

        [layer.transform]
        position = [32.0, 24.0]
    "##;
    std::fs::write(&path, src).unwrap();
    path
}

#[test]
fn render_writes_png_file() {
    let proj = write_temp_project("opencomp_cli_render.toml");
    let out = std::env::temp_dir().join("opencomp_cli_out.png");
    let _ = std::fs::remove_file(&out);

    let out2 = out.clone();
    let status = Command::new(bin())
        .args([
            "render",
            proj.to_str().unwrap(),
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "render exited non-zero");

    assert!(out2.exists(), "output PNG not created");
    assert!(std::fs::metadata(&out2).unwrap().len() > 0);

    let _ = std::fs::remove_file(&proj);
    let _ = std::fs::remove_file(&out2);
}

#[test]
fn render_missing_file_fails() {
    let status = Command::new(bin())
        .args(["render", "/nonexistent/project.toml"])
        .status()
        .unwrap();
    assert!(!status.success(), "missing project should fail");
}
