//! Drag-and-drop editor state for gate placement on the circuit.

use ketgrid_core::circuit::PlacedGate;
use ketgrid_core::gate::GateType;
use std::collections::HashSet;

/// Target position for gate placement on the circuit grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DropTarget {
    pub qubit_idx: usize,
    pub column: usize,
}

/// Clipboard content for copy-paste operations.
#[derive(Debug, Clone)]
pub enum ClipboardContent {
    /// A single gate with its original position info for relative paste.
    Single {
        gate: PlacedGate,
        /// Original column for relative positioning
        original_column: usize,
        /// Original qubits for relative positioning
        original_qubits: Vec<usize>,
    },
    /// Multiple gates (for future multi-select copy).
    Multiple(Vec<(PlacedGate, usize, Vec<usize>)>),
}

/// Identifies a gate in the circuit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GateId {
    pub index: usize,
}

/// Pending multi-qubit gate: accumulating qubit selections click-by-click.
#[derive(Debug, Clone)]
pub struct MultiQubitPending {
    pub gate: GateType,
    pub selected_qubits: Vec<usize>,
    pub column: usize,
}

/// Complete gate placement ready for circuit insertion.
#[derive(Debug, Clone)]
pub struct GatePlacement {
    pub gate: GateType,
    pub target_qubits: Vec<usize>,
    pub control_qubits: Vec<usize>,
    pub column: usize,
}

/// Type of item in a context menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContextMenuType {
    /// Context menu for a gate.
    Gate,
    /// Context menu for a measurement.
    Measurement,
}

/// Context menu state for gate/measurement operations.
#[derive(Debug, Clone)]
pub struct ContextMenuState {
    /// Position where the context menu should appear.
    pub position: egui::Pos2,
    /// Index of the item that was right-clicked (primary selection).
    pub item_index: usize,
    /// Type of item (gate or measurement).
    pub item_type: ContextMenuType,
}

impl ContextMenuState {
    pub fn new(position: egui::Pos2, item_index: usize, item_type: ContextMenuType) -> Self {
        Self {
            position,
            item_index,
            item_type,
        }
    }
}

/// Editor state for drag-and-drop gate placement.
///
/// Tracks multi-qubit pending state, gate selections, and provides placement logic.
/// Single-qubit gates are placed immediately on click.
/// Multi-qubit gates accumulate qubit selections (control first, then target).
#[derive(Debug, Default)]
pub struct EditorState {
    /// Pending multi-qubit gate placement (accumulating qubit clicks).
    pub multi_qubit_pending: Option<MultiQubitPending>,
    /// Currently selected gates (for multi-select with Ctrl+Click).
    pub selected_gates: HashSet<GateId>,
    /// Last clicked gate (single selection without Ctrl).
    pub last_selected_gate: Option<GateId>,
    /// Clipboard for copy-paste operations.
    pub clipboard: Option<ClipboardContent>,
    /// Context menu state.
    pub context_menu: Option<ContextMenuState>,
    /// Currently edited gate index (for parameter editing).
    pub editing_gate: Option<usize>,
}

impl EditorState {
    /// Attempt to place a gate at the given target.
    ///
    /// For single-qubit gates, returns placement immediately.
    /// For multi-qubit gates, accumulates qubit selections and returns
    /// placement only when all required qubits have been selected.
    /// Controls are selected first, then targets.
    pub fn try_place(
        &mut self,
        gate: &GateType,
        target: DropTarget,
    ) -> Option<GatePlacement> {
        let total_needed = gate.num_qubits();

        if total_needed == 1 {
            self.multi_qubit_pending = None;
            return Some(GatePlacement {
                gate: gate.clone(),
                target_qubits: vec![target.qubit_idx],
                control_qubits: vec![],
                column: target.column,
            });
        }

        // Check if pending matches current gate; reset if mismatched
        let pending_matches = self
            .multi_qubit_pending
            .as_ref()
            .map_or(false, |p| p.gate == *gate);

        if pending_matches {
            let pending = self.multi_qubit_pending.as_mut().unwrap();

            if pending.selected_qubits.contains(&target.qubit_idx) {
                return None;
            }

            pending.selected_qubits.push(target.qubit_idx);

            if pending.selected_qubits.len() == total_needed {
                let pending = self.multi_qubit_pending.take().unwrap();
                let num_controls = pending.gate.num_controls();
                let control_qubits = pending.selected_qubits[..num_controls].to_vec();
                let target_qubits = pending.selected_qubits[num_controls..].to_vec();

                return Some(GatePlacement {
                    gate: pending.gate,
                    target_qubits,
                    control_qubits,
                    column: pending.column,
                });
            }

            None
        } else {
            self.multi_qubit_pending = Some(MultiQubitPending {
                gate: gate.clone(),
                selected_qubits: vec![target.qubit_idx],
                column: target.column,
            });
            None
        }
    }

    /// Cancel any pending multi-qubit placement.
    pub fn cancel_pending(&mut self) {
        self.multi_qubit_pending = None;
    }

    /// Whether we're in the middle of placing a multi-qubit gate.
    pub fn is_awaiting_more_qubits(&self) -> bool {
        self.multi_qubit_pending.is_some()
    }

    /// Toggle gate selection with Ctrl key.
    /// Returns true if the gate is now selected.
    pub fn toggle_gate_selection(&mut self, gate_index: usize, ctrl_pressed: bool) -> bool {
        let gate_id = GateId { index: gate_index };

        if ctrl_pressed {
            // Toggle selection
            if self.selected_gates.contains(&gate_id) {
                self.selected_gates.remove(&gate_id);
                false
            } else {
                self.selected_gates.insert(gate_id);
                true
            }
        } else {
            // Clear other selections, select only this gate
            self.selected_gates.clear();
            self.selected_gates.insert(gate_id);
            self.last_selected_gate = Some(gate_id);
            true
        }
    }

    /// Clear all gate selections.
    pub fn clear_selection(&mut self) {
        self.selected_gates.clear();
        self.last_selected_gate = None;
    }

    /// Check if a gate is selected.
    pub fn is_gate_selected(&self, gate_index: usize) -> bool {
        self.selected_gates.contains(&GateId { index: gate_index })
    }

    /// Open context menu at a position for a gate or measurement.
    pub fn open_context_menu(
        &mut self,
        position: egui::Pos2,
        item_index: usize,
        is_measurement: bool,
    ) {
        let item_type = if is_measurement {
            ContextMenuType::Measurement
        } else {
            ContextMenuType::Gate
        };
        self.context_menu = Some(ContextMenuState::new(position, item_index, item_type));
        // Ensure this gate is selected (only for gates)
        if !is_measurement {
            self.toggle_gate_selection(item_index, false);
        }
    }

    /// Close the context menu.
    pub fn close_context_menu(&mut self) {
        self.context_menu = None;
    }

    /// Copy a gate to the clipboard.
    pub fn copy_gate(&mut self, gate: &PlacedGate) {
        let all_qubits: Vec<usize> = gate.all_qubits();
        self.clipboard = Some(ClipboardContent::Single {
            gate: gate.clone(),
            original_column: gate.column,
            original_qubits: all_qubits,
        });
    }

    /// Check if clipboard has content.
    pub fn has_clipboard_content(&self) -> bool {
        self.clipboard.is_some()
    }

    /// Start editing a gate's parameters.
    pub fn start_editing_gate(&mut self, gate_index: usize) {
        self.editing_gate = Some(gate_index);
        self.close_context_menu();
    }

    /// Stop editing a gate.
    pub fn stop_editing_gate(&mut self) {
        self.editing_gate = None;
    }

    /// Status text describing the current editor state.
    pub fn status_text(&self, gate: Option<&GateType>) -> String {
        if let Some(ref pending) = self.multi_qubit_pending {
            let done = pending.selected_qubits.len();
            let total = pending.gate.num_qubits();
            let controls = pending.gate.num_controls();
            if done < controls {
                format!("Select control qubit ({}/{})", done + 1, total)
            } else {
                format!("Select target qubit ({}/{})", done + 1, total)
            }
        } else if let Some(gate) = gate {
            format!("Click wire to place {}", gate.display_name())
        } else {
            "Ready".to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_qubit_placement() {
        let mut state = EditorState::default();
        let target = DropTarget {
            qubit_idx: 0,
            column: 0,
        };
        let result = state.try_place(&GateType::H, target);
        assert!(result.is_some());
        let placement = result.unwrap();
        assert_eq!(placement.gate, GateType::H);
        assert_eq!(placement.target_qubits, vec![0]);
        assert!(placement.control_qubits.is_empty());
        assert_eq!(placement.column, 0);
    }

    #[test]
    fn test_cnot_two_click_placement() {
        let mut state = EditorState::default();

        let target1 = DropTarget {
            qubit_idx: 0,
            column: 1,
        };
        let result1 = state.try_place(&GateType::Cnot, target1);
        assert!(result1.is_none());
        assert!(state.is_awaiting_more_qubits());

        let target2 = DropTarget {
            qubit_idx: 1,
            column: 1,
        };
        let result2 = state.try_place(&GateType::Cnot, target2);
        assert!(result2.is_some());
        let placement = result2.unwrap();
        assert_eq!(placement.gate, GateType::Cnot);
        assert_eq!(placement.control_qubits, vec![0]);
        assert_eq!(placement.target_qubits, vec![1]);
        assert_eq!(placement.column, 1);
        assert!(!state.is_awaiting_more_qubits());
    }

    #[test]
    fn test_swap_two_click_placement() {
        let mut state = EditorState::default();

        let target1 = DropTarget {
            qubit_idx: 0,
            column: 2,
        };
        assert!(state.try_place(&GateType::Swap, target1).is_none());

        let target2 = DropTarget {
            qubit_idx: 1,
            column: 2,
        };
        let result = state.try_place(&GateType::Swap, target2);
        assert!(result.is_some());

        let placement = result.unwrap();
        assert_eq!(placement.target_qubits, vec![0, 1]);
        assert!(placement.control_qubits.is_empty());
    }

    #[test]
    fn test_toffoli_three_click_placement() {
        let mut state = EditorState::default();

        let t1 = DropTarget {
            qubit_idx: 0,
            column: 0,
        };
        assert!(state.try_place(&GateType::Toffoli, t1).is_none());

        let t2 = DropTarget {
            qubit_idx: 1,
            column: 0,
        };
        assert!(state.try_place(&GateType::Toffoli, t2).is_none());

        let t3 = DropTarget {
            qubit_idx: 2,
            column: 0,
        };
        let result = state.try_place(&GateType::Toffoli, t3);
        assert!(result.is_some());

        let placement = result.unwrap();
        assert_eq!(placement.control_qubits, vec![0, 1]);
        assert_eq!(placement.target_qubits, vec![2]);
    }

    #[test]
    fn test_same_qubit_rejected() {
        let mut state = EditorState::default();

        let target1 = DropTarget {
            qubit_idx: 0,
            column: 1,
        };
        state.try_place(&GateType::Cnot, target1);

        let target2 = DropTarget {
            qubit_idx: 0,
            column: 1,
        };
        let result = state.try_place(&GateType::Cnot, target2);
        assert!(result.is_none());
        assert!(state.is_awaiting_more_qubits());
    }

    #[test]
    fn test_cancel_pending() {
        let mut state = EditorState::default();

        let target = DropTarget {
            qubit_idx: 0,
            column: 1,
        };
        state.try_place(&GateType::Cnot, target);
        assert!(state.is_awaiting_more_qubits());

        state.cancel_pending();
        assert!(!state.is_awaiting_more_qubits());
    }

    #[test]
    fn test_gate_change_resets_pending() {
        let mut state = EditorState::default();

        let target = DropTarget {
            qubit_idx: 0,
            column: 1,
        };
        state.try_place(&GateType::Cnot, target);
        assert!(state.is_awaiting_more_qubits());

        // Switching to a different gate should reset pending
        let target2 = DropTarget {
            qubit_idx: 1,
            column: 2,
        };
        state.try_place(&GateType::Cz, target2);
        // Now pending should be for Cz, not Cnot
        let pending = state.multi_qubit_pending.as_ref().unwrap();
        assert_eq!(pending.gate, GateType::Cz);
        assert_eq!(pending.selected_qubits, vec![1]);
    }

    #[test]
    fn test_status_text() {
        let mut state = EditorState::default();

        assert_eq!(state.status_text(None), "Ready");
        assert_eq!(
            state.status_text(Some(&GateType::H)),
            "Click wire to place H"
        );

        let target = DropTarget {
            qubit_idx: 0,
            column: 0,
        };
        state.try_place(&GateType::Cnot, target);
        assert_eq!(
            state.status_text(Some(&GateType::Cnot)),
            "Select target qubit (2/2)"
        );
    }
}
