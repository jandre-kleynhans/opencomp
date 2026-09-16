# ADR-0002: Full-canvas layer centering convention

Status: Accepted
Date: 2026-09-16

## Context

Phase 1 introduced `position` as the layer footprint center. When a layer has
`size = [0, 0]` (full canvas), position offset pushed the already-centered
footprint off-canvas, producing incorrect composites.

## Decision

**For full-canvas layers (`size = [0, 0]`), `position` is ignored and the layer
is centered at the canvas center.** For sized layers, `position` is the center
of the layer footprint, measured from the canvas top-left (same as After Effects
anchor point semantics).

Keyframe expressions that animate `position` on a full-canvas layer are a no-op
by design. Authors who want to move a layer must give it a `size`.

## Consequences

- Full-canvas layers always cover the entire frame — the simplest possible case
  has zero parameterization bugs.
- Sized layers are fully parameterized and composit with position, rotation,
  scale, and opacity, matching After Effects semantics.
- Expressions on position of a full-canvas layer evaluate but their result is
  silently ignored (clear contract; no runtime warning, by design in v0.2).
