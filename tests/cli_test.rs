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

#[test]
fn render_accepts_frame_flag() {
    let proj = write_temp_project("opencomp_cli_frame.toml");
    let out = std::env::temp_dir().join("opencomp_cli_f0.png");
    let _ = std::fs::remove_file(&out);
    let out2 = out.clone();
    let status = Command::new(bin())
        .args([
            "render",
            proj.to_str().unwrap(),
            "-f",
            "0",
            "-o",
            out.to_str().unwrap(),
        ])
        .status()
        .unwrap();
    assert!(status.success(), "render with -f 0 exited non-zero");
    assert!(out2.exists(), "output PNG not created");
    let _ = std::fs::remove_file(&proj);
    let _ = std::fs::remove_file(&out2);
}

#[test]
fn different_frames_produce_different_pngs() {
    let (f0, f10);
    {
        // Animated project: red square moves across 10 frames
        let proj = std::env::temp_dir().join("opencomp_cli_anim.toml");
        let src = r##"
            [project]
            name = "anim"
            fps = 24
            width = 64
            height = 48
            duration = 10
            bg_color = "#000000"

            [[layer]]
            name = "red"
            type = "solid"
            color = "#ff0000"
            size = [20.0, 20.0]

            [[layer.keyframe]]
            property = "position"
            time = 0
            value = [32.0, 24.0]

            [[layer.keyframe]]
            property = "position"
            time = 10
            value = [54.0, 24.0]
        "##;
        std::fs::write(&proj, src).unwrap();

        let out0 = std::env::temp_dir().join("opencomp_cli_anim_f0.png");
        let out10 = std::env::temp_dir().join("opencomp_cli_anim_f10.png");
        let _ = std::fs::remove_file(&out0);
        let _ = std::fs::remove_file(&out10);

        let s0 = Command::new(bin())
            .args([
                "render",
                proj.to_str().unwrap(),
                "-f",
                "0",
                "-o",
                out0.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        let s10 = Command::new(bin())
            .args([
                "render",
                proj.to_str().unwrap(),
                "-f",
                "10",
                "-o",
                out10.to_str().unwrap(),
            ])
            .status()
            .unwrap();
        assert!(s0.success() && s10.success());

        f0 = std::fs::read(&out0).unwrap();
        f10 = std::fs::read(&out10).unwrap();
        let _ = std::fs::remove_file(&proj);
        let _ = std::fs::remove_file(&out0);
        let _ = std::fs::remove_file(&out10);
    }
    assert_ne!(f0, f10, "frame 0 and frame 10 PNGs should differ");
}
