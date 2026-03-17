# OpenQASM 2.0 Export

Export KetGrid circuits to OpenQASM 2.0 format for interoperability with IBM quantum hardware and the broader quantum computing ecosystem.

## Overview

OpenQASM (Open Quantum Assembly Language) is the standard quantum assembly language used by IBM and the quantum computing community. This feature enables exporting circuits created in KetGrid to the OpenQASM 2.0 format, which can then be executed on IBM Quantum hardware or simulated using Qiskit Aer.

## Usage

### GUI Export

1. Create a circuit in KetGrid
2. Go to **File → Export to OpenQASM…**
3. Choose a location and filename (`.qasm` extension)
4. The exported file contains valid OpenQASM 2.0 code

### Programmatic Export

```rust
use ketgrid_core::Circuit;
use ketgrid_core::format::qasm::circuit_to_qasm;

let circuit = Circuit::new(2);
// ... add gates ...

// Export to OpenQASM string
let qasm_code = circuit_to_qasm(&circuit)?;

// Or use the convenience method
let qasm_code = circuit.to_qasm()?;
```

## Gate Mappings

KetGrid gates map to OpenQASM 2.0 built-in gates as follows:

| KetGrid Gate | OpenQASM Syntax |
|--------------|-----------------|
| `H` | `h q[n];` |
| `X` | `x q[n];` |
| `Y` | `y q[n];` |
| `Z` | `z q[n];` |
| `S` | `s q[n];` |
| `T` | `t q[n];` |
| `Rx(theta)` | `rx(theta) q[n];` |
| `Ry(theta)` | `ry(theta) q[n];` |
| `Rz(theta)` | `rz(theta) q[n];` |
| `Cnot` | `cx q[ctrl],q[target];` |
| `Cz` | `cz q[ctrl],q[target];` |
| `Swap` | `swap q[q1],q[q2];` |
| `Toffoli` | `ccx q[c1],q[c2],q[t];` |
| `Barrier` | `barrier q[n];` |
| `Identity` | *(skipped - no-op)* |
| `Custom(name)` | `// Custom gate: {name}` |

## Output Format

Exported OpenQASM files follow the standard structure:

```qasm
OPENQASM 2.0;
include "qelib1.inc";

qreg q[2];
creg c[2];

h q[0];
cx q[0],q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
```

### Key Features

- **Header**: Standard OpenQASM 2.0 declaration with qelib1.inc include
- **Registers**: `qreg` for quantum registers, `creg` for classical registers (only if measurements exist)
- **Gate Order**: Gates are sorted by column for correct execution order
- **Measurements**: Standard OpenQASM measurement syntax `measure q[q] -> c[c]`
- **No Trailing Newline**: Clean output format

## Example: Bell State Export

A Bell state circuit in KetGrid exports to:

```qasm
OPENQASM 2.0;
include "qelib1.inc";

qreg q[2];
creg c[2];

h q[0];
cx q[0],q[1];

measure q[0] -> c[0];
measure q[1] -> c[1];
```

## Implementation

The export functionality is implemented in `crates/ketgrid-core/src/format/qasm.rs`:

- `QasmError` - Error type for export failures
- `circuit_to_qasm()` - Main export function
- `Circuit::to_qasm()` - Convenience method on `Circuit`

## Error Handling

The export can fail with:

- `QasmError::EmptyCircuit` - Circuit has no qubits
- `QasmError::InvalidGate(String)` - Invalid gate configuration

## See Also

- [Qiskit Export](qiskit-export.md) - Export to Python Qiskit code
- [docs/index.md](index.md) - Full documentation index
