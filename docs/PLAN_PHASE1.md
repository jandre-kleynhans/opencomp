# Phase 1: Compositing Core — Implementation Plan

> **For Hermes:** Follow `test-driven-development`: RED → GREEN per task, verify failure before implementation, commit after each task.
> Full context: `docs/ARCHITECTURE.md`, `docs/PROJECT_FORMAT.md`, `docs/ROADMAP.md`.

**Goal:** A Rust binary `opencomp` that reads a TOML project and renders a PNG frame.

**Architecture:** `project.rs` parses the TOML into serde structs (the file-format contract). `compositor.rs` resolves the layer stack into pixels. `main.rs` wires CLI → parse → compose → PNG. No GPU yet — Phase 1 is a correct CPU rasterizer that proves the pipeline; wgpu comes later.

**Tech stack:** Rust (edition 2021), deps: `serde` / `toml` / `png` / `clap` (CLI). TDD with Rust's built-in test harness. Format contract per `docs/PROJECT_FORMAT.md`.

---

### Task 1: Scaffold Cargo project + deps

**Files:**
- Create: `Cargo.toml`, `src/main.rs`, `tests/` directory

**Steps:**
1. `cargo new opencomp` (or scaffold by hand if cargo not ready — structure: `Cargo.toml`, `src/main.rs`, `tests/`)
2. `Cargo.toml`:
```toml
[package]
name = "opencomp"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1", features = ["derive"] }
toml = "0.8"
png = "0.17"
clap = { version = "4", features = ["derive"] }
```
3. `src/main.rs` — placeholder `fn main() { println!("opencomp: phase 1"); }`
4. `cargo build` → succeeds
5. Commit: `chore: scaffold opencomp cargo project`

---

### Task 2: Project model — serde structs (RED first)

**Files:**
- Create: `src/project.rs`
- Modify: `src/main.rs` (add `mod project;`)
- Create: `tests/project_test.rs`

**Step 1 — failing tests:**
```rust
// tests/project_test.rs
use opencomp::project::{Project, Layer, Transform, BlendMode};

#[test]
fn parses_minimal_project() {
    let src = r#"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#0a0a0a"
    "#;
    let proj: Project = toml::from_str(src).unwrap();
    assert_eq!(proj.name, "demo");
    assert_eq!(proj.width, 640);
    assert_eq!(proj.height, 480);
    assert_eq!(proj.bg_color.to_hex(), "#0a0a0a");
}

#[test]
fn parses_layer_with_defaults() {
    let src = r#"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#000000"

        [[layer]]
        name = "backdrop"
        type = "solid"
        color = "#ff0000"
    "#;
    let proj: Project = toml::from_str(src).unwrap();
    let l = &proj.layers[0];
    assert_eq!(l.name, "backdrop");
    assert_eq!(l.transform.position, [0.0, 0.0]); // default
    assert_eq!(l.transform.opacity, 100.0);        // default
    assert_eq!(l.transform.scale, [1.0, 1.0]);     // default
    assert_eq!(l.blend, BlendMode::Normal);
}

#[test]
fn parses_full_layer() {
    let src = r#"
        [project]
        name = "demo"
        fps = 24
        width = 640
        height = 480
        duration = 120
        bg_color = "#000000"

        [[layer]]
        name = "card"
        type = "solid"
        color = "#00ff00"
        blend = "normal"

        [layer.transform]
        position = [10.0, 20.0]
        scale = [2.0, 2.0]
        opacity = 50.0
        rotation = 45.0
    "#;
    let proj: Project = toml::from_str(src).unwrap();
    let l = &proj.layers[0];
    assert_eq!(l.transform.position, [10.0, 20.0]);
    assert_eq!(l.transform.scale, [2.0, 2.0]);
    assert_eq!(l.transform.opacity, 50.0);
    assert_eq!(l.transform.rotation, 45.0);
}
```

**Step 2 — run + verify fail:** `cargo test` → compile error (no `opencomp::project`). Expected RED.

**Step 3 — implement `src/project.rs`:**
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Color { pub r: u8, pub g: u8, pub b: u8, pub a: u8 }

impl Color {
    pub fn from_hex(s: &str) -> Self { /* parse #RRGGBB[AA] */ }
    pub fn to_hex(&self) -> String { format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b) }
}

// serde default fns
fn default_pos() -> [f32; 2] { [0.0, 0.0] }
fn default_scale() -> [f32; 2] { [1.0, 1.0] }
fn default_opacity() -> f32 { 100.0 }
fn default_rotation() -> f32 { 0.0 }
fn default_blend() -> BlendMode { BlendMode::Normal }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    #[serde(default = "default_pos")] pub position: [f32; 2],
    #[serde(default = "default_scale")] pub scale: [f32; 2],
    #[serde(default = "default_opacity")] pub opacity: f32,
    #[serde(default = "default_rotation")] pub rotation: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode { Normal }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    #[serde(rename = "type")] pub kind: LayerKind,
    pub color: Option<Color>,
    #[serde(default, flatten)] pub transform: Transform,
    #[serde(default = "default_blend")] pub blend: BlendMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum LayerKind { Solid }

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub name: String, pub fps: u32, pub width: u32, pub height: u32,
    pub duration: u32, pub bg_color: Color, pub layers: Vec<Layer>,
}
```
*(Note: `transform` may need `#[serde(default, flatten)]` + `impl Default` for `Transform` — the test run will tell. Adjust to make the tests pass, keeping the format contract intact.)*

**Step 4 — run + verify pass:** `cargo test` → 3 passed.

**Step 5 — commit:** `feat: project model (serde) with TOML parsing`

---

### Task 3: Color hex parsing (RED first)

**Files:**
- Modify: `src/project.rs`
- Create: `tests/color_test.rs`

**Step 1 — failing tests:**
```rust
#[test]
fn color_from_rgb_hex() {
    let c = Color::from_hex("#ff0000");
    assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
}
#[test]
fn color_from_rgba_hex() {
    let c = Color::from_hex("#ff000080");
    assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 128));
}
#[test]
fn color_from_hex_invalid_panics() {
    // expect a clear panic/error on garbage input
}
```

**Step 2 — verify fail; Step 3 — implement real parsing (strip `#`, parse hex pairs, default alpha 255, reject wrong length/hex chars). Step 4 — verify pass. Step 5 — commit:** `feat: color hex parsing`

---

### Task 4: Background color fill (RED first)

**Files:**
- Create: `src/compositor.rs`
- Modify: `src/main.rs` (`mod compositor;`)
- Create: `tests/compositor_test.rs`

**Step 1 — failing test:**
```rust
#[test]
fn fills_background_color() {
    let proj = /* minimal project, bg #0a0a0a, no layers */;
    let frame = render_frame(&proj, 0);
    assert_eq!(frame.width, 640); assert_eq!(frame.height, 480);
    // every pixel == bg color, fully opaque
    for px in frame.pixels.chunks_exact(4) { assert_eq!(px, &[10, 10, 10, 255]); }
}
```

**Step 2 — verify fail (no `compositor` module). Step 3 — implement:**
```rust
pub struct Frame { pub width: u32, pub height: u32, pub pixels: Vec<u8> }
pub fn render_frame(proj: &Project, _frame: u32) -> Frame {
    // fill bg_color; no layers yet
}
```
**Step 4 — verify pass. Step 5 — commit:** `feat: background fill in compositor`

---

### Task 5: Solid layer rasterization — layer footprint (RED first)

**Step 1 — failing test:** solid red layer, full-canvas footprint (no transform) over black bg → top-left pixel is red, fully opaque.
**Step 2 — verify fail.**
**Step 3 — implement:** start with the full-canvas case: any solid layer with default transform covers the whole frame with its color. (Footprint math comes next.)
**Step 4 — verify pass. Step 5 — commit:** `feat: rasterize solid layer over background`

---

### Task 6: Transform application (RED first)

**Step 1 — failing tests:**
- position `[100, 100]`, size 100×100 solid → pixel at (150,150) is colored, pixel at (0,0) is bg
- scale `[2.0, 2.0]` → layer footprint doubles
- opacity `50.0` → pixel is 50%-blended color over bg (`out = src*a + dst*(1-a)`)
- rotation 45° → corners clipped (assert center still colored; quarter-turn exact equality is overkill here)

**Step 2 — verify fail. Step 3 — implement** a per-pixel loop: compute layer-space coords, inside-footprint test with rotation via inverse transform, then alpha blend. All four sub-behaviors land here — the tests drive the math.
**Step 4 — verify pass. Step 5 — commit:** `feat: transforms (position/scale/opacity/rotation)`

---

### Task 7: Layer stacking order (RED first)

**Step 1 — failing test:** bg black; layer A red, full canvas (bottom); layer B blue, half-canvas (top) → overlap region is blue, non-overlap is red. Proves bottom-up order + alpha blend between layers.
**Step 2 — verify fail; Step 3 — implement:** iterate layers bottom-up, each layer blends over the accumulated frame.
**Step 4 — verify pass. Step 5 — commit:** `feat: bottom-up layer stacking with alpha blending`

---

### Task 8: PNG export (RED first)

**Files:**
- Create: `src/export.rs`
- Modify: `src/main.rs` (`mod export;`)
- Create: `tests/export_test.rs`

**Step 1 — failing tests:**
- `write_png(frame, path)` writes a file that exists and is non-empty
- round-trip: `read_png(path)` (via `png` crate) equals the original `Frame` pixels

**Step 2 — verify fail. Step 3 — implement** with the `png` crate (RGBA8, no interlace). **Step 4 — verify pass. Step 5 — commit:** `feat: PNG export`

---

### Task 9: CLI (`render` subcommand) (RED first)

**Files:**
- Modify: `src/main.rs`
- Modify: `tests/` (CLI integration test — run the binary via `std::process::Command` if needed, or test the `run()` function directly)

**Step 1 — failing tests:**
- `opencomp render examples/demo.toml -o out/frame_0000.png`:
  - exit code 0
  - `out/frame_0000.png` exists and is non-empty
  - missing file arg → non-zero exit with usage; missing project file → non-zero with clear error

**Step 2 — verify fail (no CLI yet). Step 3 — implement** with clap: `render <project>` + `-o/--output` (default `frame_0000.png` in cwd or `out/`).
**Step 4 — verify pass. Step 5 — commit:** `feat: CLI render command`

---

### Task 10: Demo project + README update + final verification

**Files:**
- Create: `examples/demo.toml` (2–3 layered solid rectangles, deliberately visible: distinct colors, offsets, one semi-transparent, maybe rotated)
- Modify: `README.md` (document `cargo run -- render examples/demo.toml -o out/demo.png`)

**Steps:**
1. Write `examples/demo.toml`
2. Run the full pipeline: `cargo run -- release render examples/demo.toml -o out/demo.png`
3. **Verify by eye:** open `out/demo.png` (vision_analyze) and confirm layers/blend/rotation look right
4. **Verify by test:** `cargo test` — all green
5. Commit: `docs: demo project + quickstart`

---

### Task 11: GitHub public repo ✅ DONE — 2026-09-16

- SSH key `~/.ssh/id_ed25519_github` added to GitHub (user `jandre-kleynhans`, auth key `mothership-opencomp`)
- `~/.ssh/config` alias: `github-opencomp` → github.com / key
- Remote: `git@github-opencomp:jandre-kleynhans/opencomp.git`
- Pushed `main`; repo **public**: https://github.com/jandre-kleynhans/opencomp
- Verified: `git ls-remote` + GitHub API `private: false`

**Prereq:** user provides GitHub token or creates repo. See `github-auth` + `github-repo-management` skills.

**Steps:**
1. Auth (HTTPS token or SSH key)
2. `gh repo create opencomp --public --source . --push` (or curl equivalent)
3. Verify `git ls-remote` / repo page live
4. Commit: `chore: push to GitHub`

---

## Verification checklist (end of Phase 1)

- [ ] `cargo test` — all pass
- [ ] `cargo build --release` — clean
- [ ] demo renders to PNG, correct by eye (vision_analyze)
- [ ] repo public on GitHub, `git ls-remote` works
- [ ] README quickstart commands actually run on a fresh clone

## Risks / open questions

- **`flatten` + defaults for `Transform`** — serde quirkiness possible; adjust implementation, keep format contract stable (tests are the contract).
- **Rotation math** — keep it simple (per-pixel inverse transform); optimize later.
- **Cargo.lock** — commit it for a binary (reproducible builds). (.gitignore currently excludes it — flip that.)
- **GitHub auth** — no credentials on this box yet; need user's token or SSH key.