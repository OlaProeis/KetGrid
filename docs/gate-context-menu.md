# Gate Context Menu

Interactive right-click context menu for gates and measurements in the circuit editor.

## Features

- **Right-click any gate** to open the context menu
- **Multi-select** gates with Ctrl+Click
- **Selection highlighting** with yellow border
- **Keyboard shortcuts** for paste (Ctrl+V)

## Context Menu Options

### For Gates

| Option | Description |
|--------|-------------|
| Edit Parameters... | Opens parameter editor for Rx/Ry/Rz gates with angle slider (-360° to 360°) |
| Copy | Copies gate to clipboard |
| Paste Here | Pastes copied gate at the clicked gate's column position |
| Delete | Removes the gate from the circuit |

### For Measurements

| Option | Description |
|--------|-------------|
| Delete | Removes the measurement from the circuit |

## Usage

1. **Right-click a gate** to open its context menu
2. **Select "Edit Parameters"** to modify rotation angles
3. **Copy and Paste** to duplicate gates at specific positions
4. **Ctrl+Click multiple gates** to select them (visual feedback only - batch operations planned)

## Implementation Details

### Editor State (`crates/ketgrid-gui/src/editor.rs`)

```rust
pub struct EditorState {
    selected_gates: HashSet<GateId>,     // Multi-selection support
    clipboard: Option<ClipboardContent>,  // Copy-paste buffer
    context_menu: Option<ContextMenuState>, // Active context menu
    editing_gate: Option<usize>,          // Gate being edited
}
```

### Clipboard Content

```rust
pub enum ClipboardContent {
    Single {
        gate: PlacedGate,
        original_column: usize,
        original_qubits: Vec<usize>,
    },
}
```

### Circuit View Integration

The circuit view tracks bounding boxes for hit-testing:
- `gate_rects: Vec<(usize, Rect)>` for gate hit-testing
- `measurement_rects: Vec<(usize, Rect)>` for measurement hit-testing

### Context Menu State

```rust
pub enum ContextMenuType {
    Gate,
    Measurement,
}

pub struct ContextMenuState {
    position: Pos2,
    item_index: usize,
    item_type: ContextMenuType,
}
```

## Key Files

| File | Purpose |
|------|---------|
| `crates/ketgrid-gui/src/circuit_view.rs` | Gate/measurement rendering and hit-testing |
| `crates/ketgrid-gui/src/editor.rs` | Selection, clipboard, context menu state |
| `crates/ketgrid-gui/src/app.rs` | Context menu UI and action handling |
| `crates/ketgrid-core/src/circuit.rs` | Gate/measurement removal and modification APIs |

## Future Enhancements

- Multi-gate copy-paste for batch operations
- Cut operation (copy + delete)
- Drag-to-move gates between positions
- Keyboard-only editing mode
