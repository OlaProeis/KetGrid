# Documentation Index

> **Rules:** This file is a pure documentation map. It must ONLY contain an index of docs with one-line descriptions.
> No project history, no task lists, no architecture overviews.
> Update this file whenever a new doc is added to `docs/`.

## Core Context
- `ai-context.md` - Core project architecture, rules, and conventions.

## Technical Docs
- `project-scaffolding.md` - Workspace structure and crate organization.
- `circuit-data-model.md` - Core data structures for quantum circuit representation.
- `wire-management.md` - Qubit wire lifecycle: add, remove, rename, reorder.
- `gate-matrices.md` - Unitary matrix representations for quantum gates.
- `state-vector-simulator.md` - State vector simulation engine: gate application, qubit ordering, API.
- `circuit-renderer.md` - Circuit visualization: gate rendering, layout constants, visual symbols.
- `state-visualization.md` - Probability histogram with phase-aware coloring and amplitude table.
- `status-bar.md` - Real-time circuit metrics: qubit count, gate count, memory estimates with warnings.
- `gate-palette.md` - Gate selection panel with categorized, collapsible gate buttons.
- `example-circuits.md` - Five example quantum circuits (Bell, GHZ, Deutsch-Jozsa, teleportation, Grover) with descriptions and expected simulation results.
- `drag-and-drop-placement.md` - Drag-and-drop gate placement: click-to-place, multi-qubit flows, editor state machine.
