# ADR-0006: Media I/O — ffmpeg subprocess (don't reinvent codecs)

Status: Accepted
Date: 2026-09-16

## Context

Phase 6 requires media I/O: encode frames to h264/h265/ProRes/WebM, decode
video/image sequences as layer sources. The master plan explicitly says
"ffmpeg/libav — don't reinvent codecs".

## Decision

**The Rust CLI shells out to the system `ffmpeg` binary for encoding.**

- `opencomp render project.toml --all -o out.mp4` renders every frame to a temp
  PNG sequence, then pipes it through `ffmpeg -framerate <fps> -i frame_%04d.png
  -c:v libx264 ...` for h264 output.
- `video` / `image` layer kinds are part of the project format; their `source`
  field is stored but full decode is the Python SDK's job (PIL/ffmpeg) in the
  current milestone — the Rust core stores and validates the kind/source.
- Codec choices arrive as CLI flags in a follow-up (`--codec h264|hevc|prores|webm`).

Why subprocess: libav bindings (`ffmpeg-next`) drag in FFI system deps and
version-matching pain; `ffmpeg` is already on the system and battle-tested.

## Consequences

- Encoding works today (verified: demo → 640×480 h264 mp4, 5.0s, 120 frames).
- The core stays dependency-light; ffmpeg is an optional runtime tool.
- Framerate comes from the project `fps`, so timecode stays frame-accurate.
- Windows/macOS users must have ffmpeg in PATH (bundling is a later dist task).