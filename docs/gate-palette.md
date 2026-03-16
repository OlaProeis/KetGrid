# Gate Palette

Gate selection panel with categorized, collapsible gate buttons for circuit building.

## Overview

The gate palette provides a visual catalog of available quantum gates organized by category. Users can click or drag gates to select them for placement on the circuit.

## Categories

| Section | Gates | Description |
|---------|-------|-------------|
| **Basic Gates** | H, X, Y, Z | Fundamental single-qubit operations |
| **Phase Gates** | S, T | Phase rotation gates (π/2 and π/4) |
| **Rotation Gates** | Rx, Ry, Rz | Parameterized rotation gates (default π/2) |
| **Multi-Qubit Gates** | CNOT (⊕), CZ, SWAP (⇄), Toffoli (CC⊕) | Entangling and controlled operations |
| **Meta Gates** | Barrier (\|), Identity (I) | Circuit organization gates |

## UI Features

- **Collapsible sections** — Each category can be expanded/collapsed via egui headers
- **Visual selection** — Selected gates show blue highlight with white text
- **Variable sizing** — Longer labels (e.g., "CC⊕") get wider buttons
- **Scrollable panel** — Content scrolls when exceeding panel height
- **Drag detection** — Both click and drag operations set the active gate

## Implementation

Located in `crates/ketgrid-gui/src/gate_palette.rs`.

### Key Components

```rust
pub struct GatePalette {
    selected_gate: Option<GateType>,    // Currently selected gate for drag
    basic_open: bool,                    // Section open states
    phase_open: bool,
    rotation_open: bool,
    multi_qubit_open: bool,
    meta_open: bool,
}
```

### Public API

| Method | Purpose |
|--------|---------|
| `show(&mut self, ui: &mut Ui)` | Render the complete palette |
| `selected_gate(&self) -> Option<&GateType>` | Get current selection |
| `clear_selection(&mut self)` | Clear after drag completes |

## Integration

The palette is rendered in the left side panel via `app.rs`:

```rust
egui::SidePanel::left("gate_palette")
    .show(ctx, |ui| {
        ui.heading("Gates");
        self.gate_palette.show(ui);
    });
```

## Selection Behavior

1. **Click** on a gate button to select it
2. **Drag** a gate to initiate placement mode
3. Both actions populate `selected_gate` for the editor to consume
4. Editor calls `clear_selection()` after successful placement
