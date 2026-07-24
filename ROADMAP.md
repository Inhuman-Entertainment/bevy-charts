# Roadmap

Work that is known to be missing, roughly in the order it is worth doing.

This is a holding area, not a commitment. When one of these is picked up it
should become an OpenSpec change proposal (`openspec new change <name>`) with its
own specs and tasks, and this entry should link to it.

## Verification

The library's pure functions are well covered, but nothing tests it as a Bevy
app. Every spec scenario describing ECS behaviour — that a chart rebuilds when
its data changes, that an unchanged chart is left alone, that the material cache
stops rebuilds from leaking assets, that plugins compose — was checked once by
hand and cannot be rechecked by CI.

- [ ] Integration tests driving a headless `App` (`MinimalPlugins` + `AssetPlugin`),
      asserting on generated children across `app.update()` calls.

This matters more than it looks. The one real bug found so far — chart children
missing `Visibility`, which in Bevy 0.19 `Mesh3d` does not require — compiled
cleanly, passed every unit test, and rendered nothing. Only running the app
caught it. An app-level test would have caught it automatically.

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

## Repository

- [ ] **Branch protection on `main`**, requiring the CI check to pass before a
      merge. Nothing enforces this today, which is how PR #1 was able to merge
      empty and leave the repository with no code in it.
