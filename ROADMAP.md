# Roadmap

KetGrid development follows a phased approach. This document tracks what's been completed and what's planned.

**Current status:** Phase 1 complete, Phase 2 in progress (75% done).

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

## Phase 2: Interactive Visual Editor 🔧

*Core editing is working. Remaining items focus on polish and power-user features.*

- [x] Gate palette with categorized, collapsible sections
- [x] Drag-and-drop gate placement with visual indicators
- [ ] **Gate context menu** — right-click for edit parameters, delete, copy/paste
- [ ] **Undo/redo system** — operation-based edit history with Ctrl+Z/Y
- [ ] **Real-time debounced simulation** — auto-simulate on edits, background thread for large circuits
- [ ] **Keyboard shortcuts** — Delete, Ctrl+C/V, wire management hotkeys

---

## Phase 3: Visualization & Analysis

*Rich visualization to make quantum states tangible.*

- [ ] **Bloch sphere** — 2D-projected Bloch sphere per qubit using reduced density matrix
- [ ] **Step-through mode** — single-gate stepping with playback controls (step, play, reset)
- [ ] **Entanglement visualization** — color-coded qubit wires showing entanglement groups
- [ ] **Circuit statistics panel** — gate counts by type, circuit depth, hardware time estimates

---

## Phase 4: Export, Import & Ecosystem

*Interoperate with the quantum computing ecosystem.*

- [ ] **OpenQASM 2.0 export** — standard quantum assembly format
- [ ] **Qiskit Python export** — generate importable `QuantumCircuit` code
- [ ] **OpenQASM import** — parse QASM files into KetGrid circuits
- [ ] **SVG circuit export** — vector graphics for documentation and presentations
- [ ] **Example library browser** — searchable UI with 15+ categorized circuits

---

## Phase 5: Advanced Features (Post-1.0)

*Planned for after the initial stable release.*

- [ ] **Noise simulation** — depolarizing, amplitude/phase damping models; ideal vs noisy comparison
- [ ] **Parameterized circuits** — slider-controlled rotation gates with parameter sweep plots
- [ ] **Custom gate definitions** — define gates as sub-circuits, gate decomposition view
- [ ] **Tutorial mode** — guided interactive lessons, challenges ("Build a circuit that produces |Phi+>")
- [ ] **GPU acceleration** — CUDA state vector simulation for large circuits (20+ qubits)
  - Trait-based backend abstraction (CPU/GPU)
  - CUDA kernels for gate operations
  - VRAM management and circuit chunking
  - Hybrid CPU/GPU scheduling with automatic backend selection
- [ ] **WASM target** — web version via egui's WASM support

---

## Version Milestones

### v0.1.0 — MVP (Phase 1 + 2)
- Visually build a 5-qubit circuit via drag-and-drop
- Real-time simulation results
- Save/load circuits as JSON
- Cross-platform (Windows, macOS, Linux)

### v0.5.0 — Visualization (Phase 3)
- Bloch sphere visualization
- Step-through mode
- Entanglement visualization

### v1.0.0 — Ecosystem (Phase 4)
- Export to Qiskit, OpenQASM, LaTeX
- Import from OpenQASM
- 15+ example circuits
- Stable `ketgrid-core` API, published on crates.io

---

## How to Contribute

Check the roadmap items above and pick something that interests you. The best places to start:

- **Phase 2 items** are the highest priority — they complete the core editing experience
- **Phase 4 export formats** are well-scoped and independent of each other
- **Example circuits** — adding more quantum algorithm examples is always welcome

See [CONTRIBUTING section in README](README.md#contributing) for setup instructions.
