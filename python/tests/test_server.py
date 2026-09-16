"""
REST API tests (Phase 3) — the agent surface from the master plan.

POST /project/load · GET /composition/layers · POST /layer/add
POST /layer/keyframe · POST /render/frame?n=42 · POST /render/all
GET /project/diff?since=abc · WS /events
"""
import json, os, subprocess, sys, tempfile, time
from pathlib import Path
import urllib.request

import pytest

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))
from opencomp import Project, Layer, Keyframe, AnimProp
from opencomp.server import start_server, stop_server

SAMPLE_PROJECT = """
[project]
name = "rest"
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
def server():
    import socket
    s = socket.socket()
    s.bind(("127.0.0.1", 0))
    port = s.getsockname()[1]
    s.close()
    srv = start_server(port)
    yield f"http://127.0.0.1:{port}", srv, port
    stop_server(srv)


@pytest.fixture
def loaded_project(server):
    """Load a project so layer/keyframe endpoints work."""
    base, srv, port = server
    p = Path(tempfile.mkdtemp()) / "p.toml"
    p.write_text(SAMPLE_PROJECT)
    http(base + "/project/load", method="POST",
         data=json.dumps({"path": str(p)}).encode(),
         headers={"Content-Type": "application/json"})
    return base, srv, port


def http(url, method="GET", data=None, headers=None):
    req = urllib.request.Request(url, method=method, data=data, headers=headers or {})
    try:
        with urllib.request.urlopen(req, timeout=5) as resp:
            return resp.status, resp.read()
    except urllib.error.HTTPError as e:
        return e.code, e.read()


def test_health(server):
    base, _, _ = server
    st, body = http(base + "/health")
    assert st == 200
    assert json.loads(body)["status"] == "ok"


def test_load_project(server, tmp_path):
    base, _, _ = server
    path = tmp_path / "p.toml"
    path.write_text(SAMPLE_PROJECT)
    st, body = http(base + "/project/load", method="POST",
                    data=json.dumps({"path": str(path)}).encode(),
                    headers={"Content-Type": "application/json"})
    assert st == 200
    d = json.loads(body)
    assert d["name"] == "rest"
    assert d["width"] == 64


def test_composition_layers(server, tmp_path):
    base, _, _ = server
    path = tmp_path / "p.toml"
    path.write_text(SAMPLE_PROJECT)
    http(base + "/project/load", method="POST",
         data=json.dumps({"path": str(path)}).encode(),
         headers={"Content-Type": "application/json"})
    st, body = http(base + "/composition/layers")
    assert st == 200
    layers = json.loads(body)
    assert len(layers) == 1
    assert layers[0]["name"] == "red"


def test_layer_add(loaded_project):
    base, _, _ = loaded_project
    st, body = http(base + "/layer/add", method="POST",
                    data=json.dumps({"name": "sq", "type": "solid",
                                     "color": "#00ff00", "size": [10, 10]}).encode(),
                    headers={"Content-Type": "application/json"})
    assert st == 200
    layers = json.loads(http(base + "/composition/layers")[1])
    assert len(layers) == 2  # original red + sq
    assert layers[1]["color"] == "#00ff00"


def test_layer_keyframe(loaded_project):
    base, _, _ = loaded_project
    http(base + "/layer/add", method="POST",
         data=json.dumps({"name": "sq", "type": "solid", "color": "#ff0000"}).encode(),
         headers={"Content-Type": "application/json"})
    st, body = http(base + "/layer/keyframe", method="POST",
                    data=json.dumps({"layer": "sq", "property": "position",
                                     "time": 0, "value": [0, 0]}).encode(),
                    headers={"Content-Type": "application/json"})
    assert st == 200
    st, body = http(base + "/layer/keyframe", method="POST",
                    data=json.dumps({"layer": "sq", "property": "position",
                                     "time": 24, "value": [100, 100],
                                     "easing": "ease_in_out"}).encode(),
                    headers={"Content-Type": "application/json"})
    assert st == 200
    layers = json.loads(http(base + "/composition/layers")[1])
    kfs = layers[1]["keyframes"]  # sq is the second layer (red is first)
    assert len(kfs) == 2
    assert kfs[1]["easing"] == "ease_in_out"


def test_render_frame_endpoint(loaded_project):
    base, _, _ = loaded_project
    st, body = http(base + "/render/frame?n=3")
    assert st == 200
    assert body[:8] == b"\x89PNG\r\n\x1a\n"


def test_render_all_endpoint(loaded_project):
    base, _, _ = loaded_project
    st, body = http(base + "/render/all", method="POST", data=b"{}")
    assert st == 200
    # returns JSON manifest with frame count
    d = json.loads(body)
    assert d["frames"] >= 1


def test_project_diff(server, tmp_path):
    base, _, _ = server
    p = tmp_path / "p.toml"
    p.write_text(SAMPLE_PROJECT)
    http(base + "/project/load", method="POST",
         data=json.dumps({"path": str(p)}).encode(),
         headers={"Content-Type": "application/json"})
    http(base + "/layer/add", method="POST",
         data=json.dumps({"name": "new", "type": "solid", "color": "#123456"}).encode(),
         headers={"Content-Type": "application/json"})
    st, body = http(base + "/project/diff?since=0")
    assert st == 200
    diff = json.loads(body)
    assert diff["changed"] is True
    assert "layers_count" in diff


def test_ws_events(server):
    """WS /events pushes a message when a layer is added."""
    base, srv, port = server
    import websockets.sync.client as ws_client
    from opencomp.server import ws_port
    with ws_client.connect(f"ws://127.0.0.1:{ws_port()}/events", timeout=3) as ws:
        # need a project loaded first
        p = Path(tempfile.mkdtemp()) / "p.toml"
        p.write_text(SAMPLE_PROJECT)
        http(base + "/project/load", method="POST",
             data=json.dumps({"path": str(p)}).encode(),
             headers={"Content-Type": "application/json"})
        # add layer on REST -> should emit event(s); first is project_loaded, then layer_added
        http(base + "/layer/add", method="POST",
             data=json.dumps({"name": "evt", "type": "solid", "color": "#ffffff"}).encode(),
             headers={"Content-Type": "application/json"})
        msg = ws.recv(timeout=3)  # project_loaded
        assert json.loads(msg)["event"] == "project_loaded"
        msg = ws.recv(timeout=3)  # layer_added
        d = json.loads(msg)
        assert d["event"] == "layer_added"
        assert d["layer"] == "evt"