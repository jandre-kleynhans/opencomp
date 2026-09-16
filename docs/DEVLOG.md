# OpenComp — Development Log

> Chronological session records. NEWEST LAST.

## [2026-09-16] Session 4 — Phase 2 implemented (Tasks 1–9), Windows exe shipped

### What changed (all TDD, commit per task)

**Tasks 1–5 — keyframe engine:**
- `src/project.rs`: `AnimProp`, `Easing` (5 names, explicit serde renames for `ease_in_out`), `KeyTime` (frame int or `"MM:SS:FF"` untagged), `AnimValue` (custom deser: int-or-float scalar, `[x,y]` vec), `Keyframe`, `Expression`. `Layer` gains `keyframes` + `expressions` vecs (`[layer.keyframe]` / `[layer.expression]` tables).
- `src/keyframe.rs`: `time_to_frame` (MM:SS:FF → frames, FF = **frames** not seconds — found & fixed a wrong test expectation), `ease` (linear/cubic in/out/in-out/smoothstep/step), `resolve_value` (hold extrapolation both ends, left-key easing governs segment, step = hold left), `resolve_transform` (per-frame transform).
- `src/compositor.rs`: `render_frame` now resolves each layer's transform at the frame.

**Task 6 — expressions:** pure-Rust arithmetic evaluator (`+ - * /`, parens, unary minus, variable substitution `value/time/frame/fps/width/height/duration`; vector `[x,y]` with `value[0]`/`value[1]`). **ADR-0003** documents the decision to NOT pull rustpython-vm — v0.2 expressions are arithmetic-only; Python-level expressions are a later opt-in behind the same API.

**Task 7 — CLI:** `-f/--frame N` on `render` (default 0). Two integration tests: accepts flag, different frames → different PNGs.

**Task 8 — demo + docs:** `examples/demo.toml` now animates: right_card position (ease_in_out up-down) + rotation (±15°), center_strip opacity pulse + **rotation expression** (`frame * 2.0`). README Phase 2, `docs/PROJECT_FORMAT.md` v0.2 (keyframes, expressions, timecode, CLI).

**Task 9 — ADRs:** `0002-full-canvas-centering.md` (position ignored for full-canvas layers — keyframing it is a no-op by design), `0003-python-expressions.md` (pure-Rust arithmetic evaluator, no Python runtime dep).

### Windows exe (user request)

- Cross-compiled: `cargo build --release --target x86_64-pc-windows-gnu` (mingw installed, rustup target added).
- `dist/win/`: `opencomp.exe` (3.7 MB, valid PE32+, static — only system DLLs), `demo.toml`, `run_demo.bat` (renders frames 0/60/120, double-click).
- Verified: PE32+ structure via `file`/`objdump` (only kernel32/msvcrt/ntdll/WS2_32 etc.); same source passes 15 native test binaries; native Linux render works end-to-end.
- **Not yet run on real Windows** — PC offline at ship time (Tailscale `desktop-c511tl4` last seen 1h ago, LAN 192.168.1.13 unreachable). User will test.

### Pitfalls hit (future-proof)

1. **Raw strings + hex colors**: `r#"#ff0000"#` truncates at `"#` → must use `r##"..."##` (Phase 1 lesson, hit again in tests).
2. **serde `rename_all = "lowercase"`** gives `easeinout` not `ease_in_out` → explicit per-variant renames.
3. **Test wrongness vs code wrongness**: `00:01:00` is 1 second (24 frames @24fps) not 1 minute — fixed the TEST, kept the code (FF = frames).
4. **AnimValue untagged int-or-float**: TOML `value = 10` (int) → `Number(10.0)` via custom deser; `[10, 20]` ints → Vec2.
5. **vector expr `value[0]` substitution order**: replace `value[0]`/`value[1]` BEFORE bare `value` to avoid partial matches.
6. **`_frame` param shadowing**: `render_frame(proj, _frame)` → renamed to `frame` when time-aware.

### Test summary

15 test binaries green (~51 tests): 3 lib + 7 color + 3 compositor + 2 layer + 3 transform + 5 transform-stacked + 2 export + 4 CLI + 6 keyframe parse + 10 anim_time/easing + 9 keyframe_eval + 3 anim_composite + 9 expression.

### Git history (this session)

```
098d92e docs: ADR-0002/0003 + project format v0.2 (Task 9)
11bc54f feat: animated demo + README Phase 2 (Task 8)
a5eb367 feat: CLI --frame flag (Task 7)
9981a68 feat: Python expression evaluator + expression override (Task 6)
5f98bd5 feat: time-aware compositing (keyframed transforms)
d7933b8 feat: keyframe interpolation + per-property resolution (Task 4)
b42ed8c feat: timecode parsing + easing curves (Tasks 2-3)
1cd3c70 feat: keyframe + expression model (Task 1)
```

### Next

- **User tests `dist/win/run_demo.bat` on Windows** → feedback → fix
- Task 10: GitHub Issues (needs `gh` + auth)
- Push to GitHub (needs user to confirm repo still public / SSH push)
- Phase 3 planning: Python SDK + REST