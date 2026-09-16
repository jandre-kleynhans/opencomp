"""
opencomp CLI tests (Phase 3) — render / preview / frame / serve.

The CLI is the Python entry point that makes the engine agent-usable from a
shell: `opencomp render p.toml`, `opencomp frame p.toml -n 42`,
`opencomp preview p.toml` (writes a preview strip), `opencomp serve` (REST).
"""
import os, subprocess, sys, tempfile, json
from pathlib import Path

import pytest

# Path to the CLI module
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


def test_render_writes_png(proj_file, tmp_path):
    # render with -f writes a single PNG file
    out = tmp_path / "out.png"
    r = subprocess.run(CLI + ["render", str(proj_file), "-f", "0", "-o", str(out)],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    assert out.exists()
    assert out.read_bytes()[:8] == b"\x89PNG\r\n\x1a\n"


def test_render_all_writes_sequence_dir(proj_file, tmp_path):
    # render without -f writes a frame sequence directory
    outdir = tmp_path / "seq"
    r = subprocess.run(CLI + ["render", str(proj_file), "-o", str(outdir)],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    assert outdir.is_dir()
    assert len(list(outdir.glob("frame_*.png"))) == 10  # duration


def test_frame_flag(proj_file, tmp_path):
    out = tmp_path / "f10.png"
    r = subprocess.run(CLI + ["frame", str(proj_file), "-n", "10", "-o", str(out)],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    assert out.exists()


def test_preview_writes_strip(proj_file, tmp_path):
    out = tmp_path / "preview.png"
    r = subprocess.run(CLI + ["preview", str(proj_file), "-o", str(out), "--frames", "4"],
                       capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    assert out.exists()


def test_serve_health():
    """serve starts, answers /health, then exits."""
    import socket
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()

    proc = subprocess.Popen(CLI + ["serve", "--port", str(port)],
                            stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    try:
        # poll /health
        import urllib.request
        import time
        for _ in range(50):
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/health", timeout=0.5) as resp:
                    assert resp.status == 200
                    assert json.loads(resp.read())["status"] == "ok"
                    break
            except Exception:
                time.sleep(0.1)
        else:
            pytest.fail("serve did not become healthy")
    finally:
        proc.terminate()
        try:
            proc.wait(timeout=5)
        except subprocess.TimeoutExpired:
            proc.kill()