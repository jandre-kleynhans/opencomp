use crate::project::{Layer, LayerKind, Project};
use std::f32::consts::PI;

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
/// Phase 1: fills the background, then composites layers bottom-up with
/// position, scale, opacity, and rotation transforms applied.
pub fn render_frame(proj: &Project, _frame: u32) -> Frame {
    let width = proj.project.width;
    let height = proj.project.height;
    let n = (width * height) as usize;

    let bg = proj.project.bg_color;
    let mut pixels = vec![0u8; n * 4];
    for px in pixels.chunks_exact_mut(4) {
        px.copy_from_slice(&[bg.r, bg.g, bg.b, 255]);
    }

    for layer in &proj.layers {
        composite_layer(&mut pixels, layer, width, height);
    }

    Frame::new(width, height, pixels)
}

/// Composite one layer over the frame, applying transforms.
///
/// Uses an inverse-transform approach: for each canvas pixel, un-rotate
/// and check whether it falls inside the layer's footprint. This is
/// correct for any combination of position + scale + rotation.
fn composite_layer(pixels: &mut [u8], layer: &Layer, canvas_w: u32, canvas_h: u32) {
    let color = match (layer.kind, layer.color) {
        (LayerKind::Solid, Some(c)) => c,
        _ => return,
    };

    // Effective opacity: layer opacity (0–100) × color alpha (0–255).
    let effective_alpha = (layer.transform.opacity / 100.0) * (color.a as f32 / 255.0);
    if effective_alpha <= 0.0 {
        return;
    }

    // Layer footprint size. [0, 0] means full canvas.
    let (lw, lh) = if layer.size == [0.0, 0.0] {
        (canvas_w as f32, canvas_h as f32)
    } else {
        (layer.size[0], layer.size[1])
    };
    // After scale.
    let sw = lw * layer.transform.scale[0];
    let sh = lh * layer.transform.scale[1];
    let half_w = sw / 2.0;
    let half_h = sh / 2.0;

    // Rotation in radians (degrees → radians, clockwise).
    let theta = layer.transform.rotation * PI / 180.0;
    let cos_t = theta.cos();
    let sin_t = theta.sin();

    // Layer center in canvas space. For full-canvas layers, the footprint
    // is the whole canvas: center at the canvas center (position is
    // relative offset from there).
    let (cx, cy) = if lw == canvas_w as f32 && lh == canvas_h as f32 {
        (
            canvas_w as f32 * 0.5 + layer.transform.position[0],
            canvas_h as f32 * 0.5 + layer.transform.position[1],
        )
    } else {
        (layer.transform.position[0], layer.transform.position[1])
    };

    let a = effective_alpha;
    let inv = 1.0 - a;

    for y in 0..canvas_h {
        for x in 0..canvas_w {
            // Translate to center-relative.
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;

            // Un-rotate by -θ (inverse rotation).
            let lx = dx * cos_t + dy * sin_t;
            let ly = -dx * sin_t + dy * cos_t;

            // Is this pixel inside the layer footprint?
            if lx.abs() <= half_w && ly.abs() <= half_h {
                let idx = ((y * canvas_w + x) * 4) as usize;
                let r = pixels[idx] as f32;
                let g = pixels[idx + 1] as f32;
                let b = pixels[idx + 2] as f32;
                pixels[idx] = (color.r as f32 * a + r * inv).round() as u8;
                pixels[idx + 1] = (color.g as f32 * a + g * inv).round() as u8;
                pixels[idx + 2] = (color.b as f32 * a + b * inv).round() as u8;
                pixels[idx + 3] = 255;
            }
        }
    }
}
