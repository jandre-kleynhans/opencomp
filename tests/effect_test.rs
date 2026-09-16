// Phase 4 — WASM effect runtime tests (RED first).
//
// ABI contract (see docs/ADRs/0004-wasm-effects.md):
//   Plugin wasm exports:
//     alloc(len: i32) -> i32         allocate len bytes, return offset (linear mem)
//     process(ptr: i32, w: i32, h: i32) -> i32   process RGBA8 frame in place, return ptr
//     free(ptr: i32, len: i32)
//   The engine copies frame bytes into linear memory, calls process, reads back.
use opencomp::effect::{apply_effect, load_effect, EffectError};

const INVERT_WAT: &str = r#"
(module
  (memory (export "memory") 1)
  (func (export "alloc") (param i32) (result i32)
    (i32.const 8))
  (func (export "process") (param i32 i32 i32) (result i32)
    ;; invert RGBA: out[i] = 255 - in[i] for RGB, keep alpha
    ;; ptr (param 0), w (1), h (2) — iterate all pixels
    (local $i i32)
    (local $end i32)
    local.get 0
    local.set $i
    ;; end = ptr + w*h*4
    local.get 1
    local.get 2
    i32.mul
    i32.const 4
    i32.mul
    local.get 0
    i32.add
    local.set $end
    (block $done
      (loop $l
        (br_if $done (i32.ge_u (local.get $i) (local.get $end)))
        ;; r = 255 - mem[i]
        (i32.store8
          (local.get $i)
          (i32.sub (i32.const 255) (i32.load8_u (local.get $i))))
        ;; g = 255 - mem[i+1]
        (i32.store8
          (i32.add (local.get $i) (i32.const 1))
          (i32.sub (i32.const 255) (i32.load8_u (i32.add (local.get $i) (i32.const 1)))))
        ;; b = 255 - mem[i+2]
        (i32.store8
          (i32.add (local.get $i) (i32.const 2))
          (i32.sub (i32.const 255) (i32.load8_u (i32.add (local.get $i) (i32.const 2)))))
        ;; a unchanged
        (i32.store8
          (i32.add (local.get $i) (i32.const 3))
          (i32.load8_u (i32.add (local.get $i) (i32.const 3))))
        (local.set $i (i32.add (local.get $i) (i32.const 4)))
        (br $l)
      )
    )
    local.get 0
  )
  (func (export "free") (param i32 i32))
)
"#;

#[test]
fn loads_invert_plugin_from_wat() {
    let eff = load_effect(INVERT_WAT.to_string().into_bytes()).unwrap();
    assert!(eff.name.is_empty() || eff.name == ""); // default
}

#[test]
fn apply_effect_inverts_rgba() {
    let eff = load_effect(INVERT_WAT.to_string().into_bytes()).unwrap();
    // 2x2 RGBA frame: red, green, blue, white
    let frame = vec![
        255, 0, 0, 255, // red
        0, 255, 0, 255, // green
        0, 0, 255, 255, // blue
        255, 255, 255, 255, // white
    ];
    let out = apply_effect(&eff, &frame, 2, 2).unwrap();
    // inverted: cyan, magenta, yellow, black ; alpha preserved
    assert_eq!(&out[0..4], &[0, 255, 255, 255]); // cyan
    assert_eq!(&out[4..8], &[255, 0, 255, 255]); // magenta
    assert_eq!(&out[8..12], &[255, 255, 0, 255]); // yellow
    assert_eq!(&out[12..16], &[0, 0, 0, 255]); // black
}

#[test]
fn effect_error_on_bad_wasm() {
    let err = load_effect(b"not wasm".to_vec());
    assert!(err.is_err());
    match err {
        Err(EffectError::Compile(_)) => {}
        _ => panic!("expected compile error"),
    }
}
