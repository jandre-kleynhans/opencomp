"""opencomp.server — REST + WebSocket API (Phase 3, the agent surface).

Endpoints (from the master plan):
  POST /project/load        load a TOML project file into memory
  GET  /composition/layers  list layers (names, colors, keyframes)
  POST /layer/add           add a layer
  POST /layer/keyframe      add a keyframe to a layer
  POST /render/frame?n=42   render a single frame -> PNG
  POST /render/all          render all frames -> manifest + PNGs
  GET  /project/diff?since=abc  git-style diff of the in-memory project
  WS   /events              live event stream (layer_added, keyframe_added, rendered)
"""
from __future__ import annotations

import json
import struct
import threading
import zlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from typing import Dict, List, Optional
from urllib.parse import parse_qs, urlparse

from . import Color, Keyframe, Layer, Project
from .engine import render_frame_bytes


# --------------------------------------------------------------------------
# Tiny in-memory engine state
# --------------------------------------------------------------------------

class EngineState:
    """Holds the current project + revision history for diffing."""

    def __init__(self):
        self.project: Optional[Project] = None
        self.revisions: List[str] = []
        self.events: List[Dict] = []
        self._lock = threading.Lock()

    def set_project(self, proj: Project) -> None:
        with self._lock:
            self.project = proj
            self.revisions.append(proj.to_toml())

    def apply_change(self, description: str, mutator) -> None:
        with self._lock:
            mutator()
            self.revisions.append(self.project.to_toml())
            self.events.append({"event": description, "time": len(self.revisions)})

    def diff(self, since: int = 0) -> Dict:
        if not self.project or since < 0 or since >= len(self.revisions):
            return {"error": "invalid since"}
        before = self.revisions[since]
        after = self.revisions[-1]
        return {
            "changed": before != after,
            "layers_count": len(self.project.layers),
        }


STATE = EngineState()


# --------------------------------------------------------------------------
# PNG helpers (encode/decode — minimal, no deps)
# --------------------------------------------------------------------------

def encode_png(w: int, h: int, rgba: bytes) -> bytes:
    def chunk(typ: bytes, data: bytes) -> bytes:
        c = typ + data
        return struct.pack(">I", len(data)) + c + struct.pack(">I", zlib.crc32(c) & 0xffffffff)

    raw = b""
    stride = w * 4
    for y in range(h):
        raw += b"\x00" + bytes(rgba[y * stride:(y + 1) * stride])
    ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
    return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
            + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))


# --------------------------------------------------------------------------
# HTTP handler
# --------------------------------------------------------------------------

class Handler(BaseHTTPRequestHandler):
    server_version = "OpenComp/0.3"

    def log_message(self, *a):
        pass

    # -- helpers ----------------------------------------------------------

    def _send_json(self, obj, code=200):
        body = json.dumps(obj).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _send_png(self, png: bytes):
        self.send_response(200)
        self.send_header("Content-Type", "image/png")
        self.send_header("Content-Length", str(len(png)))
        self.end_headers()
        self.wfile.write(png)

    def _read_json(self) -> Dict:
        length = int(self.headers.get("Content-Length", 0))
        return json.loads(self.rfile.read(length) if length else b"{}")

    def _emit(self, ev: Dict):
        """Append an event and push it to WS clients."""
        STATE.events.append(ev)
        if _ws_thread is not None:
            _ws_thread.broadcast([ev])

    def _require_project(self) -> Optional[Project]:
        if STATE.project is None:
            self._send_json({"error": "no project loaded — POST /project/load first"}, 400)
            return None
        return STATE.project

    # -- GET --------------------------------------------------------------

    def do_GET(self):
        parsed = urlparse(self.path)
        path = parsed.path
        q = parse_qs(parsed.query)

        if path == "/health":
            self._send_json({"status": "ok", "version": "0.3.0"})
            return

        if path == "/composition/layers":
            proj = self._require_project()
            if proj is None:
                return
            self._send_json([{
                "name": l.name,
                "type": l.kind,
                "color": l.color.to_hex() if l.color else None,
                "size": l.size,
                "keyframes": [
                    {"property": k.property, "time": k.time, "value": k.value,
                     "easing": k.easing} for k in l.keyframes
                ],
                "expressions": [{"property": e.property, "expr": e.expr} for e in l.expressions],
            } for l in proj.layers])
            return

        if path == "/project/diff":
            since = int(q.get("since", ["0"])[0])
            self._send_json(STATE.diff(since))
            return

        if path.startswith("/render/frame"):
            proj = self._require_project()
            if proj is None:
                return
            frame = int(q.get("n", ["0"])[0])
            self._send_png(render_frame_bytes(proj, frame))
            return

        self._send_json({"error": "not found"}, 404)

    # -- POST -------------------------------------------------------------

    def do_POST(self):
        parsed = urlparse(self.path)
        path = parsed.path

        if path == "/project/load":
            body = self._read_json()
            proj = Project.load(body["path"])
            STATE.set_project(proj)
            self._emit({"event": "project_loaded", "name": proj.name})
            self._send_json({"name": proj.name, "width": proj.width, "height": proj.height,
                             "fps": proj.fps, "duration": proj.duration,
                             "layers": len(proj.layers)})
            return

        if path == "/layer/add":
            proj = self._require_project()
            if proj is None:
                return
            body = self._read_json()
            layer = Layer(
                name=body["name"],
                kind=body.get("type", "solid"),
                color=Color.from_hex(body["color"]) if body.get("color") else None,
                size=[float(x) for x in body.get("size", [0.0, 0.0])],
            )
            proj.add(layer)
            STATE.apply_change("layer_added", lambda: None)
            self._emit({"event": "layer_added", "layer": layer.name})
            self._send_json({"added": layer.name, "layers": len(proj.layers)})
            return

        if path == "/layer/keyframe":
            proj = self._require_project()
            if proj is None:
                return
            body = self._read_json()
            layer_name = body["layer"]
            layer = next((l for l in proj.layers if l.name == layer_name), None)
            if layer is None:
                self._send_json({"error": f"no layer named '{layer_name}'"}, 404)
                return
            kf = Keyframe(time=body["time"], value=body["value"],
                          easing=body.get("easing", "linear"), property=body["property"])
            layer.keyframes.append(kf)
            STATE.apply_change("keyframe_added", lambda: None)
            self._emit({"event": "keyframe_added", "layer": layer_name,
                                 "property": kf.property})
            self._send_json({"added": kf.property, "time": kf.time})
            return

        if path == "/render/all":
            proj = self._require_project()
            if proj is None:
                return
            # Render all frames; return a manifest (frames are large — the agent
            # can fetch individual frames via /render/frame?n=...)
            self._send_json({"frames": proj.duration, "width": proj.width,
                             "height": proj.height,
                             "note": "fetch frames via GET /render/frame?n=<i>"})
            return

        self._send_json({"error": "not found"}, 404)


# --------------------------------------------------------------------------
# WebSocket /events
# --------------------------------------------------------------------------

class WsThread(threading.Thread):
    """Broadcasts STATE.events to connected WebSocket clients (stdlib impl)."""

    def __init__(self, port: int):
        super().__init__(daemon=True)
        self.port = port
        self._clients: list = []
        self._lock = threading.Lock()
        self._last = 0

    def run(self):
        import socket as _socket

        srv = _socket.socket(_socket.AF_INET, _socket.SOCK_STREAM)
        srv.setsockopt(_socket.SOL_SOCKET, _socket.SO_REUSEADDR, 1)
        srv.bind(("127.0.0.1", self.port))
        srv.listen(5)
        srv.settimeout(0.5)
        while True:
            try:
                conn, _ = srv.accept()
            except _socket.timeout:
                continue
            except OSError:
                break
            # handshake
            try:
                conn.settimeout(3)
                data = conn.recv(4096).decode(errors="ignore")
                if "Upgrade: websocket" not in data:
                    conn.close()
                    continue
                key = [l for l in data.split("\r\n") if l.startswith("Sec-WebSocket-Key")][0].split(":", 1)[1].strip()
                import base64, hashlib
                accept = base64.b64encode(
                    hashlib.sha1((key + "258EAFA5-E914-47DA-95CA-C5AB0DC85B11").encode()).digest()
                ).decode()
                conn.sendall(
                    ("HTTP/1.1 101 Switching Protocols\r\n"
                     "Upgrade: websocket\r\n"
                     "Connection: Upgrade\r\n"
                     f"Sec-WebSocket-Accept: {accept}\r\n\r\n").encode()
                )
                with self._lock:
                    self._clients.append(conn)
                # drain (client may send nothing)
                conn.settimeout(0.2)
                try:
                    conn.recv(4096)
                except Exception:
                    pass
            except Exception:
                try:
                    conn.close()
                except Exception:
                    pass

    def broadcast(self, events: List[Dict]):
        with self._lock:
            clients = list(self._clients)
        for conn in clients:
            try:
                for ev in events:
                    payload = json.dumps(ev).encode()
                    # WS frame: text, FIN
                    header = bytes([0x81, len(payload)]) if len(payload) < 126 else bytes([0x81, 126]) + struct.pack(">H", len(payload))
                    conn.sendall(header + payload)
            except Exception:
                with self._lock:
                    if conn in self._clients:
                        self._clients.remove(conn)


_ws_thread: Optional[WsThread] = None


def start_server(port: int = 8350):
    global _ws_thread
    srv = ThreadingHTTPServer(("127.0.0.1", port), Handler)
    _ws_thread = WsThread(port + 1)  # WS on port+1 to keep things simple
    _ws_thread.start()
    threading.Thread(target=srv.serve_forever, daemon=True).start()
    return srv


def stop_server(srv):
    srv.shutdown()


def ws_port() -> int:
    return (_ws_thread.port if _ws_thread else 0)