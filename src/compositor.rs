use crate::project::{Layer, LayerKind, Project};

/// A rendered frame: RGBA8, row-major, top-left origin, width*height*4 bytes.
#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub pixels: Vec<u8>,
}

impl Frame {
    pub fn new(width: u32, height: u32, pixels: Vec<u8>) -> Self {
        Frame {
            width,
            height,
            pixels,
        }
    }
}

/// Render a single frame of a project.
///
/// Phase 1: fills the background, then composites layers bottom-up.
/// Solid layers currently cover the full canvas (transform/position
/// footprint comes in Task 6).
pub fn render_frame(proj: &Project, _frame: u32) -> Frame {
    let width = proj.project.width;
    let height = proj.project.height;
    let n = (width * height) as usize;

    // Start with opaque background fill.
    let bg = proj.project.bg_color;
    let mut pixels = vec![0u8; n * 4];
    for px in pixels.chunks_exact_mut(4) {
        px.copy_from_slice(&[bg.r, bg.g, bg.b, 255]);
    }

    // Composite layers bottom-up.
    for layer in &proj.layers {
        composite_layer(&mut pixels, layer);
    }

    Frame::new(width, height, pixels)
}

/// Composite one layer over the frame.
///
/// Phase 1: solid layers only, full-canvas footprint.
/// Blend: normal alpha over — `out = src*a + dst*(1-a)` per channel.
fn composite_layer(pixels: &mut [u8], layer: &Layer) {
    let color = match (layer.kind, layer.color) {
        (LayerKind::Solid, Some(c)) => c,
        (LayerKind::Solid, None) => return, // no color → invisible
    };
    let a = color.a as f32 / 255.0;
    let inv = 1.0 - a;
    for px in pixels.chunks_exact_mut(4) {
        let (r, g, b) = (px[0] as f32, px[1] as f32, px[2] as f32);
        px[0] = (color.r as f32 * a + r * inv).round() as u8;
        px[1] = (color.g as f32 * a + g * inv).round() as u8;
        px[2] = (color.b as f32 * a + b * inv).round() as u8;
        px[3] = 255; // frame stays opaque
    }
}