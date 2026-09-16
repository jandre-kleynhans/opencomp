// Phase 4 — WASM effect runtime.
//
// ABI: plugin wasm exports
//   alloc(len: i32) -> i32
//   process(ptr: i32, w: i32, h: i32) -> i32   (process RGBA8 in place)
//   free(ptr: i32, len: i32)
// See docs/ADRs/0004-wasm-effects.md.
use std::sync::Arc;

use wasmtime::{Engine, Instance, Linker, Memory, Module, Store};

pub struct Effect {
    pub name: String,
    engine: Engine,
    module: Module,
}

#[derive(Debug)]
pub enum EffectError {
    Compile(String),
    Link(String),
    Instantiate(String),
    Call(String),
}

impl std::fmt::Display for EffectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EffectError::Compile(e) => write!(f, "effect compile error: {e}"),
            EffectError::Link(e) => write!(f, "effect link error: {e}"),
            EffectError::Instantiate(e) => write!(f, "effect instantiate error: {e}"),
            EffectError::Call(e) => write!(f, "effect call error: {e}"),
        }
    }
}

impl std::error::Error for EffectError {}

/// Load a WASM effect from raw bytes (wat or wasm).
pub fn load_effect(bytes: Vec<u8>) -> Result<Effect, EffectError> {
    let engine = Engine::default();
    let module = Module::new(&engine, &bytes).map_err(|e| EffectError::Compile(e.to_string()))?;
    Ok(Effect {
        name: String::new(),
        engine,
        module,
    })
}

/// Run an effect over an RGBA8 frame (in place). Returns the processed frame.
pub fn apply_effect(eff: &Effect, frame: &[u8], w: u32, h: u32) -> Result<Vec<u8>, EffectError> {
    let mut store = Store::new(&eff.engine, ());
    let linker = Linker::new(&eff.engine);
    let instance = linker
        .instantiate(&mut store, &eff.module)
        .map_err(|e| EffectError::Instantiate(e.to_string()))?;

    let memory: Memory = instance
        .get_memory(&mut store, "memory")
        .ok_or_else(|| EffectError::Link("no exported memory".into()))?;

    let alloc = instance
        .get_typed_func::<i32, i32>(&mut store, "alloc")
        .map_err(|e| EffectError::Link(e.to_string()))?;
    let process = instance
        .get_typed_func::<(i32, i32, i32), i32>(&mut store, "process")
        .map_err(|e| EffectError::Link(e.to_string()))?;
    let free = instance
        .get_typed_func::<(i32, i32), ()>(&mut store, "free")
        .map_err(|e| EffectError::Link(e.to_string()))?;

    let n = (w * h * 4) as usize;
    let ptr = alloc
        .call(&mut store, n as i32)
        .map_err(|e| EffectError::Call(e.to_string()))?;

    // copy frame bytes into linear memory
    unsafe {
        let mem = memory.data_ptr(&store);
        std::ptr::copy_nonoverlapping(frame.as_ptr(), mem.add(ptr as usize), n);
    }

    process
        .call(&mut store, (ptr, w as i32, h as i32))
        .map_err(|e| EffectError::Call(e.to_string()))?;

    // read back
    let mut out = vec![0u8; n];
    unsafe {
        let mem = memory.data_ptr(&store);
        std::ptr::copy_nonoverlapping(mem.add(ptr as usize), out.as_mut_ptr(), n);
    }

    free.call(&mut store, (ptr, n as i32))
        .map_err(|e| EffectError::Call(e.to_string()))?;

    Ok(out)
}

/// A loaded effect instance, ready to apply to frames (thread-safe wrapper).
pub struct EffectHandle {
    pub effect: Arc<Effect>,
}

impl EffectHandle {
    pub fn apply(&self, frame: &[u8], w: u32, h: u32) -> Result<Vec<u8>, EffectError> {
        apply_effect(&self.effect, frame, w, h)
    }
}
