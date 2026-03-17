# Step-Through Mode

Single-gate-column stepping with playback controls for debugging quantum circuits.

## Overview

Step-through mode lets you execute a circuit one column at a time, observing how the quantum state evolves after each layer of gates. This is useful for understanding intermediate states, debugging circuit behavior, and teaching quantum concepts.

## UI Controls

A stepper toolbar appears at the top of the circuit editor:

| Button | Action |
|--------|--------|
| **Step Mode** | Toggle step mode on/off |
| **⏮ Reset** | Return to step 0 (initial \|0…0⟩ state) |
| **◀ Back** | Go back one column (re-simulates from \|0…0⟩) |
| **▶ Fwd** | Apply the next column's gates |
| **⏵ Play / ⏸ Pause** | Auto-advance through columns at 500ms intervals |

A **step counter** (e.g., "Step 2/3") shows the current position.

## Visual Indicators

- **Cyan vertical cursor line** — drawn at the boundary between applied and unapplied columns
- **Dim overlay** — covers the "future" (unapplied) region of the circuit
- **Triangle marker** — small indicator at the top of the cursor line

## State View Integration

When in step mode, the right-panel state view shows the **step simulator's state** (not the full-circuit simulation). The heading displays the current step position.

## Simulation Behavior

- **Step Forward**: applies the next column's gates to the existing state (efficient, no re-simulation)
- **Step Back**: re-simulates from |0…0⟩ up to the previous column (simple, avoids inverse gates)
- **Circuit edits** during step mode reset the step position and recompute the column list

## Architecture

### Simulator (`ketgrid-sim`)

`StateVectorSimulator` exposes two column-wise methods:

- `apply_column(circuit, column)` — applies gates at exactly one column
- `apply_columns_up_to(circuit, max_column)` — applies gates from column 0 through `max_column`

### App State (`ketgrid-gui`)

Step-through state lives on `KetGridApp`:

- `step_mode` — whether stepping is active
- `step_position` — 0 = initial state, k = first k unique columns applied
- `step_columns` — sorted unique gate column indices
- `step_simulator` — separate simulator instance for step state
- `step_playing` / `step_last_advance` — auto-play state

### Circuit View (`circuit_view.rs`)

`CircuitView::show()` accepts `step_cursor_col: Option<usize>`. When set, it draws the cursor line and dim overlay. Gates at columns below the cursor are rendered normally; gates at or above are dimmed.
