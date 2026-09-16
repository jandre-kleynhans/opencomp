"""opencomp.cli — Python command-line interface (Phase 3).

Subcommands:
  render   <project.toml> [-f N] [-o out.png]   render one frame
  frame    <project.toml> -n N [-o out.png]     render a specific frame
  preview  <project.toml> [-o strip.png] [--frames N]  render a contact strip
  serve    --port N                             REST API

All rendering is delegated to the Rust core binary via opencomp.engine.
"""
from __future__ import annotations

import argparse
import json
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import List

from .engine import _find_binary, render_frame_bytes
from . import Project


def _render_to(proj: Project, frame: int, out: Path) -> None:
    out = Path(out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_bytes(render_frame_bytes(proj, frame))


def cmd_render(args) -> int:
    proj = Project.load(args.project)
    if args.all:
        # delegate full render+encode to the Rust binary (ffmpeg)
        bin_path = _find_binary()
        subprocess.run(
            [str(bin_path), "render", str(args.project), "--all", "-o", str(args.output)],
            check=True,
        )
        print(f"rendered all frames -> {args.output}")
        return 0
    if args.frame is not None:
        _render_to(proj, args.frame, args.output)
        print(f"rendered frame {args.frame} -> {args.output}")
    else:
        # render all frames into a sequence (ffmpeg-able)
        out_dir = Path(args.output)
        out_dir.mkdir(parents=True, exist_ok=True)
        for f in range(proj.duration):
            out = out_dir / f"frame_{f:04d}.png"
            _render_to(proj, f, out)
        print(f"rendered {proj.duration} frames -> {out_dir}/frame_*.png")
    return 0


def cmd_frame(args) -> int:
    proj = Project.load(args.project)
    _render_to(proj, args.n, args.output)
    print(f"rendered frame {args.n} -> {args.output}")
    return 0


def cmd_preview(args) -> int:
    proj = Project.load(args.project)
    import struct, zlib

    def decode_png(data: bytes):
        # minimal PNG decode for RGBA8 (no interlacing)
        pos = 8
        idat = b""
        w = h = 0
        while pos < len(data):
            ln = struct.unpack(">I", data[pos:pos + 4])[0]
            typ = data[pos + 4:pos + 8]
            chunk = data[pos + 8:pos + 8 + ln]
            if typ == b"IHDR":
                w, h = struct.unpack(">II", chunk[:8])
            elif typ == b"IDAT":
                idat += chunk
            pos += 12 + ln
        raw = zlib.decompress(idat)
        stride = w * 4
        out = bytearray()
        prev = bytearray(stride)
        i = 0
        for _ in range(h):
            f = raw[i]; i += 1
            line = bytearray(raw[i:i + stride]); i += stride
            if f == 1:
                for x in range(4, stride): line[x] = (line[x] + line[x - 4]) & 0xff
            elif f == 2:
                for x in range(stride): line[x] = (line[x] + prev[x]) & 0xff
            elif f == 3:
                for x in range(stride):
                    a = line[x - 4] if x >= 4 else 0
                    line[x] = (line[x] + ((a + prev[x]) >> 1)) & 0xff
            elif f == 4:
                for x in range(stride):
                    a = line[x - 4] if x >= 4 else 0
                    b = prev[x]; c = prev[x - 4] if x >= 4 else 0
                    p = a + b - c
                    pa, pb, pc = abs(p - a), abs(p - b), abs(p - c)
                    pr = a if (pa <= pb and pa <= pc) else (b if pb <= pc else c)
                    line[x] = (line[x] + pr) & 0xff
            out += line
            prev = line
        return w, h, bytes(out)

    n_frames = args.frames
    frames = []
    for f in range(0, proj.duration, max(1, proj.duration // n_frames)):
        frames.append(decode_png(render_frame_bytes(proj, f)))
    # build strip: stack frames side by side
    fw, fh = frames[0][0], frames[0][1]
    gap = 4
    tw = fw * len(frames) + gap * (len(frames) - 1)
    th = fh
    strip = bytearray([0] * (tw * th * 4))
    for i, (w, h, px) in enumerate(frames):
        x0 = i * (fw + gap)
        for y in range(h):
            src = y * w * 4
            dst = (y * tw + x0) * 4
            strip[dst:dst + w * 4] = px[src:src + w * 4]
    # encode PNG
    def encode_png(w, h, rgba):
        def chunk(typ, data):
            c = typ + data
            return struct.pack(">I", len(data)) + c + struct.pack(">I", zlib.crc32(c) & 0xffffffff)
        raw = b""
        stride = w * 4
        prev = bytearray(stride)
        for y in range(h):
            line = bytes(rgba[y * stride:(y + 1) * stride])
            # filter 0 (none)
            raw += b"\x00" + line
            prev = line
        ihdr = struct.pack(">IIBBBBB", w, h, 8, 6, 0, 0, 0)
        return (b"\x89PNG\r\n\x1a\n" + chunk(b"IHDR", ihdr)
                + chunk(b"IDAT", zlib.compress(raw)) + chunk(b"IEND", b""))

    Path(args.output).parent.mkdir(parents=True, exist_ok=True)
    Path(args.output).write_bytes(encode_png(tw, th, bytes(strip)))
    print(f"preview: {len(frames)} frames -> {args.output}")
    return 0


def cmd_serve(args) -> int:
    from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
    import urllib.parse

    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *a):
            pass

        def _json(self, obj, code=200):
            body = json.dumps(obj).encode()
            self.send_response(code)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)

        def do_GET(self):
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == "/health":
                self._json({"status": "ok", "version": "0.3.0"})
                return
            if parsed.path.startswith("/render"):
                # /render?project=...&frame=42
                q = urllib.parse.parse_qs(parsed.query)
                proj = Project.load(q["project"][0])
                frame = int(q.get("frame", ["0"])[0])
                png = render_frame_bytes(proj, frame)
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Content-Length", str(len(png)))
                self.end_headers()
                self.wfile.write(png)
                return
            self._json({"error": "not found"}, 404)

        def do_POST(self):
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == "/render":
                body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", 0))))
                proj = Project.from_toml(body["project"])
                png = render_frame_bytes(proj, body.get("frame", 0))
                self.send_response(200)
                self.send_header("Content-Type", "image/png")
                self.send_header("Content-Length", str(len(png)))
                self.end_headers()
                self.wfile.write(png)
                return
            self._json({"error": "not found"}, 404)

    srv = ThreadingHTTPServer(("127.0.0.1", args.port), Handler)
    print(f"opencomp serve listening on http://127.0.0.1:{args.port}")
    srv.serve_forever()


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(prog="opencomp", description="OpenComp Python CLI (Phase 3)")
    sub = p.add_subparsers(dest="cmd", required=True)

    r = sub.add_parser("render", help="render a project (one frame, all frames, or mp4)")
    r.add_argument("project")
    r.add_argument("-f", "--frame", type=int, default=None, help="frame number (default: all frames)")
    r.add_argument("-o", "--output", default="out.png")
    r.add_argument("--all", action="store_true", help="render all frames and encode to mp4 via ffmpeg")
    r.set_defaults(func=cmd_render)

    f = sub.add_parser("frame", help="render a specific frame")
    f.add_argument("project")
    f.add_argument("-n", "--number", type=int, required=True, dest="n")
    f.add_argument("-o", "--output", default="frame.png")
    f.set_defaults(func=cmd_frame)

    pv = sub.add_parser("preview", help="render a contact strip preview")
    pv.add_argument("project")
    pv.add_argument("-o", "--output", default="preview.png")
    pv.add_argument("--frames", type=int, default=6)
    pv.set_defaults(func=cmd_preview)

    s = sub.add_parser("serve", help="run the REST API")
    s.add_argument("--port", type=int, default=8350)
    s.set_defaults(func=cmd_serve)

    return p


def main(argv: List[str] | None = None) -> int:
    p = build_parser()
    args = p.parse_args(argv)
    return args.func(args)


if __name__ == "__main__":
    sys.exit(main())