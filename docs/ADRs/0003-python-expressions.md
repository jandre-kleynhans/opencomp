# ADR-0003: Expression evaluator — pure-Rust arithmetic (no Python runtime dependency)

Status: Accepted
Date: 2026-09-16

## Context

The original Phase 2 plan specified `rustpython-vm` as the expression evaluator
dependency. After implementing the TDD tests, the actual expression patterns are
all arithmetic (`value * 2`, `[value[0] + 10, value[1] - 10]`, `time * fps`),
so a full Python runtime is unnecessary for v0.2.

## Decision

**Use a pure-Rust arithmetic expression evaluator** that:
- Substitutes variables (`value`, `time`, `frame`, `fps`, `width`, `height`,
  `duration`) before tokenization
- Supports `+`, `-`, `*`, `/`, parentheses, unary minus, and integer/float
  literals
- Returns `f32` (coerced from `f64` internal precision)
- Panics with a clear `"expression syntax error"` or
  `"expression must be a number expression"` message on invalid input (v0.2
  convention; typed `Result` error type deferred to Phase 3)

**If Python expressions are needed later** (complex logic, imports, loops), the
arithmetic evaluator remains as the fast path and `rustpython-vm` (or equivalent)
is added as an optional feature gated behind the same `evaluate_expression` API.

The expression contract is locked by the `expression_test.rs` test suite (9
tests). Any future Python runtime must pass the same tests.

## Consequences

- **No C toolchain or Python runtime required** to build or run OpenComp — the
  exe ships as a single static binary (Linux, Windows, macOS).
- Build time stays under 30 seconds (vs. ~2 min with `rustpython-vm`).
- Users who need Python-level expressions can open a feature request and opt in
  to the heavier runtime.
- The test contract means the swap is non-breaking: same function signatures,
  same panic messages, same return types.
