# Drag-and-Drop Gate Placement

Gate placement from the palette onto qubit wires in the circuit editor.

## Interaction Modes

### Click-to-Select, Click-to-Place (Primary)
1. Click a gate button in the palette — it highlights as selected.
2. Hover over the circuit — a blue ghost gate and wire highlight show the drop target.
3. Click on a wire position — the gate is placed and simulation refreshes.
4. Click the same gate button again to deselect (toggle).

### Drag-from-Palette (Secondary)
1. Start dragging a gate button in the palette.
2. Release the pointer over the circuit area.
3. The gate is placed at the nearest wire/column under the pointer.

### Multi-Qubit Gates
Multi-qubit gates require multiple clicks (one per qubit):

| Gate | Clicks | Order |
|------|--------|-------|
| CNOT | 2 | Control → Target |
| CZ | 2 | Control → Target |
| SWAP | 2 | Target A → Target B |
| Toffoli | 3 | Control 1 → Control 2 → Target |

Visual feedback during multi-qubit placement:
- Green indicator on already-selected qubits (with control dot).
- Blue ghost gate at hover position.
- Vertical connecting line between selected and hover qubits.
- Status bar shows progress ("Select control qubit (1/2)").

### Cancellation
- **Escape key**: Cancels pending placement and clears palette selection.
- **Selecting a different gate**: Cancels current pending and starts new placement.
- **Toggle click**: Clicking the selected gate button again deselects it.

## Architecture

### `editor.rs` — State Machine
Core types:
- `DropTarget { qubit_idx, column }` — grid position on the circuit.
- `MultiQubitPending { gate, selected_qubits, column }` — accumulates qubit clicks.
- `GatePlacement { gate, target_qubits, control_qubits, column }` — ready for `circuit.add_gate()`.
- `EditorState` — owns `multi_qubit_pending`, exposes `try_place()`.

`try_place(gate, target)` logic:
- Single-qubit: returns `GatePlacement` immediately.
- Multi-qubit: stores first click as pending, accumulates subsequent clicks, returns `GatePlacement` when all qubits selected.
- Controls are filled first (ordered by click), then targets.
- Same-qubit clicks are rejected. Gate mismatch resets pending.

### `circuit_view.rs` — Visual Feedback
Layout constants reused from gate rendering:
- `WIRE_SPACING = 60.0`, `COLUMN_WIDTH = 64.0`, `WIRE_MARGIN_TOP = 40.0`.

New functionality:
- `hit_test_at(pos, ...)` — maps screen position to nearest valid `DropTarget`.
- `hit_test(pos, num_qubits)` — public method for cross-panel drag detection.
- `draw_drop_indicator(...)` — blue ghost gate box with label.
- `draw_pending_qubit_indicator(...)` — green box with control dot.
- Wire highlight: subtle blue tint on the target wire row.
- Crosshair cursor when hovering with a gate active.

### `app.rs` — Coordination
The `update()` method orchestrates the flow:
1. Palette renders → may set `selected_gate`.
2. Escape/mismatch detection → cancels stale pending.
3. Active gate resolved (pending gate takes priority over palette).
4. Circuit view renders with active gate + editor state → returns clicked target.
5. `editor_state.try_place()` → if placement returned, `circuit.add_gate()` + refresh sim.
6. Cross-panel drag fallback: checks pointer release over circuit area.
7. Status bar shows contextual text from `editor_state.status_text()`.

### `gate_palette.rs` — Toggle Selection
Gate buttons now toggle: clicking the same gate deselects it (sets `selected_gate = None`).
