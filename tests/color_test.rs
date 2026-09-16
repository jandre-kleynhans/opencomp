use opencomp::project::Color;

#[test]
fn color_rgb() {
    let c = Color::from_hex("#ff0000");
    assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
}

#[test]
fn color_rgba() {
    let c = Color::from_hex("#ff000080");
    assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 128));
}

#[test]
fn color_lowercase_hex() {
    let c = Color::from_hex("#0a0a0a");
    assert_eq!((c.r, c.g, c.b, c.a), (10, 10, 10, 255));
}

#[test]
fn color_no_hash_prefix() {
    let c = Color::from_hex("00ff00");
    assert_eq!((c.r, c.g, c.b, c.a), (0, 255, 0, 255));
}

#[test]
#[should_panic]
fn color_invalid_length_panics() {
    let _ = Color::from_hex("#ff00"); // 4 chars — invalid
}

#[test]
#[should_panic]
fn color_invalid_hex_digit_panics() {
    let _ = Color::from_hex("#zzz000"); // non-hex char
}

#[test]
fn color_to_hex_roundtrip() {
    let c = Color::from_hex("#1A2B3C");
    assert_eq!(c.to_hex(), "#1a2b3c");
    // Round-trip: from_hex(to_hex(c)) == c (alpha defaulted to 255)
    let back = Color::from_hex(&c.to_hex());
    assert_eq!(back.r, c.r);
    assert_eq!(back.g, c.g);
    assert_eq!(back.b, c.b);
    assert_eq!(back.a, 255);
}