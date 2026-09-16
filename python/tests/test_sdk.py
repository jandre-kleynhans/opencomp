"""
opencomp — Python SDK tests (Phase 3, TDD).

The SDK is a thin, typed wrapper around the TOML project format. It lets
agents load/create projects, add layers and keyframes, and render frames —
all without hand-editing TOML. It is the "agent surface" of OpenComp.
"""
import tempfile, os
from pathlib import Path
import pytest

from opencomp import Project, Layer, Keyframe, Color, AnimProp, Easing
from opencomp.engine import render_frame_bytes


# ---------------------------------------------------------------- fixtures

@pytest.fixture
def proj():
    p = Project(name="test", width=100, height=100)
    return p


def test_project_defaults(proj):
    assert proj.name == "test"
    assert proj.fps == 24
    assert proj.width == 100
    assert proj.height == 100
    assert proj.bg_color == Color(10, 10, 10)
    assert proj.duration == 120
    assert proj.layers == []


def test_add_layer(proj):
    layer = Layer(name="red", kind="solid", color="#ff0000", size=[20, 20])
    layer.transform.position = [50, 50]
    proj.add(layer)
    assert len(proj.layers) == 1
    assert proj.layers[0].name == "red"
    assert proj.layers[0].color.r == 255


def test_layer_keyframes(proj):
    layer = Layer(name="sq", kind="solid", color="#00ff00")
    layer.animate(AnimProp.POSITION, [
        Keyframe(time=0, value=[0, 0]),
        Keyframe(time=24, value=[100, 100], easing="ease_in_out"),
    ])
    assert len(layer.keyframes) == 2
    assert layer.keyframes[0].property == "position"
    assert layer.keyframes[0].value == [0, 0]
    assert layer.keyframes[1].easing == "ease_in_out"


def test_project_to_toml_roundtrip(proj):
    layer = Layer(name="sq", kind="solid", color="#ff0000", size=[10, 10])
    layer.animate(AnimProp.OPACITY, [
        Keyframe(time=0, value=0),
        Keyframe(time=24, value=100, easing="ease_in"),
    ])
    proj.add(layer)
    toml_str = proj.to_toml()
    # parse back
    p2 = Project.from_toml(toml_str)
    assert p2.name == "test"
    assert len(p2.layers) == 1
    assert p2.layers[0].keyframes[0].value == 0
    assert p2.layers[0].keyframes[1].easing == "ease_in"


def test_project_save_load(proj):
    layer = Layer(name="sq", kind="solid", color="#0000ff")
    proj.add(layer)
    with tempfile.TemporaryDirectory() as d:
        path = Path(d) / "proj.toml"
        proj.save(path)
        p2 = Project.load(path)
        assert p2 == proj


def test_render_frame_png_bytes(proj):
    layer = Layer(name="sq", kind="solid", color="#ff0000", size=[20, 20])
    layer.transform.position = [50, 50]
    proj.add(layer)
    png = render_frame_bytes(proj, 0)
    assert png[:8] == b"\x89PNG\r\n\x1a\n"  # PNG magic
    assert len(png) > 100


def test_render_frame_animated_different_pngs():
    p = Project(name="anim", width=64, height=64)
    layer = Layer(name="sq", kind="solid", color="#ff0000", size=[10, 10])
    layer.animate(AnimProp.POSITION, [
        Keyframe(time=0, value=[30, 30]),
        Keyframe(time=10, value=[60, 30]),
    ])
    p.add(layer)
    f0 = render_frame_bytes(p, 0)
    f10 = render_frame_bytes(p, 10)
    assert f0 != f10