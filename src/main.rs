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
    /// Render a project file to a PNG frame
    Render {
        /// Path to a .toml project file
        project: PathBuf,

        /// Output PNG path
        #[arg(short, long, default_value = "frame_0000.png")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    match cli.cmd {
        Command::Render { project, output } => {
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
            let frame = compositor::render_frame(&proj, 0);
            match export::write_png(&frame, &output) {
                Ok(()) => println!(
                    "rendered {}×{} → {}",
                    frame.width,
                    frame.height,
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
