# State Vector Simulator

Quantum circuit simulation using full state vector representation. Located in `crates/ketgrid-sim/`.

## Architecture

Two main types in `state_vector.rs`:

- **`StateVector`** — Internal data structure holding a complex amplitude vector (length 2^n for n qubits). Provides the core `apply_gate()` operation that multiplies a unitary matrix into the relevant subspace of the state vector.
- **`StateVectorSimulator`** — Public API wrapping `StateVector`. Implements the `Simulator` trait. Provides `apply_circuit()`, `state_vector()`, and `probabilities()`.

## Qubit Ordering

Big-endian convention: qubit 0 is the most significant bit.

```
|q₀ q₁ … qₙ₋₁⟩  →  index = q₀·2ⁿ⁻¹ + q₁·2ⁿ⁻² + … + qₙ₋₁
```

This matches the gate matrix definitions in `ketgrid-core/src/gate.rs` (e.g., CNOT matrix row 2 = |10⟩ where qubit 0 is the control).

## Gate Application Algorithm

For an m-qubit gate acting on circuit qubits `[g₀, g₁, …]`:

1. Map circuit qubits to bit positions: `bit_pos[i] = n − 1 − g[i]`
2. Iterate over all 2^(n−m) combinations of non-gate-qubit bits
3. For each combination, collect the 2^m amplitudes in the gate subspace
4. Apply matrix-vector multiply within that subspace
5. Write results back

Qubit list for each gate: **controls first, then targets** (matching the gate matrix MSB→LSB ordering).

Complexity: O(2^n · 2^m) per gate — standard for state vector simulation.

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

## Feature Flags

- **Default (no features):** Custom simulator using `ketgrid-core` gate matrices. Always available.
- **`quantrs2`:** Optional dependency on the `quantrs2` crate (v0.1.2). Reserved for future QuantRS2-backed simulation. The custom simulator serves as the fallback.

## Supported Gates

All gates with a unitary matrix in `GateType::matrix()`:
- Single-qubit: H, X, Y, Z, S, T, Rx(θ), Ry(θ), Rz(θ), Identity
- Two-qubit: CNOT, CZ, SWAP
- Three-qubit: Toffoli

Barrier and Custom gates (no matrix) are silently skipped.

## Tests

15 tests in `state_vector::tests` covering:
- Bell state, GHZ state, single-gate operations
- Self-inverse properties (HH=I, XX=I)
- All multi-qubit gates (CNOT, CZ, SWAP, Toffoli)
- Parameterized gates (Rx)
- Edge cases (barrier, identity, empty circuit)
- Simulator trait integration
