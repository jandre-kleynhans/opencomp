# ADR-0004: WASM effect runtime — linear-memory ABI

Status: Accepted
Date: 2026-09-16

## Context

Phase 4 requires a plugin runtime so effects can be authored in any language
that compiles to WASM (Rust, C, Go, AssemblyScript), sandboxed from the core.
The plan draft showed an `Effect` trait with `FrameBuffer` + `Param`; the
simplest ABI that wasmtime supports robustly is a linear-memory contract.

## Decision

**Effects are WASM modules exporting three functions:**

```
alloc(len: i32) -> i32
process(ptr: i32, w: i32, h: i32) -> i32   # RGBA8 frame, processed in place
free(ptr: i32, len: i32)
```

- `memory` export holds the frame bytes.
- The engine copies the composited frame into linear memory, calls `process`,
  reads back the (possibly modified) bytes, calls `free`.
- The core treats effects as opaque: `name`, `plugin` (path or inline wasm),
  optional `params` — params are passed as a side-channel (future).

The `process` in-place model keeps the ABI tiny and deterministic. Full
`Param` plumbing and a plugin registry are Phase 4 follow-ups behind the same
ABI (tests are the contract).

## Consequences

- Any language with WASM32 support can author effects; no SDK signing.
- The core's memory is never exposed to untrusted code — only linear memory
  copy-in/copy-out.
- One extra copy per effect (core → linear → core) — acceptable for v0.4;
  future GPU path can avoid it.
- The WAT test module doubles as documentation of the contract.