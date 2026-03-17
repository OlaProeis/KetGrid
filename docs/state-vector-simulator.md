# State Vector Simulator

Quantum circuit simulation using full state vector representation. Located in `crates/ketgrid-sim/`.

## Architecture

Two main types in `state_vector.rs`:

- **`StateVector`** — Internal data structure holding a complex amplitude vector (length 2^n for n qubits). Pure data holder; gate application is handled by the simulator.
- **`StateVectorSimulator`** — Public API wrapping `StateVector`. Implements the `Simulator` trait. Owns pre-allocated buffers for zero-allocation gate application and supports parallel, incremental, and fused simulation paths.

## Qubit Ordering

Big-endian convention: qubit 0 is the most significant bit.

```
|q₀ q₁ … qₙ₋₁⟩  →  index = q₀·2ⁿ⁻¹ + q₁·2ⁿ⁻² + … + qₙ₋₁
```

This matches the gate matrix definitions in `ketgrid-core/src/gate.rs` (e.g., CNOT matrix row 2 = |10⟩ where qubit 0 is the control).

## Gate Application Algorithm

For an m-qubit gate acting on circuit qubits `[g₀, g₁, …]`:

1. Pre-compute bit layout: `GateBitLayout` maps circuit qubits to bit positions and identifies non-gate bits.
2. Iterate over all 2^(n−m) combinations of non-gate-qubit bits (sequential path) **or** iterate over all 2^n output amplitudes (parallel path).
3. For each subspace, collect the 2^m amplitudes, apply matrix-vector multiply, write results back.

Qubit list for each gate: **controls first, then targets** (matching the gate matrix MSB→LSB ordering).

Complexity: O(2^n · 2^m) per gate — standard for state vector simulation.

## Performance Optimizations

### Pre-allocated Buffers

`StateVectorSimulator` owns three reusable buffers, all sized at construction:

| Buffer | Size | Purpose |
|--------|------|---------|
| `amps_buffer` | max(2^n, 8) | Amplitude workspace for sequential gate application |
| `indices_buffer` | max(2^n, 8) | Index workspace for sequential gate application |
| `scratch` | 2^n | Full state copy for parallel gate application |

These eliminate per-gate heap allocations in the hot loop.

### Sequential vs Parallel Gate Application

| Constant | Value | Purpose |
|----------|-------|---------|
| `PARALLEL_THRESHOLD` | 12 qubits | Below this, `apply_gate_sequential` is used |
| `CHECKPOINT_MAX_QUBITS` | 15 qubits | Max qubits for saving column checkpoints |

- **Sequential path** (`apply_gate_sequential`): Iterates subspaces using the `amps_buffer` and `indices_buffer`. Zero heap allocation per gate.
- **Parallel path** (`apply_gate_parallel`): Copies state to `scratch`, then uses `rayon::par_iter_mut` over the output slice. Each parallel task reads from immutable `scratch` and writes to its own exclusive `&mut` output slot — no unsafe code needed.

### Column Checkpoints (Incremental Simulation)

For circuits ≤ 15 qubits, `apply_circuit()` saves a `ColumnCheckpoint` at each column boundary (a full copy of the state vector). `apply_circuit_from_column(circuit, col)` restores the latest checkpoint at or before `col` and re-simulates only from that point forward.

This enables the GUI's `dirty_column` tracking: when a gate is edited at column 10 of a 14-qubit circuit, only columns 10+ are re-simulated.

### Gate Fusion

`apply_circuit_optimized()` composes consecutive single-qubit gates on the same qubit into a single fused matrix using nalgebra matrix multiplication. This reduces the number of state vector traversals. The fused path does not save column checkpoints (intended for one-shot simulation).

Example: H → Z → H on the same qubit becomes a single X gate matrix applied once.

### Current Performance Status

These optimizations (buffers, parallelism, checkpoints, fusion) improve throughput but have **not yet achieved the target latency** for smooth real-time editing at 14+ qubits. Further optimization (GPU compute, sparse state vectors, or algorithmic improvements) may be needed. The auto-sim threshold remains at 15 qubits.

## Simulation Paths

| Method | Use Case | Checkpoints | Gate Fusion |
|--------|----------|-------------|-------------|
| `apply_circuit()` | Standard full simulation | Yes (≤15 qubits) | No |
| `apply_circuit_from_column(col)` | Incremental re-sim after edit | Yes | No |
| `apply_circuit_optimized()` | One-shot fused simulation | No | Yes |
| `apply_columns_up_to(col)` | Step-through mode | No | No |
| `apply_column(col)` | Single column step | No | No |

## Usage

```rust
use ketgrid_core::{Circuit, GateType};
use ketgrid_sim::StateVectorSimulator;

let mut circuit = Circuit::new(2);
circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

let mut sim = StateVectorSimulator::new(2);
sim.apply_circuit(&circuit);

let probs = sim.probabilities(); // [0.5, 0.0, 0.0, 0.5]
let amps = sim.state_vector();   // Complex<f64> slice, length 4
```

## Simulator Trait

`StateVectorSimulator` implements `Simulator::run()`, which resets state to |0…0⟩, applies the circuit, and returns a `SimulationResult` containing:
- `state_vector` — `Option<Vec<Complex<f64>>>`
- `probabilities` — `Vec<f64>`
- `num_qubits` — qubit count
- `measurements` — (not yet populated, reserved for future measurement simulation)

## Entanglement Analysis

Post-simulation analysis in `state_vector.rs`:

- **`reduced_density_matrix(qubit)`** — Partial trace yielding a 2×2 reduced density matrix.
- **`single_qubit_purity(qubit)`** — Tr(ρ²) for entanglement detection (< 1.0 = entangled).
- **`entanglement_clusters()`** — Union-find grouping of entangled qubits by purity threshold.

## Feature Flags

- **Default (no features):** Custom simulator using `ketgrid-core` gate matrices. Always available.
- **`quantrs2`:** Optional dependency on the `quantrs2` crate. Reserved for future QuantRS2-backed simulation.

## Dependencies

- `nalgebra` — Complex vector and matrix operations
- `rayon` — Data-parallel gate application (≥12 qubits)

## Supported Gates

All gates with a unitary matrix in `GateType::matrix()`:
- Single-qubit: H, X, Y, Z, S, T, Rx(θ), Ry(θ), Rz(θ), Identity
- Two-qubit: CNOT, CZ, SWAP
- Three-qubit: Toffoli

Barrier and Custom gates (no matrix) are silently skipped.

## Tests

29 tests in `state_vector::tests` covering:
- Bell state, GHZ state, single-gate operations
- Self-inverse properties (HH=I, XX=I)
- All multi-qubit gates (CNOT, CZ, SWAP, Toffoli)
- Parameterized gates (Rx)
- Edge cases (barrier, identity, empty circuit)
- Simulator trait integration
- Gate fusion verification (H-Z-H = X, mixed single/multi-qubit)
- Incremental simulation (checkpoint restore + partial re-sim)
- Entanglement detection and cluster analysis
