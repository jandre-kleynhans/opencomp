"""
Phase 6 — Media I/O tests (RED first).

opencomp CLI gains:
  render <project> --all -o out.mp4       encode full composition to mp4
  render <project> -f N -o out.png        (already works)
Layer kinds: solid | video | image (parse TOML, render via ffmpeg/PIL)
"""
import json, os, subprocess, sys, tempfile, time
from pathlib import Path

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from opencomp import Project, Layer

# path to the Python CLI
CLI = [sys.executable, "-m", "opencomp.cli"]

SAMPLE = """
[project]
name = "sample"
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
[layer.transform]
position = [32.0, 24.0]
"""


@pytest.fixture
def proj_file(tmp_path):
    p = tmp_path / "sample.toml"
    p.write_text(SAMPLE)
    return p


def test_parse_video_layer(proj_file):
    src = """
[project]
name = "v"
fps = 24
width = 64
height = 48
duration = 10
bg_color = "#000000"
[[layer]]
name = "clip"
type = "video"
source = "assets/clip.mp4"
"""
    p = Project.from_toml(src)
    assert len(p.layers) == 1
    assert p.layers[0].kind == "video"
    assert p.layers[0].source == "assets/clip.mp4"


def test_parse_image_layer(proj_file):
    src = """
[project]
name = "v"
fps = 24
width = 64
height = 48
duration = 10
bg_color = "#000000"
[[layer]]
name = "img"
type = "image"
source = "assets/frame.png"
"""
    p = Project.from_toml(src)
    assert len(p.layers) == 1
    assert p.layers[0].kind == "image"
    assert p.layers[0].source == "assets/frame.png"


def test_render_all_encodes_mp4(proj_file, tmp_path):
    out = tmp_path / "out.mp4"
    r = subprocess.run(CLI + ["render", str(proj_file), "--all", "-o", str(out)],
                       capture_output=True, text=True, timeout=30)
    assert r.returncode == 0, r.stderr
    assert out.exists()
    assert out.stat().st_size > 1000  # at least some bytes
    # verify it's a valid mp4 container (ftyp box)
    data = out.read_bytes()
    assert b"ftyp" in data[:32]


def test_video_layer_renders(proj_file, tmp_path):
    """Video layers require actual file — skip if no sample. We just test parse."""
    src = """
[project]
name = "v"
fps = 24
width = 64
height = 48
duration = 10
bg_color = "#000000"
[[layer]]
name = "clip"
type = "video"
source = "missing.mp4"
"""
    p = Project.from_toml(src)
    assert p.layers[0].kind == "video"


def test_image_layer_renders(proj_file):
    """Image layers require actual file — test parse only."""
    src = """
[project]
name = "v"
fps = 24
width = 64
height = 48
duration = 10
bg_color = "#000000"
[[layer]]
name = "img"
type = "image"
source = "missing.png"
"""
    p = Project.from_toml(src)
    assert p.layers[0].kind == "image"