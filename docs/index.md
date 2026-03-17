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
- `state-vector-simulator.md` - State vector simulation engine: gate application, parallelism, checkpoints, gate fusion, qubit ordering, API.
- `circuit-renderer.md` - Circuit visualization: gate rendering, layout constants, visual symbols.
- `state-visualization.md` - Probability histogram with phase-aware coloring and amplitude table.
- `status-bar.md` - Real-time circuit metrics: qubit count, gate count, memory estimates with warnings.
- `gate-palette.md` - Gate selection panel with categorized, collapsible gate buttons.
- `example-circuits.md` - Five example quantum circuits (Bell, GHZ, Deutsch-Jozsa, teleportation, Grover) with descriptions and expected simulation results.
- `drag-and-drop-placement.md` - Drag-and-drop gate placement: click-to-place, multi-qubit flows, editor state machine.
- `gate-context-menu.md` - Right-click context menu for gates and measurements with edit, copy, paste, and delete operations.
- `undo-redo.md` - Operation-based undo/redo system: EditHistory, EditOperation, Ctrl+Z/Y bindings.
- `debounced-simulation.md` - Background simulation with 100ms debounce, std::thread + mpsc, incremental re-sim via dirty_column, manual trigger for >15 qubits.
- `bloch-sphere.md` - Per-qubit Bloch sphere visualization: partial trace, Bloch vector, interactive 2D-projected rendering.
- `step-through-mode.md` - Single-gate-column stepping with playback controls, vertical cursor, and per-step state visualization.
- `entanglement-visualization.md` - Color-coded entanglement clusters: purity-based detection, union-find grouping, wire coloring.
- `circuit-statistics.md` - Detailed circuit metrics panel: gate counts by type, depth, qubit usage, memory and hardware time estimates.
- `qiskit-export.md` - Export circuits to Qiskit Python code for IBM hardware and Qiskit ecosystem integration.
- `openqasm-export.md` - Export circuits to OpenQASM 2.0 format for IBM quantum hardware and broader ecosystem compatibility.
- `openqasm-import.md` - Parse OpenQASM 2.0 files into circuits: nom parser, gate mapping, parameter expressions, ASAP column scheduling.
- `svg-export.md` - Vector export of circuit diagrams for publication-quality figures.
- `example-library-browser.md` - Categorized, searchable UI for browsing and loading 21+ quantum circuit examples.
- `keyboard-shortcuts.md` - Complete reference for keyboard shortcuts: undo/redo, copy/paste, delete, wire management.
