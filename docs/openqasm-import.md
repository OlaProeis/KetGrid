# OpenQASM 2.0 Import

Parse OpenQASM 2.0 files into KetGrid circuits, enabling import from IBM Quantum, Qiskit, and other tools in the quantum computing ecosystem.

## Public API

```rust
use ketgrid_core::format::qasm::{circuit_from_qasm, QasmImportResult};

let qasm = std::fs::read_to_string("circuit.qasm").unwrap();
let result = circuit_from_qasm(&qasm).unwrap();
let circuit = result.circuit;      // Circuit
let warnings = result.warnings;    // Vec<String> — skipped gates

// Convenience method:
let result = Circuit::from_qasm(&qasm).unwrap();
```

## Supported Elements

| QASM Syntax | KetGrid Gate | Notes |
|---|---|---|
| `h q[i]` | `GateType::H` | |
| `x q[i]` | `GateType::X` | |
| `y q[i]` | `GateType::Y` | |
| `z q[i]` | `GateType::Z` | |
| `s q[i]` | `GateType::S` | |
| `t q[i]` | `GateType::T` | |
| `rx(θ) q[i]` | `GateType::Rx(θ)` | |
| `ry(θ) q[i]` | `GateType::Ry(θ)` | |
| `rz(θ) q[i]` | `GateType::Rz(θ)` | Also `p`, `u1` |
| `cx q[c],q[t]` | `GateType::Cnot` | Also `CX` |
| `cz q[c],q[t]` | `GateType::Cz` | |
| `swap q[a],q[b]` | `GateType::Swap` | |
| `ccx q[a],q[b],q[t]` | `GateType::Toffoli` | |
| `id q[i]` | `GateType::Identity` | |
| `barrier q[i],...` | `GateType::Barrier` | One per qubit |
| `measure q[i] -> c[j]` | `Measurement` | |

## Parameter Expressions

The parser supports arithmetic in gate parameters:

- `pi` — π constant
- Numeric literals: `3.14`, `1.5708`, `.5`
- Arithmetic: `pi/2`, `2*pi/3`, `pi + 1`, `-pi/4`
- Parenthesized: `(pi/2)`

## Column Scheduling

Gates are placed using ASAP (as-soon-as-possible) scheduling. Each qubit tracks its next free column; a gate's column is the maximum of all involved qubits' next-free values. Independent gates on different qubits share columns.

```
h q[0];   → col 0
x q[1];   → col 0  (parallel with H)
cx q[0],q[1]; → col 1  (both qubits busy after col 0)
```

## Multiple Registers

Multiple `qreg` declarations are concatenated into a single circuit:

```
qreg a[2];   → qubits 0, 1
qreg b[3];   → qubits 2, 3, 4
```

## Error Handling

- **Missing `qreg`**: returns `QasmError::EmptyCircuit`
- **Undefined register**: returns `QasmError::ParseError`
- **Index out of bounds**: returns `QasmError::ParseError`
- **Unsupported gates**: skipped with a warning string in `QasmImportResult.warnings`
- **Comments** (`//`): stripped before parsing

## GUI Integration

File → Import from OpenQASM… opens a file picker for `.qasm` files, replaces the current circuit, and displays any warnings in the status bar.

## Files

| File | Role |
|---|---|
| `crates/ketgrid-core/src/format/qasm.rs` | Parser, gate mapping, `circuit_from_qasm()` |
| `crates/ketgrid-core/src/format/mod.rs` | Re-exports |
| `crates/ketgrid-gui/src/app.rs` | `import_qasm_dialog()`, menu item |

## Dependencies

- `nom = "7"` — parser combinator library (workspace dependency)
