use crate::compositor::Frame;
use std::io::BufWriter;
use std::path::Path;

/// Write a frame to a PNG file (RGBA8, no interlace).
pub fn write_png(frame: &Frame, path: &Path) -> Result<(), String> {
    let file = std::fs::File::create(path).map_err(|e| format!("create: {e}"))?;
    let mut encoder = png::Encoder::new(BufWriter::new(file), frame.width, frame.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header().map_err(|e| format!("header: {e}"))?;
    writer
        .write_image_data(&frame.pixels)
        .map_err(|e| format!("write: {e}"))?;
    Ok(())
}

/// Read a PNG file back into a Frame.
pub fn read_png(path: &Path) -> Result<Frame, String> {
    let file = std::fs::File::open(path).map_err(|e| format!("open: {e}"))?;
    let mut decoder = png::Decoder::new(file);
    decoder.set_transformations(png::Transformations::ALPHA | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().map_err(|e| format!("info: {e}"))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|e| format!("frame: {e}"))?;
    buf.truncate(info.buffer_size());
    Ok(Frame::new(info.width, info.height, buf))
}
