use crate::project::Project;

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
/// Phase 1: fills the background color. Layers are composited on top
/// (Task 5+). Future: keyframe resolution at `frame` time.
pub fn render_frame(proj: &Project, _frame: u32) -> Frame {
    let width = proj.project.width;
    let height = proj.project.height;
    let bg = proj.project.bg_color;
    let n = (width * height) as usize;
    let mut pixels = Vec::with_capacity(n * 4);
    for _ in 0..n {
        // Background is a solid fill — force opaque (a solid color "background"
        // with alpha < 255 would let an artificial transparency show through).
        pixels.extend_from_slice(&[bg.r, bg.g, bg.b, 255]);
    }
    Frame::new(width, height, pixels)
}
