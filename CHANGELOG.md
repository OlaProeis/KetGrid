# Changelog

All notable changes to KetGrid are documented here.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] — 2026-03-17

First public release of KetGrid — a native desktop quantum circuit editor and simulator.

### Added

#### Core Data Model (`ketgrid-core`)
- `Circuit`, `QubitWire`, `PlacedGate`, and `GateType` enum with all standard quantum gate variants (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Toffoli, Barrier, Identity, Custom)
- Qubit wire management: add, remove, rename, and reorder operations with automatic gate index updates
- Unitary gate matrices using nalgebra — single-qubit (2x2), multi-qubit (4x4, 8x8), and parameterized rotations
- JSON circuit serialization (`.ket.json` format) with versioned schema, file I/O, and validation
- OpenQASM 2.0 export with full gate mapping (H, X, Y, Z, S, T, Rx, Ry, Rz, CNOT, CZ, SWAP, Toffoli, barriers, measurements)
- OpenQASM 2.0 import via nom parser with ASAP column scheduling and parameter expression support
- Qiskit Python code export generating importable `QuantumCircuit` code
- SVG circuit export for publication-quality vector diagrams
- Serde-based `Serialize`/`Deserialize` for all core types

#### Simulation Engine (`ketgrid-sim`)
- Custom state vector simulator supporting all gate types
- Full complex amplitude tracking with correct qubit ordering
- Gate application for single-qubit, controlled, and multi-qubit operations
- Pre-allocated reusable buffers eliminating per-gate heap allocations
- Rayon-parallelized gate application for ≥12 qubit circuits
- Column checkpoint system for incremental re-simulation (≤15 qubits)
- Gate fusion: consecutive single-qubit gates on the same qubit composed into a single matrix
- Three simulation paths: standard (with checkpoints), incremental (from dirty column), optimized (with gate fusion)
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
- Right-click context menu for gates with edit, copy, paste, and delete operations
- Operation-based undo/redo system (Ctrl+Z / Ctrl+Y) with 100-operation stack
- Background simulation with 100ms debounce, std::thread + mpsc channels, manual trigger for >15 qubits
- Per-qubit Bloch sphere visualization using partial trace and 2D-projected rendering
- Step-through mode with single-column stepping, playback controls, and vertical cursor
- Color-coded entanglement visualization using purity-based detection and union-find grouping
- Circuit statistics panel: gate counts by type, depth, qubit usage, memory and hardware time estimates
- Example library browser with categorized, searchable UI for 21 quantum circuit examples
- Keyboard shortcuts for undo/redo, copy/paste, delete, and wire management
- File dialogs for open/save via native OS dialogs (rfd)

#### Examples
- 21 example circuits as `.ket.json` files across three categories:
  - **Fundamentals**: Bell state, Hadamard, Pauli-X/Y/Z, Phase gate, T gate, Rotation gates, SWAP, Toffoli
  - **Algorithms**: Deutsch-Jozsa, Bernstein-Vazirani, Simon's algorithm, Grover's search (2-qubit), QFT (3-qubit), Superdense coding, Quantum teleportation
  - **Error Correction**: Bit-flip code, Phase-flip code, Shor code

#### Infrastructure
- GitHub Actions CI/CD workflow for cross-platform release builds (Windows, macOS x86_64/aarch64, Linux)

#### Documentation
- 24 technical docs covering architecture, data model, simulation, rendering, interaction design, and all features
