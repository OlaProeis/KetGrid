# Circuit Renderer

The circuit renderer (`crates/ketgrid-gui/src/circuit_view.rs`) draws quantum circuits in the egui central panel using immediate-mode painting.

## Layout

| Constant | Value | Purpose |
|----------|-------|---------|
| `WIRE_SPACING` | 60 px | Vertical distance between qubit wires |
| `WIRE_MARGIN_TOP` | 40 px | Top padding before first wire |
| `LABEL_WIDTH` | 55 px | Reserved space for qubit labels on the left |
| `COLUMN_WIDTH` | 64 px | Horizontal spacing between time-step columns |
| `GATE_BOX_SIZE` | 36 px | Width/height of standard gate boxes |

Qubit wires run horizontally; gates are positioned on a column grid where each column represents one time step. The column X coordinate is computed from `PlacedGate.column`, and the Y coordinate from the qubit index.

## Gate Rendering by Type

| Gate Type | Visual | Notes |
|-----------|--------|-------|
| H, X, Y, Z, S, T, I, Custom | Labeled box | `display_name()` text centered in a rounded rect |
| Rx(θ), Ry(θ), Rz(θ) | Wide labeled box | Shows parameter value, e.g. `Rx(3.1)` |
| CNOT | ● + ⊕ + vertical line | Filled dot on control, circle-with-plus on target |
| Toffoli | ●● + ⊕ + vertical line | Two control dots, one ⊕ target |
| CZ | ● + ● + vertical line | Filled dots on both control and target (symmetric) |
| SWAP | ✕ + ✕ + vertical line | Two diagonal crosses connected vertically |
| Barrier | Dashed vertical line | Faded, non-operational separator |
| Measurement | Meter icon (box + arc + arrow) | Drawn at the measurement's column position |

## Key Functions

- `CircuitView::show()` — entry point, draws background, wires, gates, measurements.
- `draw_gate()` — dispatches to specialized renderers per gate type.
- `draw_gate_box()` — labeled rectangular box for single-qubit gates.
- `draw_controlled_not()` — control dots + ⊕ target + vertical line.
- `draw_cz()` — two control dots + vertical line.
- `draw_swap()` — two ✕ crosses + vertical line.
- `draw_oplus()` — the ⊕ (XOR/NOT target) symbol.
- `draw_measurement()` — meter icon with semicircular arc and arrow.

## Data Flow

```
Circuit.gates_by_column() → Vec<&PlacedGate>
  └─ each PlacedGate has: .gate (GateType), .target_qubits, .control_qubits, .column

Circuit.measurements → Vec<Measurement>
  └─ each Measurement has: .qubit_id, .column
```

The renderer reads the circuit model immutably — no mutations during rendering.

## Demo Circuit

`app.rs` initializes with a Bell state demo: H on q0 (col 0), CNOT q0→q1 (col 1), measurements on q0/q1 (col 2). Run `cargo run -p ketgrid-gui` to see it.
