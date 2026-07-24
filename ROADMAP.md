# Roadmap

Work that is known to be missing, roughly in the order it is worth doing.

This is a holding area, not a commitment. When one of these is picked up it
should become an OpenSpec change proposal (`openspec new change <name>`) with its
own specs and tasks, and this entry should link to it.

## Verification

- [x] Integration tests driving a headless `App` — see `tests/rebuild.rs`. Covers
      rebuild-on-change, no-rebuild-when-unchanged (by entity identity, not just
      count), the material cache not leaking across rebuilds, plugin composition,
      stale geometry being cleared, non-finite values being skipped, and every
      generated entity carrying `Visibility`.

Still uncovered:

- [ ] Anything requiring a GPU: that the meshes are *correct*, not merely
      present. The showcase render is checked by eye when it changes.
- [ ] Large-dataset behaviour under load.

## Features

- [ ] **Text labels** — axis ticks, category names, and legends in 3D. The largest
      gap: `ChartData::labels` and `Dataset::label` are carried through the data
      model but never drawn, so charts cannot currently identify themselves and
      depend on external UI. Billboarded text in a 3D scene needs a design pass
      before any code.
- [ ] **Stacked bars and area charts.**
- [ ] **Picking and tooltips.**

## Scales, via an optional `charton` feature

`charton`'s public `ScaleTrait` is a better abstraction than the hand-rolled
[`Scale`](src/axis.rs), and would bring log, time, and band scales. It is not a
core dependency — the reasoning is in the bootstrap change's `design.md`.

- [ ] Optional `charton` feature adapting `ScaleTrait` to `Scale`, for log, time,
      and band axes.
- [ ] Extension chart types built on it: boxplot, density/violin.

## Performance

- [ ] **Incremental rebuilds.** A chart currently despawns and respawns every
      child on any change, which is fine at the sampling rates the examples use
      but wrong for a large, frequently-updated dataset.
- [ ] **Skip zero-extent bars.** Bar and histogram charts spawn an entity per
      value including zeros, which `box_transform` collapses to a 1e-5 sliver —
      invisible, but still an entity and a draw call. A histogram with mostly
      empty bins pays for all of them. Noticed while writing `tests/rebuild.rs`,
      where four observations produce ten bars.

## Repository

- [ ] **Branch protection on `main`**, requiring the CI check to pass before a
      merge. Nothing enforces this today, which is how PR #1 was able to merge
      empty and leave the repository with no code in it.
