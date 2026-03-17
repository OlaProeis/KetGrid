# Keyboard Shortcuts

> **Scope:** Document all keyboard shortcuts available in the KetGrid circuit editor.

## General Shortcuts

| Shortcut | Action | Context |
|----------|--------|---------|
| `Ctrl+Z` | Undo last operation | Global |
| `Ctrl+Y` or `Ctrl+Shift+Z` | Redo last undone operation | Global |
| `Ctrl+C` | Copy selected gate(s) | When gate(s) selected |
| `Ctrl+V` | Paste copied gate | Global |
| `Delete` | Remove selected gate(s)/measurement(s) | When item(s) selected |
| `Escape` | Cancel pending placement / clear selection | Global |

## Wire Management Shortcuts

| Shortcut | Action | Notes |
|----------|--------|-------|
| `+` or `=` | Add qubit | Adds new qubit wire at the bottom |
| `-` | Remove last qubit | Only works if last qubit has no gates/measurements |

## Gate Placement

| Shortcut | Action | Context |
|----------|--------|---------|
| `Ctrl+Click` | Multi-select gates | Circuit view |
| `Right-click` | Context menu (edit/copy/paste/delete) | On gate or measurement |

## Usage Examples

### Building a Circuit with Keyboard Only

1. Press `+` three times to add three qubits
2. Select the H gate from the palette
3. Click on wire 0, column 0 to place H
4. Press `Escape` to clear selection
5. Select CNOT from the palette
6. Click on wire 0 (control), then wire 1 (target)
7. Press `Ctrl+Z` to undo if needed

### Copy-Paste Workflow

1. Click a gate to select it
2. Press `Ctrl+C` to copy
3. Click another gate position
4. Press `Ctrl+V` to paste the copied gate

### Multi-Selection and Deletion

1. `Ctrl+Click` multiple gates to select them
2. Press `Delete` to remove all selected gates
3. Press `Ctrl+Z` to restore them

## Implementation Details

- Copy-paste uses an internal clipboard (not system clipboard)
- Only gates can be copied; measurements must be placed manually
- The `RemoveQubit` operation is reversible via undo/redo
- Qubits can only be removed if they have no gates or measurements targeting them
- Multiple gates can be selected with `Ctrl+Click` and deleted together
