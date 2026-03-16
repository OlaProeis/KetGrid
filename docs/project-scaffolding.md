# Project Scaffolding

Workspace structure and crate organization for the Ket quantum circuit editor.

## Overview

The project uses a Cargo workspace with three crates separating concerns between data model, simulation, and GUI.

## Workspace Layout

```
Cargo.toml          # Workspace root
├── crates/
│   ├── ket-core/   # Circuit data model, gates, serialization
│   ├── ket-sim/    # State vector simulation, QuantRS2 integration
│   └── ket-gui/    # egui application, circuit editor UI
```

## Crate Dependencies

- **ket-core**: `serde`, `serde_json` — Self-contained data types
- **ket-sim**: `ket-core`, `nalgebra`, `quantrs2` (optional) — Depends on core for circuit definition
- **ket-gui**: `ket-core`, `ket-sim`, `eframe`, `egui`, `egui_extras`, `dirs` — Full stack for the application

## Key Types

- `Circuit` — Container for qubits and gates
- `PlacedGate` — Gate positioned at a specific column/target qubits
- `GateType` — Enum of H, X, Y, Z, S, T, CNOT, Measure
- `StateVector` — Complex vector for simulation (length = 2^n)

## GUI Layout

Three-panel egui interface:
- **Left**: Gate palette with draggable gate buttons
- **Center**: Circuit editor with qubit wire rendering
- **Right**: State view showing measurement probabilities

## Build Configuration

Release profile uses LTO and strip for minimal binary size (~4.8 MB target).
