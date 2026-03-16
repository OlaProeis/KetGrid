# Gate Matrices and Properties

Unitary matrix representations for all quantum gates in KetGrid, implemented using nalgebra for simulation.

## Overview

Each `GateType` variant has a corresponding unitary matrix representation used by the simulation engine to transform quantum state vectors. Single-qubit gates use 2×2 matrices, two-qubit gates use 4×4 matrices, and three-qubit gates use 8×8 matrices.

## Type Aliases

```rust
pub type C = Complex<f64>;                      // Complex number type
pub type GateMatrix = DMatrix<C>;               // Dynamic N×N matrix
pub type GateMatrix2 = Matrix2<C>;              // 2×2 matrix (single-qubit)
pub type GateMatrix4 = Matrix4<C>;              // 4×4 matrix (two-qubit)
pub type GateMatrix8 = SMatrix<C, 8, 8>;       // 8×8 matrix (three-qubit)
```

## Single-Qubit Gates (2×2)

| Gate | Matrix | `matrix()` | `matrix2()` |
|------|--------|------------|-------------|
| **H** (Hadamard) | `1/√2 × [[1, 1], [1, -1]]` | ✓ | ✓ |
| **X** (Pauli-X) | `[[0, 1], [1, 0]]` | ✓ | ✓ |
| **Y** (Pauli-Y) | `[[0, -i], [i, 0]]` | ✓ | ✓ |
| **Z** (Pauli-Z) | `[[1, 0], [0, -1]]` | ✓ | ✓ |
| **S** | `[[1, 0], [0, i]]` | ✓ | ✓ |
| **T** | `[[1, 0], [0, e^(iπ/4)]]` | ✓ | ✓ |
| **Rx(θ)** | `[[cos(θ/2), -i·sin(θ/2)], [-i·sin(θ/2), cos(θ/2)]]` | ✓ | ✓ |
| **Ry(θ)** | `[[cos(θ/2), -sin(θ/2)], [sin(θ/2), cos(θ/2)]]` | ✓ | ✓ |
| **Rz(θ)** | `[[e^(-iθ/2), 0], [0, e^(iθ/2)]]` | ✓ | ✓ |
| **Identity** | `[[1, 0], [0, 1]]` | ✓ | ✓ |

## Multi-Qubit Gates

### CNOT (Controlled-NOT, 4×4)

Control qubit on first wire, target on second:
```
[[1, 0, 0, 0],
 [0, 1, 0, 0],
 [0, 0, 0, 1],
 [0, 0, 1, 0]]
```
Action: `|10⟩ → |11⟩`, `|11⟩ → |10⟩` (flips target when control is |1⟩)

### CZ (Controlled-Z, 4×4)

```
[[1, 0, 0, 0],
 [0, 1, 0, 0],
 [0, 0, 1, 0],
 [0, 0, 0, -1]]
```
Action: Applies Z gate when both qubits are |1⟩.

### SWAP (4×4)

```
[[1, 0, 0, 0],
 [0, 0, 1, 0],
 [0, 1, 0, 0],
 [0, 0, 0, 1]]
```
Action: Exchanges two qubit states (`|01⟩ ↔ |10⟩`).

### Toffoli (CCNOT, 8×8)

Controlled-controlled-NOT: flips the target qubit when both control qubits are |1⟩.

Identity on all basis states except:
- `|110⟩ ↔ |111⟩` (swaps indices 6 and 7)

## Gates Without Matrix Representation

- **Barrier** — Visual separator, returns `None`
- **Custom** — User-defined gates, returns `None` (matrix must be provided externally)

## Usage

```rust
use ketgrid_core::{GateType, C};

// Get dynamic matrix
let h_gate = GateType::H;
if let Some(mat) = h_gate.matrix() {
    // mat is DMatrix<C> with shape (2, 2)
}

// Get optimized 2×2 matrix for single-qubit gates
if let Some(mat2) = h_gate.matrix2() {
    // mat2 is GateMatrix2 for efficient operations
}

// Parameterized gates compute dynamically
let rx = GateType::Rx(std::f64::consts::PI);
let rx_mat = rx.matrix().unwrap();
```

## Location

- Implementation: `crates/ketgrid-core/src/gate.rs`
- Types exported in: `crates/ketgrid-core/src/lib.rs`
- Dependency: `nalgebra` (workspace version 0.33)
