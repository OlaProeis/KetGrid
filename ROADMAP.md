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

## Phase 5: Post-v0.1.0 Major Workstreams

Three complementary workstreams have been defined for post-v0.1.0 development. These can proceed in parallel by separate contributors.

### Workstream A: GPU Acceleration via wgpu Compute Shaders

Push the simulation ceiling from ~14 qubits to 25+ qubits by offloading state vector operations to the GPU.

**Why wgpu (not CUDA):** Cross-platform (Vulkan, DX12, Metal, WebGPU), mature Rust ecosystem, already linked via eframe, pure Cargo build, and enables future WASM target.

- [ ] **Task 37** — Research wgpu compute shader feasibility (f32 precision analysis)
- [ ] **Task 38** — Design `QuantumSimulator` trait with CPU + GPU implementations
- [ ] **Task 39** — Implement wgpu state vector shaders (single-qubit, controlled, swap gates)
- [ ] **Task 40** — GPU memory management (VRAM estimation, chunking, buffer pools)
- [ ] **Task 41** — Hybrid CPU/GPU scheduling (auto-select: CPU ≤12 qubits, GPU 13+)
- [ ] **Task 42** — Post-implementation benchmarking and cross-platform validation

**Key technical risk:** WGSL uses f32; acceptable for circuits up to ~20 qubits with <1e-5 relative error.

**Expected gains:** 20 qubits: 500ms → 10ms; 25 qubits: 30s → 200ms.

### Workstream B: Quantum Phenomena Visualization

Make quantum mechanics visceral by visualizing *how* and *why* states transform — not just the final numbers. A presentation layer on top of the existing CPU simulator (can start immediately).

- [ ] **Task 43.1** — Amplitude Flow Engine: compute significant amplitude transfers between basis states
- [ ] **Task 43.2** — Amplitude Flow Renderer: animated Sankey-like diagrams with phase-colored flows
- [ ] **Task 43.3** — Measurement Collapse Animation: probability wheel, state renormalization visualization
- [ ] **Task 43.4** — Entanglement Propagation: animate partner qubits snapping to correlated states
- [ ] **Task 43.5** — Quantum Playground Mode: combined visualization for ≤6 qubit circuits

**Novelty:** No existing tool combines amplitude flow, animated measurement collapse, and entanglement propagation.

### Workstream C: True Quantum Emulator (Shots, Black Box, Bell Test)

Model the experience of interacting with real quantum hardware — you never see the state vector, only measurement outcomes.

- [ ] **Task 45** — Shots-based statistical simulation: animated histogram building, convergence metrics
- [ ] **Task 46** — Black Box / Measurement-Only Mode: hide state vector, "?" Bloch spheres, "Peek" button
- [ ] **Task 47** — Correlation Discovery Dashboard: correlation matrix, scatter plots, live annotations
- [ ] **Task 48** — Interactive Bell Test / CHSH Violation: run Bell experiments, watch S-value cross classical limit |S|=2

**Why this matters:** The Bell inequality violation is the foundational proof of quantum mechanics. No existing tool lets you run it interactively.

---

### Future Considerations (Beyond Current Workstreams)

- [ ] **Noise simulation** — depolarizing, amplitude/phase damping models; ideal vs noisy comparison
- [ ] **Parameterized circuits** — slider-controlled rotation gates with parameter sweep plots
- [ ] **Custom gate definitions** — define gates as sub-circuits, gate decomposition view
- [ ] **Tutorial mode** — guided interactive lessons, challenges ("Build a circuit that produces |Φ+⟩")
- [ ] **WASM target** — web version via egui's WASM support (enabled by wgpu compute shaders)

### Relationship Between Workstreams

```
         ketgrid-sim                    ketgrid-gui
    ┌─────────────────┐           ┌──────────────────────────┐
    │ QuantumSimulator│           │   Existing UI            │
    │     trait (38)   │           │   Step-through (28)      │
    │         │        │           │   Entanglement (29)      │
    │    ┌────┴────┐   │           │         │                 │
    │  CPU sim   wgpu  │           │   ┌─────┴──────┐         │
    │  (existing) sim  │           │   │ Quantum     │         │
    │            (39)  │           │   │ Phenomena   │         │
    │              │   │           │   │ Viz (43)    │         │
    │         Memory   │           │   └────────────┘         │
    │         mgmt(40) │           │                          │
    │              │   │           │   ┌──────────────────┐   │
    │         Scheduler│           │   │ True Emulator    │   │
    │           (41)   │           │   │ Shots (45)       │   │
    │              │   │  ┌────────┤   │ Black Box (46)   │   │
    │  ShotAccum.  │   │  │        │   │ Correlations(47) │   │
    │  Correlation │◄──┼──┘        │   │ Bell Test (48)   │   │
    │   (45, 47)   │   │           │   └──────────────────┘   │
    └─────────────────┘           └──────────────────────────┘
       Workstream A                  Workstreams B + C
```

---

## Version Milestones

### v0.1.0 — MVP ✅ (Released 2026-03-17)
- Visually build quantum circuits via drag-and-drop
- Real-time simulation results with Bloch sphere and step-through
- Save/load circuits as JSON, export to OpenQASM/Qiskit/SVG
- 21 example circuits with browsable library
- Cross-platform release builds (Windows, macOS, Linux)

### v0.2.0 — Quantum Phenomena Visualization & True Emulator
- Amplitude flow visualization (Sankey-like animated diagrams)
- Measurement collapse animations with probability wheels
- Entanglement propagation visualization
- Quantum Playground mode for ≤6 qubit circuits
- Shots-based statistical simulation with animated histograms
- Black Box / Measurement-Only mode (state vector hidden)
- Correlation discovery dashboard with heatmaps
- Interactive Bell Test / CHSH violation experiments

### v0.3.0 — GPU Acceleration
- wgpu compute shader backend for state vector simulation
- Hybrid CPU/GPU scheduling (auto-select optimal backend)
- 25+ qubit simulation support (target: <200ms for 25 qubits)
- Cross-platform GPU support (Vulkan/DX12/Metal)

### v0.4.0 — Performance & Polish
- Simulation performance optimization (target: <100ms for 20 qubits)
- Noise simulation (depolarizing, amplitude/phase damping)
- Parameterized circuit UI with live sliders

### v1.0.0 — Full Platform
- Custom gate definitions and decomposition view
- Tutorial mode with guided challenges
- WASM web target via WebGPU
- Stable `ketgrid-core` API, published on crates.io

---

## How to Contribute

Check the roadmap items above and pick something that interests you. The best places to start:

- **Performance optimization** — profiling and improving simulation speed
- **Phase 5 features** — noise simulation, parameterized circuits
- **Example circuits** — adding more quantum algorithm examples is always welcome

See [CONTRIBUTING section in README](README.md#contributing) for setup instructions.
