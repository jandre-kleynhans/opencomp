use clap::{Parser, Subcommand};
use opencomp::{compositor, export};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "opencomp", version, about = "OpenComp compositing engine")]
struct Cli {
    #[command(subcommand)]
    cmd: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Render a project file to a PNG frame or MP4 video
    Render {
        /// Path to a .toml project file
        project: PathBuf,

        /// Output file path (PNG for single frame, MP4 for --all)
        #[arg(short, long, default_value = "frame_0000.png")]
        output: PathBuf,

        /// Frame number to render (0-based)
        #[arg(short, long, default_value_t = 0)]
        frame: u32,

        /// Render all frames and encode to MP4 (requires ffmpeg)
        #[arg(long)]
        all: bool,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Render {
            project,
            output,
            frame,
            all,
        } => {
            let src = match std::fs::read_to_string(&project) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot read {}: {e}", project.display());
                    std::process::exit(1);
                }
            };
            let proj = match toml::from_str::<opencomp::project::Project>(&src) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("error: invalid project: {e}");
                    std::process::exit(1);
                }
            };

            if all {
                render_all(&proj, &output, proj.project.fps);
            } else {
                let rendered = compositor::render_frame(&proj, frame);
                match export::write_png(&rendered, &output) {
                    Ok(()) => println!(
                        "rendered {}×{} frame {} → {}",
                        rendered.width,
                        rendered.height,
                        frame,
                        output.display()
                    ),
                    Err(e) => {
                        eprintln!("error: write failed: {e}");
                        std::process::exit(1);
                    }
                }
            }
        }
    }
}

fn render_all(proj: &opencomp::project::Project, output: &PathBuf, fps: u32) {
    // Write frames to a temp dir, then pipe to ffmpeg
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let frame_pattern = temp_dir.path().join("frame_%04d.png");

    // Render all frames to PNG sequence
    let duration = proj.project.duration;
    for f in 0..duration {
        let frame = compositor::render_frame(proj, f);
        let frame_path = temp_dir.path().join(format!("frame_{:04}.png", f));
        export::write_png(&frame, &frame_path).expect("failed to write frame");
        if f % 30 == 0 {
            eprint!("\rrendering frame {}/{}", f + 1, duration);
        }
    }
    eprintln!();

    // Encode with ffmpeg
    let status = std::process::Command::new("ffmpeg")
        .args([
            "-y",
            "-framerate",
            &fps.to_string(),
            "-i",
            frame_pattern.to_str().unwrap(),
            "-c:v",
            "libx264",
            "-preset",
            "medium",
            "-crf",
            "18",
            "-pix_fmt",
            "yuv420p",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("failed to spawn ffmpeg");

    if !status.success() {
        eprintln!("error: ffmpeg failed");
        std::process::exit(1);
    }

    println!(
        "rendered {} frames → {}",
        proj.project.duration,
        output.display()
    );
}
