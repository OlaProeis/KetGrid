# Roadmap

KetGrid development follows a phased approach. This document tracks what's been completed and what's planned.

**Current status:** v0.1.0 released — Phases 1–4 complete. Working on performance optimization and future directions.

---

## Phase 1: Foundation — Circuit Model & Basic Rendering ✅

*All Phase 1 items are complete.*

- [x] Cargo workspace scaffolding (ketgrid-core, ketgrid-sim, ketgrid-gui)
- [x] Circuit data model (`Circuit`, `PlacedGate`, `GateType` with all standard gates)
- [x] Qubit wire management (add, remove, rename, reorder)
- [x] Gate matrices with nalgebra (single-qubit, multi-qubit, parameterized rotations)
- [x] State vector simulator (custom engine, optional QuantRS2 backend)
- [x] Circuit renderer (wires, gate boxes, control dots, connections, measurements)
- [x] Probability histogram with phase-aware coloring and amplitude table
- [x] JSON circuit format (`.ket.json`) with save/load
- [x] 5 example circuits (Bell, GHZ, Deutsch-Jozsa, teleportation, Grover)
- [x] Status bar (qubit count, gate count, memory estimate)

---

## Phase 2: Interactive Visual Editor ✅

*All Phase 2 items are complete.*

- [x] Gate palette with categorized, collapsible sections
- [x] Drag-and-drop gate placement with visual indicators
- [x] Gate context menu — right-click for edit parameters, delete, copy/paste
- [x] Undo/redo system — operation-based edit history with Ctrl+Z/Y
- [x] Real-time debounced simulation — auto-simulate on edits, background thread for large circuits
- [x] Keyboard shortcuts — Delete, Ctrl+C/V, wire management hotkeys

---

## Phase 3: Visualization & Analysis ✅

*All Phase 3 items are complete.*

- [x] Bloch sphere — 2D-projected Bloch sphere per qubit using reduced density matrix
- [x] Step-through mode — single-gate stepping with playback controls (step, play, reset)
- [x] Entanglement visualization — color-coded qubit wires showing entanglement groups
- [x] Circuit statistics panel — gate counts by type, circuit depth, hardware time estimates

---

## Phase 4: Export, Import & Ecosystem ✅

*All Phase 4 items are complete.*

- [x] OpenQASM 2.0 export — standard quantum assembly format
- [x] Qiskit Python export — generate importable `QuantumCircuit` code
- [x] OpenQASM 2.0 import — parse QASM files into KetGrid circuits via nom parser
- [x] SVG circuit export — vector graphics for documentation and presentations
- [x] Example library browser — searchable UI with 21 categorized circuits

---

## Phase 5: Advanced Features (Post-1.0)

*Planned for future releases.*

- [ ] **Performance optimization** — rayon parallelism tuning, gate fusion improvements, SIMD via nalgebra
- [ ] **Noise simulation** — depolarizing, amplitude/phase damping models; ideal vs noisy comparison
- [ ] **Parameterized circuits** — slider-controlled rotation gates with parameter sweep plots
- [ ] **Custom gate definitions** — define gates as sub-circuits, gate decomposition view
- [ ] **Tutorial mode** — guided interactive lessons, challenges ("Build a circuit that produces |Φ+⟩")
- [ ] **GPU acceleration** — CUDA state vector simulation for large circuits (20+ qubits)
  - Trait-based backend abstraction (CPU/GPU)
  - CUDA kernels for gate operations
  - VRAM management and circuit chunking
  - Hybrid CPU/GPU scheduling with automatic backend selection
- [ ] **WASM target** — web version via egui's WASM support

### Future Consideration: Quantum Kernel Emulator

After v1.0.0, evaluate evolving from state vector simulation to a full **quantum kernel emulator** that explicitly models:

- **1-to-n qubit relationships** — visual representation of how single operations affect the entire quantum register
- **Probabilistic values that collapse on observation** — measurement as a distinct, visible event rather than just reading amplitudes
- **True entanglement simulation** — when one qubit is measured, correlated qubits instantly reflect the collapsed state

This approach could make quantum phenomena more visceral and educational, potentially differentiating KetGrid from all existing circuit tools.

---

## Version Milestones

### v0.1.0 — MVP ✅ (Released)
- Visually build quantum circuits via drag-and-drop
- Real-time simulation results with Bloch sphere and step-through
- Save/load circuits as JSON, export to OpenQASM/Qiskit/SVG
- 21 example circuits with browsable library
- Cross-platform release builds (Windows, macOS, Linux)

### v0.5.0 — Performance & Polish
- Simulation performance optimization (target: <100ms for 15 qubits)
- Noise simulation
- Parameterized circuit UI with sliders

### v1.0.0 — Full Platform
- GPU acceleration for large circuits
- Custom gate definitions
- Tutorial mode
- Stable `ketgrid-core` API, published on crates.io

---

## How to Contribute

Check the roadmap items above and pick something that interests you. The best places to start:

- **Performance optimization** — profiling and improving simulation speed
- **Phase 5 features** — noise simulation, parameterized circuits
- **Example circuits** — adding more quantum algorithm examples is always welcome

See [CONTRIBUTING section in README](README.md#contributing) for setup instructions.
