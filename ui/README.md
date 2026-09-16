# OpenComp UI (Phase 5) — Timeline + Canvas + REST

This is a self-contained single-file UI for OpenComp. It talks to the OpenComp
REST server (`opencomp serve` or Python `opencomp -m opencomp.server`) and
provides:

- **Canvas viewport** — renders the current frame via `/render/frame?n=`
- **Layer list** — add/remove layers, click to select
- **Timeline** — ruler, playhead, lanes per layer/property, keyframe diamonds
- **Keyframe editing** — click a lane to add, click a diamond to edit time/value/easing, delete
- **Playback** — play/pause, frame scrubber, duration
- **Save TOML** — exports the full project (ready for `opencomp render`)

## How to use on matrix (Windows)

1. **Start the REST server** (from your opencomp repo on the mothership, or from a terminal on matrix if you copied the python/ dir):
   ```bash
   cd python
   python -m opencomp.server --port 8350
   ```
   Or from the Rust side: the CLI doesn't have `serve` yet — use the Python module.

2. **Open the UI**: double-click `opencomp_ui.html` on your Desktop (Edge/Firefox). It defaults to `http://127.0.0.1:8350`. Click **Connect**.

3. **Animate**:
   - Click **+ Layer** to add a solid layer.
   - Click a layer to select it.
   - In the timeline, click a **lane** (position/scale/opacity/rotation) at the current frame — a yellow keyframe diamond appears.
   - Click the diamond → edit time/value/easing in the bottom panel.
   - Drag the playhead or press **Play** to scrub.
   - The canvas renders the current frame via `/render/frame?n=`.

4. **Save** → downloads a `project.toml` you can render with the Rust CLI:
   ```bash
   opencomp.exe render project.toml -f 60 -o frame_0060.png
   ```

## Requirements

- The OpenComp REST server running at `http://127.0.0.1:8350` (or change the Server URL in the UI).
- The Python SDK installed (the server is in `python/opencomp/server.py`).
- The Rust `opencomp.exe` binary available for the `engine.render_frame_bytes` calls (the server uses it).

## Files on matrix

- `C:\Users\Public\Desktop\opencomp_ui.html` — this UI
- `C:\Users\Public\Desktop\opencomp\opencomp.exe` — the renderer
- `C:\Users\Public\Desktop\opencomp\demo.toml` — the animated demo
- `C:\Users\Public\Desktop\opencomp\run_demo.bat` — batch render script