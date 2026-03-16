# Changelog

All notable changes to KetGrid are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Core Data Model (`ketgrid-core`)
- `Circuit`, `QubitWire`, `PlacedGate`, and `GateType` enum with all standard quantum gate variants (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Toffoli, Barrier, Identity, Custom)
- Qubit wire management: add, remove, rename, and reorder operations with automatic gate index updates
- Unitary gate matrices using nalgebra — single-qubit (2x2), multi-qubit (4x4, 8x8), and parameterized rotations
- JSON circuit serialization (`.ket.json` format) with versioned schema, file I/O, and validation
- Serde-based `Serialize`/`Deserialize` for all core types

#### Simulation Engine (`ketgrid-sim`)
- Custom state vector simulator supporting all gate types
- Full complex amplitude tracking with correct qubit ordering
- Gate application for single-qubit, controlled, and multi-qubit operations
- Optional QuantRS2 backend via feature flag
- Memory estimation utilities

#### GUI Application (`ketgrid-gui`)
- egui/eframe application shell with three-panel layout (palette, circuit, state view)
- Circuit renderer with standard quantum notation: horizontal wires, gate boxes, control dots (●), target symbols (⊕), multi-qubit connections, measurement meters
- Grid-based column layout with configurable spacing
- Probability histogram with phase-aware coloring (hue mapped to complex phase) and toggleable amplitude table
- Status bar displaying qubit count, gate count, and memory usage estimates with warnings
- Gate palette with collapsible categories (Basic, Phase, Rotation, Multi-Qubit, Measurement)
- Drag-and-drop gate placement: click-to-place from palette, visual drop indicators, multi-qubit gate connection workflow
- Editor state machine managing placement modes and interaction states

#### Examples
- 5 example circuits as `.ket.json` files: Bell state, GHZ state, Deutsch-Jozsa algorithm, quantum teleportation, Grover's search (2-qubit)

#### Documentation
- 11 technical docs covering architecture, data model, simulation, rendering, and interaction design
