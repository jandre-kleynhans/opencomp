"""opencomp.engine — render frames by shelling out to the Rust core.

The Rust binary (opencomp) is the authoritative renderer. This module finds
it, writes a temp project when needed, and returns PNG bytes.
"""
from __future__ import annotations

import os
import shutil
import subprocess
import tempfile
from pathlib import Path
from typing import Optional

from . import Project


def _find_binary() -> Path:
    """Locate the opencomp Rust binary: env override, PATH, or repo target."""
    env = os.environ.get("OPENCOMP_BIN")
    if env:
        return Path(env)
    which = shutil.which("opencomp")
    if which:
        return Path(which)
    # Fallback: repo release build (dev convenience)
    for rel in ("target/release/opencomp", "target/debug/opencomp"):
        p = Path(__file__).resolve().parent.parent.parent / rel
        if p.exists():
            return p
    raise FileNotFoundError(
        "opencomp binary not found. Set OPENCOMP_BIN or build with cargo build --release."
    )


def render_frame_bytes(proj: Project, frame: int, binary: Optional[Path] = None) -> bytes:
    """Render one frame of a Project to PNG bytes."""
    bin_path = binary or _find_binary()
    with tempfile.TemporaryDirectory() as d:
        proj_file = Path(d) / "project.toml"
        out_file = Path(d) / "frame.png"
        proj.save(proj_file)
        subprocess.run(
            [str(bin_path), "render", str(proj_file), "-f", str(frame), "-o", str(out_file)],
            check=True,
            capture_output=True,
        )
        return out_file.read_bytes()


def render_frame(proj: Project, frame: int, output: Path, binary: Optional[Path] = None) -> None:
    """Render one frame to a file path."""
    Path(output).write_bytes(render_frame_bytes(proj, frame, binary))