# Example Circuits

The `examples/` directory contains five quantum circuit files demonstrating fundamental quantum algorithms and states. These circuits serve as both educational resources and test cases for the KetGrid simulator.

## Circuit Files

All circuits use the `.ket.json` format and can be loaded via `Circuit::from_json_file()`.

### bell.ket.json
**Bell State** — Creates the maximally entangled state |Φ+⟩ = (|00⟩ + |11⟩)/√2.

- **Qubits**: 2
- **Gates**: H (q0), CNOT (q0→q1)
- **Expected Result**: 50% |00⟩, 50% |11⟩

The simplest quantum entanglement, demonstrating non-local correlations between two qubits.

### ghz.ket.json
**GHZ State** — Creates a 3-qubit Greenberger-Horne-Zeilinger state (|000⟩ + |111⟩)/√2.

- **Qubits**: 3
- **Gates**: H (q0), CNOT (q0→q1), CNOT (q1→q2)
- **Expected Result**: 50% |000⟩, 50% |111⟩

Demonstrates multipartite entanglement beyond simple Bell pairs. Useful for testing multi-qubit gates and entanglement visualization.

### deutsch-jozsa.ket.json
**Deutsch-Jozsa Algorithm** — Determines if a function f: {0,1} → {0,1} is constant or balanced.

- **Qubits**: 2
- **Gates**: X (q1), H⊗H, CNOT (q0→q1), H (q0)
- **Function**: f(x) = x (balanced)
- **Expected Result**: |1⟩ on qubit 0 (indicates balanced)

Demonstrates quantum speedup: solves in 1 query what classically requires up to 2 queries.

### teleportation.ket.json
**Quantum Teleportation** — Transfers an arbitrary quantum state from Alice to Bob using shared entanglement.

- **Qubits**: 3 (q0 = input, q1,q2 = Bell pair)
- **Gates**: H(q1), CNOT(q1→q2), CNOT(q0→q1), H(q0)
- **Measurements**: q0, q1 (classical communication to Bob)

Demonstrates that quantum information can be transmitted using only classical bits and pre-shared entanglement. Bob applies corrections based on Alice's measurement results.

### grover-2qubit.ket.json
**Grover's Algorithm (2-Qubit)** — Searches for a marked item in an unsorted database of 4 items.

- **Qubits**: 2
- **Gates**: H⊗H (superposition), CZ (oracle for |11⟩), H⊗H, X⊗X, CZ (diffusion), X⊗X, H⊗H
- **Marked State**: |11⟩
- **Expected Result**: >90% probability for |11⟩

Demonstrates quadratic speedup over classical search. With 2 qubits and one Grover iteration, the marked state is amplified from 25% to ~94% probability.

## Loading Circuits

```rust
use ketgrid_core::Circuit;

// Load an example circuit
let circuit = Circuit::from_json_file("examples/bell.ket.json").unwrap();

// Simulate
use ketgrid_sim::{Simulator, StateVectorSimulator};
let mut sim = StateVectorSimulator::new(circuit.num_qubits());
let result = sim.run(&circuit).unwrap();

// Check probabilities
for (i, prob) in result.probabilities.iter().enumerate() {
    println!("|{:02b}⟩: {:.1}%", i, prob * 100.0);
}
```

## Testing

All examples are validated by integration tests:
- **Loading tests** (`ketgrid-core/tests/example_circuits.rs`): Verify JSON parsing and structure
- **Simulation tests** (`ketgrid-sim/tests/example_simulations.rs`): Verify expected quantum output

Run tests with: `cargo test --workspace`

## Circuit Format

All examples follow the PRD-defined `.ket.json` schema:

```json
{
  "ket_version": "0.1.0",
  "name": "Circuit Name",
  "description": "What this circuit does",
  "qubits": N,
  "gates": [
    { "type": "H", "targets": [0], "column": 0 },
    { "type": "CNOT", "controls": [0], "targets": [1], "column": 1 }
  ],
  "measurements": [
    { "qubit": 0, "column": 2 }
  ]
}
```

## Adding New Examples

1. Create a `.ket.json` file in `examples/`
2. Follow the naming convention: `algorithm-variant.ket.json`
3. Include name, description, and correct gate sequence
4. Add loading test in `ketgrid-core/tests/example_circuits.rs`
5. Add simulation verification in `ketgrid-sim/tests/example_simulations.rs`
