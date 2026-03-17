# Undo/Redo System

Operation-based undo/redo for all circuit edits.

## Architecture

**File:** `crates/ketgrid-gui/src/history.rs`

The system uses an `EditHistory` struct containing two stacks (undo and redo) of `EditOperation` values. Each operation stores enough data to reverse itself and be reapplied.

### EditOperation variants

| Variant | Tracked by | Undo behaviour | Redo behaviour |
|---|---|---|---|
| `AddGate` | Gate placement (click, drag-drop, paste) | Removes gate at stored index | Re-inserts gate at stored index |
| `RemoveGate` | Delete gate (context menu) | Re-inserts gate at stored index | Removes gate at stored index |
| `AddMeasurement` | Measurement placement | Removes measurement at stored index | Re-inserts measurement |
| `RemoveMeasurement` | Delete measurement (context menu) | Re-inserts measurement | Removes measurement |
| `EditParam` | Parameter editor (Rx/Ry/Rz angle) | Restores old gate type | Applies new gate type |
| `AddQubit` | Edit > Add Qubit | Pops last qubit | Adds qubit |
| `ReplaceCircuit` | New Circuit, Open File | `mem::swap` with stored circuit | `mem::swap` again |

### Stack behaviour

- New operations push to the undo stack and **clear** the redo stack.
- Undo pops from the undo stack, applies the reverse, and pushes to the redo stack.
- Redo pops from the redo stack, reapplies forward, and pushes back to the undo stack.
- The undo stack is capped at **100 entries** (oldest entries are dropped).

## Key bindings

| Shortcut | Action |
|---|---|
| `Ctrl+Z` | Undo |
| `Ctrl+Y` or `Ctrl+Shift+Z` | Redo |

## Menu

**Edit > Undo / Redo** — menu items are greyed out when the corresponding stack is empty.

## Test strategy

Unit tests in `history.rs` cover:
- Undo/redo for every operation variant
- Bell state scenario: build → Ctrl+Z twice → empty → Ctrl+Y → restore
- New operations clearing the redo stack
- Stack size limit enforcement
- Empty history edge cases
