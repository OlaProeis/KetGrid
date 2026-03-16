# Circuit Data Model

Core data structures for quantum circuit representation and serialization.

## Overview

The circuit data model provides the foundation for KetGrid's quantum circuit representation. It defines how qubits, gates, and measurements are structured and stored.

## Types

### `Circuit`

The top-level container for a quantum circuit.

```rust
pub struct Circuit {
    pub qubits: Vec<QubitWire>,       // Named qubit wires
    pub gates: Vec<PlacedGate>,      // Gates positioned on the circuit
    pub measurements: Vec<Measurement>, // Measurement operations
}
```

**Key Methods:**
- `Circuit::new(n)` — Create circuit with n qubits labeled |q₀⟩, |q₁⟩, etc.
- `Circuit::with_labels(labels)` — Create circuit with custom qubit names
- `add_gate(gate, targets, controls, column)` — Add a gate with validation
- `add_measurement(qubit_id, column)` — Add measurement at position
- `num_qubits()` — Get qubit count
- `add_qubit()` — Append a new qubit wire
- `gates_by_column()` — Get gates sorted left-to-right

### `QubitWire`

Represents a single qubit wire in the circuit.

```rust
pub struct QubitWire {
    pub id: usize,      // Wire index (0-based)
    pub label: String,  // Display label (e.g., "|q₀⟩")
}
```

### `PlacedGate`

A gate positioned at a specific column in the circuit.

```rust
pub struct PlacedGate {
    pub gate: GateType,              // Gate variant (H, X, CNOT, etc.)
    pub target_qubits: Vec<usize>,   // Target qubit indices
    pub control_qubits: Vec<usize>,  // Control qubits (for multi-qubit gates)
    pub column: usize,               // Time step position (left-to-right)
    pub parameters: Vec<f64>,        // For parameterized gates (Rx, Ry, Rz)
}
```

### `Measurement`

A measurement operation on a qubit.

```rust
pub struct Measurement {
    pub qubit_id: usize,  // Which qubit is measured
    pub column: usize,    // Time step position
}
```

### `GateType`

Quantum gate variants with serialization support.

```rust
pub enum GateType {
    // Single-qubit gates
    H, X, Y, Z, S, T,
    Rx(f64), Ry(f64), Rz(f64),  // Rotation gates with angle

    // Multi-qubit gates
    Cnot, Cz, Swap, Toffoli,

    // Meta gates
    Barrier, Identity,
    Custom(String),  // User-defined gate
}
```

**Gate Methods:**
- `num_qubits()` — Number of qubits the gate operates on
- `num_controls()` — Required control qubits
- `is_controlled()` — True if gate needs controls
- `is_parameterized()` — True for Rx/Ry/Rz
- `parameters()` — Get rotation angles
- `display_name()` — Human-readable label

## Validation

The circuit validates all operations:

- **Qubit bounds**: Gate targets and controls must exist
- **Non-overlapping**: Targets and controls must be distinct
- **Correct counts**: Target/control counts must match gate requirements

Validation returns `CircuitError`:

```rust
pub enum CircuitError {
    InvalidQubitIndex { index, num_qubits },
    InvalidControlIndex { index, num_qubits },
    OverlappingTargetsAndControls,
    InvalidTargetCount { expected, actual },
    InvalidControlCount { expected, actual },
}
```

## Serialization

All types implement `Serialize` and `Deserialize` via serde:

```rust
use ketgrid_core::{Circuit, GateType};

// Create and serialize
let mut circuit = Circuit::new(2);
circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
let json = serde_json::to_string(&circuit).unwrap();

// Deserialize
let restored: Circuit = serde_json::from_str(&json).unwrap();
```

## Usage Example

```rust
use ketgrid_core::{Circuit, GateType};

// Create a Bell state circuit
let mut circuit = Circuit::new(2);

// H gate on q0 at column 0
circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

// CNOT with control on q0, target on q1 at column 1
circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

// The circuit is ready for simulation or rendering
```

## File Location

- `crates/ketgrid-core/src/circuit.rs` — Circuit, QubitWire, PlacedGate, Measurement
- `crates/ketgrid-core/src/gate.rs` — GateType enum and gate properties
- `crates/ketgrid-core/src/lib.rs` — Public exports
