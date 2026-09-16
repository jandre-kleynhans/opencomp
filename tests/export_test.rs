use opencomp::compositor::Frame;

#[test]
fn writes_png_file() {
    let frame = Frame::new(
        2,
        2,
        vec![
            255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255,
        ],
    );
    let path = std::env::temp_dir().join("opencomp_test_write.png");
    opencomp::export::write_png(&frame, &path).unwrap();
    assert!(path.exists());
    assert!(std::fs::metadata(&path).unwrap().len() > 0);
    std::fs::remove_file(&path).ok();
}

#[test]
fn roundtrip_png() {
    let frame = Frame::new(4, 3, (0..4 * 3 * 4).map(|i| (i * 17 % 256) as u8).collect());
    let path = std::env::temp_dir().join("opencomp_test_roundtrip.png");
    opencomp::export::write_png(&frame, &path).unwrap();
    let back = opencomp::export::read_png(&path).unwrap();
    assert_eq!(back.width, 4);
    assert_eq!(back.height, 3);
    assert_eq!(back.pixels, frame.pixels);
    std::fs::remove_file(&path).ok();
}
