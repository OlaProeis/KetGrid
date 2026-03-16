# Wire Management

Qubit wire lifecycle operations: add, remove, rename, and reorder.

## Types

### `QubitWire`
Represents a named qubit wire in the circuit.

```rust
pub struct QubitWire {
    pub id: usize,      // Wire index (0-based)
    pub label: String,  // Display label (e.g., "|q₀⟩", "|ancilla⟩")
}
```

**Constructors:**
- `QubitWire::new(id, label)` — Create with custom label
- `QubitWire::with_default_label(id)` — Create with auto-generated label "|q{n}⟩"

## Circuit Methods

### `add_qubit()`
Appends a new qubit wire to the circuit with auto-generated label.

### `remove_qubit(qubit_id: usize) -> Result<(), CircuitError>`
Removes a qubit wire and renumbers remaining qubits.

**Validation:**
- Returns `CircuitError::InvalidQubitIndex` if qubit doesn't exist
- Returns `CircuitError::QubitInUse` if qubit has gates or measurements targeting it

**Index Management:**
- All qubit IDs are renumbered after removal
- Gate target/control indices are decremented for indices > removed qubit
- Measurement qubit_id values are decremented for indices > removed qubit

### `rename_qubit(qubit_id: usize, new_label: impl Into<String>) -> Result<(), CircuitError>`
Changes the display label of a qubit.

### `reorder_qubits(permutation: &[usize]) -> Result<(), CircuitError>`
Reorders qubit wires according to a permutation vector.

**Example:** `permutation = [2, 0, 1]` means:
- Old qubit 2 → New qubit 0
- Old qubit 0 → New qubit 1
- Old qubit 1 → New qubit 2

**Validation:**
- Permutation length must equal number of qubits
- Each index 0..n-1 must appear exactly once
- Returns `CircuitError::InvalidPermutation` on invalid input

**Index Management:**
- All gate target/control indices are remapped
- All measurement qubit_id values are remapped
- Qubit IDs are renumbered to match new positions

## Error Types

- `QubitInUse { qubit_id }` — Attempted to remove a qubit that has gates/measurements
- `InvalidPermutation { message }` — Invalid reordering specification
