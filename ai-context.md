# Ket — AI Context

## Rules (DO NOT UPDATE)
- Never auto-update this file or current-handover-prompt.md — only update when explicitly requested.
- Only do the task specified, do not start the next task, or go over scope.
- Run `cargo check --workspace` after changes to verify code compiles.
- Follow existing code patterns and conventions.
- Document by feature (e.g., `circuit-renderer.md`), not by task.
- Update `docs/index.md` when adding new documentation.
- Use Context7 MCP tool to fetch library documentation when needed (resolve library ID first, then fetch docs).

## Tech Stack
- **Language:** Rust
- **GUI:** egui + eframe (immediate-mode)
- **Quantum Sim:** Custom state vector (primary), QuantRS2 (optional feature flag)
- **Math:** nalgebra (complex matrix ops)
- **Parallelism:** rayon (data-parallel gate application ≥12 qubits)
- **Serialization:** serde + serde_json
- **Parsing:** nom (OpenQASM import)
- **Persistence:** dirs + serde (cross-platform config/session)

## Architecture & Data Model
Cargo workspace with three crates:
- `ket-core` — Circuit data model, gate definitions, serialization formats
- `ket-sim` — Simulation engine (state vector, stabilizer, noise)
- `ket-gui` — egui application (renderer, editor, palette, visualizations)

Core types: `Circuit`, `QubitWire`, `PlacedGate`, `GateType` (enum), `Measurement`.
JSON-based `.ket.json` file format with versioning.

## Conventions
- **Modularity:** One feature per file, one crate per concern.
- **Errors:** Strict error handling, no silent failures. Use `Result<T, Error>`.
- **Naming:** Snake_case for Rust, standard quantum notation for display (|q₀⟩, ⊕, ●).
- **Performance:** Real-time sim feedback <100ms target for ≤15 qubits (not yet achieved for 14+). Background thread for >15. Manual trigger for >20.

## Where Things Live
| Want to...                  | Look in...                          |
|-----------------------------|-------------------------------------|
| Define gate types/circuit   | `crates/ketgrid-core/src/`          |
| Run simulation              | `crates/ketgrid-sim/src/`           |
| Render circuit visually     | `crates/ketgrid-gui/src/circuit_view.rs`|
| Handle drag-and-drop        | `crates/ketgrid-gui/src/editor.rs`  |
| Manage gate palette UI      | `crates/ketgrid-gui/src/gate_palette.rs`|
| Visualize state/probs       | `crates/ketgrid-gui/src/state_view.rs`|
| Load/save JSON circuits     | `crates/ketgrid-core/src/format/`   |
| Import/export external formats | `crates/ketgrid-core/src/format/` |
| Example circuits            | `examples/`                         |
| Browse/load examples        | `crates/ketgrid-gui/src/examples.rs`|
| Project docs                | `docs/`                             |
