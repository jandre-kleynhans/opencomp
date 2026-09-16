# ADR-0005: Web UI — single-file canvas/timeline app (headless-first)

Status: Accepted
Date: 2026-09-16

## Context

Phase 5 calls for a Web UI (canvas viewport, layer list, timeline) that is
headless-first: the UI talks to the same REST API an agent would. No native
desktop app; no Electron/Tauri for v0.5 — a single self-contained HTML/JS
file is the MVP.

## Decision

**The UI is a single `ui/index.html` (vanilla JS, no build step).**

- Loads a project from the REST API (`GET /composition/layers`, `POST /layer/add`,
  `POST /layer/keyframe`, `GET /render/frame?n=`).
- Canvas viewport renders the current frame via the REST API.
- Timeline shows a ruler, playhead, lanes per layer/property, and keyframe
  diamonds (add/click/drag/edit/delete).
- Easing picker per keyframe (linear, ease_in/out/in_out, step).
- Save button downloads the full TOML for offline rendering.

**Why not a framework?** Zero build, zero deps, instant on matrix (Edge/Firefox),
runs headless (same API surface agents use). If a richer UI is needed later
(Phase 5 follow-up), the contract stays the REST API; the UI can be swapped.

## Consequences

- No build step, no npm, no node_modules — single file.
- Runs in any browser on matrix (Edge, Firefox).
- Fully headless-first: every interaction hits the REST API, so the same
  endpoint an agent uses.
- The UI is *not* a replacement for the engine — it's a thin client.