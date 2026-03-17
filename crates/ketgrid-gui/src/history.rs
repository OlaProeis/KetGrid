//! Operation-based undo/redo system for circuit edits.
//!
//! Tracks reversible operations (add/remove gates, edit parameters, etc.)
//! with a bounded undo stack and symmetric redo support.

use ketgrid_core::circuit::{Circuit, Measurement, PlacedGate};
use ketgrid_core::gate::GateType;

const MAX_HISTORY: usize = 100;

/// A reversible edit operation on the circuit.
///
/// Each variant stores enough data to both undo (reverse) and redo (reapply)
/// the operation. For `ReplaceCircuit`, the stored circuit is swapped in/out
/// on each undo/redo, making both directions a simple `mem::swap`.
#[derive(Debug, Clone)]
pub enum EditOperation {
    AddGate {
        index: usize,
        gate: PlacedGate,
    },
    RemoveGate {
        index: usize,
        gate: PlacedGate,
    },
    AddMeasurement {
        index: usize,
        measurement: Measurement,
    },
    RemoveMeasurement {
        index: usize,
        measurement: Measurement,
    },
    EditParam {
        gate_index: usize,
        old_gate_type: GateType,
        new_gate_type: GateType,
    },
    AddQubit,
    RemoveQubit {
        qubit_id: usize,
        wire: ketgrid_core::wire::QubitWire,
    },
    /// Wholesale circuit replacement (New Circuit, Open File).
    /// The stored circuit alternates between "old" and "current" on each swap.
    ReplaceCircuit {
        old_circuit: Circuit,
    },
}

impl EditOperation {
    /// Apply the reverse of this operation to the circuit.
    fn undo(&mut self, circuit: &mut Circuit) {
        match self {
            Self::AddGate { index, .. } => {
                circuit.gates.remove(*index);
            }
            Self::RemoveGate { index, gate } => {
                circuit.gates.insert(*index, gate.clone());
            }
            Self::AddMeasurement { index, .. } => {
                circuit.measurements.remove(*index);
            }
            Self::RemoveMeasurement { index, measurement } => {
                circuit.measurements.insert(*index, measurement.clone());
            }
            Self::EditParam {
                gate_index,
                old_gate_type,
                ..
            } => {
                circuit.gates[*gate_index].gate = old_gate_type.clone();
            }
            Self::AddQubit => {
                circuit.qubits.pop();
            }
            Self::RemoveQubit { qubit_id, wire } => {
                // Insert the qubit back at its original position
                circuit.qubits.insert(*qubit_id, wire.clone());
                // Update qubit IDs
                for (new_id, qubit) in circuit.qubits.iter_mut().enumerate() {
                    qubit.id = new_id;
                }
                // Gate/measurement indices were adjusted when removed, so they need
                // to be shifted back up for qubits that were above the removed one
                for gate in &mut circuit.gates {
                    gate.target_qubits = gate
                        .target_qubits
                        .iter()
                        .map(|&idx| if idx >= *qubit_id { idx + 1 } else { idx })
                        .collect();
                    gate.control_qubits = gate
                        .control_qubits
                        .iter()
                        .map(|&idx| if idx >= *qubit_id { idx + 1 } else { idx })
                        .collect();
                }
                for measurement in &mut circuit.measurements {
                    if measurement.qubit_id >= *qubit_id {
                        measurement.qubit_id += 1;
                    }
                }
            }
            Self::ReplaceCircuit { old_circuit } => {
                std::mem::swap(circuit, old_circuit);
            }
        }
    }

    /// Reapply this operation to the circuit.
    fn redo(&mut self, circuit: &mut Circuit) {
        match self {
            Self::AddGate { index, gate } => {
                circuit.gates.insert(*index, gate.clone());
            }
            Self::RemoveGate { index, .. } => {
                circuit.gates.remove(*index);
            }
            Self::AddMeasurement { index, measurement } => {
                circuit.measurements.insert(*index, measurement.clone());
            }
            Self::RemoveMeasurement { index, .. } => {
                circuit.measurements.remove(*index);
            }
            Self::EditParam {
                gate_index,
                new_gate_type,
                ..
            } => {
                circuit.gates[*gate_index].gate = new_gate_type.clone();
            }
            Self::AddQubit => {
                circuit.add_qubit();
            }
            Self::RemoveQubit { qubit_id, .. } => {
                // Re-remove the qubit (ignore error since we know it was removable before)
                let _ = circuit.remove_qubit(*qubit_id);
            }
            Self::ReplaceCircuit { old_circuit } => {
                std::mem::swap(circuit, old_circuit);
            }
        }
    }
}

/// Bounded undo/redo stack for circuit edit operations.
///
/// New operations are pushed onto the undo stack (clearing the redo stack).
/// `undo()` pops the last operation, reverses it, and moves it to the redo stack.
/// `redo()` pops from redo, reapplies, and moves back to undo.
/// The undo stack is capped at [`MAX_HISTORY`] entries.
pub struct EditHistory {
    undo_stack: Vec<EditOperation>,
    redo_stack: Vec<EditOperation>,
}

impl Default for EditHistory {
    fn default() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl EditHistory {
    /// Record a new edit operation. Clears the redo stack.
    pub fn push(&mut self, op: EditOperation) {
        self.redo_stack.clear();
        self.undo_stack.push(op);
        if self.undo_stack.len() > MAX_HISTORY {
            self.undo_stack.remove(0);
        }
    }

    /// Undo the most recent operation. Returns `true` if an operation was undone.
    pub fn undo(&mut self, circuit: &mut Circuit) -> bool {
        if let Some(mut op) = self.undo_stack.pop() {
            op.undo(circuit);
            self.redo_stack.push(op);
            true
        } else {
            false
        }
    }

    /// Redo the most recently undone operation. Returns `true` if an operation was redone.
    pub fn redo(&mut self, circuit: &mut Circuit) -> bool {
        if let Some(mut op) = self.redo_stack.pop() {
            op.redo(circuit);
            self.undo_stack.push(op);
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Clear all history (used when loading a file from scratch, etc.).
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ketgrid_core::gate::GateType;

    #[test]
    fn test_undo_redo_add_gate() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        let idx = circuit.gates.len() - 1;
        history.push(EditOperation::AddGate {
            index: idx,
            gate: circuit.gates[idx].clone(),
        });

        assert_eq!(circuit.gates.len(), 1);
        assert!(history.can_undo());
        assert!(!history.can_redo());

        history.undo(&mut circuit);
        assert_eq!(circuit.gates.len(), 0);
        assert!(!history.can_undo());
        assert!(history.can_redo());

        history.redo(&mut circuit);
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::H);
    }

    #[test]
    fn test_undo_remove_gate() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        let removed = circuit.remove_gate(0).unwrap();
        history.push(EditOperation::RemoveGate {
            index: 0,
            gate: removed,
        });

        assert_eq!(circuit.gates.len(), 0);

        history.undo(&mut circuit);
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::X);

        history.redo(&mut circuit);
        assert_eq!(circuit.gates.len(), 0);
    }

    #[test]
    fn test_undo_edit_param() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit
            .add_gate(GateType::Rx(1.0), vec![0], vec![], 0)
            .unwrap();

        let old_type = circuit.gates[0].gate.clone();
        let new_type = GateType::Rx(2.0);
        circuit
            .update_gate_parameters(0, new_type.clone())
            .unwrap();
        history.push(EditOperation::EditParam {
            gate_index: 0,
            old_gate_type: old_type.clone(),
            new_gate_type: new_type,
        });

        assert_eq!(circuit.gates[0].gate, GateType::Rx(2.0));

        history.undo(&mut circuit);
        assert_eq!(circuit.gates[0].gate, GateType::Rx(1.0));

        history.redo(&mut circuit);
        assert_eq!(circuit.gates[0].gate, GateType::Rx(2.0));
    }

    #[test]
    fn test_undo_add_measurement() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_measurement(0, 1).unwrap();
        let idx = circuit.measurements.len() - 1;
        history.push(EditOperation::AddMeasurement {
            index: idx,
            measurement: circuit.measurements[idx].clone(),
        });

        assert_eq!(circuit.measurements.len(), 1);

        history.undo(&mut circuit);
        assert_eq!(circuit.measurements.len(), 0);

        history.redo(&mut circuit);
        assert_eq!(circuit.measurements.len(), 1);
        assert_eq!(circuit.measurements[0].qubit_id, 0);
    }

    #[test]
    fn test_undo_remove_measurement() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_measurement(1, 2).unwrap();
        let removed = circuit.remove_measurement(0).unwrap();
        history.push(EditOperation::RemoveMeasurement {
            index: 0,
            measurement: removed,
        });

        assert_eq!(circuit.measurements.len(), 0);

        history.undo(&mut circuit);
        assert_eq!(circuit.measurements.len(), 1);
        assert_eq!(circuit.measurements[0].qubit_id, 1);

        history.redo(&mut circuit);
        assert_eq!(circuit.measurements.len(), 0);
    }

    #[test]
    fn test_undo_add_qubit() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_qubit();
        history.push(EditOperation::AddQubit);

        assert_eq!(circuit.num_qubits(), 3);

        history.undo(&mut circuit);
        assert_eq!(circuit.num_qubits(), 2);

        history.redo(&mut circuit);
        assert_eq!(circuit.num_qubits(), 3);
    }

    #[test]
    fn test_undo_replace_circuit() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        let mut history = EditHistory::default();

        let old = std::mem::replace(&mut circuit, Circuit::new(3));
        history.push(EditOperation::ReplaceCircuit { old_circuit: old });

        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.gates.len(), 0);

        history.undo(&mut circuit);
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.gates.len(), 1);

        history.redo(&mut circuit);
        assert_eq!(circuit.num_qubits(), 3);
        assert_eq!(circuit.gates.len(), 0);
    }

    #[test]
    fn test_bell_state_undo_redo() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        // H on q0
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        history.push(EditOperation::AddGate {
            index: 0,
            gate: circuit.gates[0].clone(),
        });

        // CNOT(q0→q1)
        circuit
            .add_gate(GateType::Cnot, vec![1], vec![0], 1)
            .unwrap();
        history.push(EditOperation::AddGate {
            index: 1,
            gate: circuit.gates[1].clone(),
        });

        assert_eq!(circuit.gates.len(), 2);

        // Ctrl+Z twice → empty circuit
        assert!(history.undo(&mut circuit));
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::H);

        assert!(history.undo(&mut circuit));
        assert_eq!(circuit.gates.len(), 0);

        // Ctrl+Y → restore H
        assert!(history.redo(&mut circuit));
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::H);

        // Ctrl+Y → restore CNOT
        assert!(history.redo(&mut circuit));
        assert_eq!(circuit.gates.len(), 2);
        assert_eq!(circuit.gates[1].gate, GateType::Cnot);
    }

    #[test]
    fn test_new_operation_clears_redo() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        history.push(EditOperation::AddGate {
            index: 0,
            gate: circuit.gates[0].clone(),
        });

        history.undo(&mut circuit);
        assert!(history.can_redo());

        // New operation should clear redo stack
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        history.push(EditOperation::AddGate {
            index: 0,
            gate: circuit.gates[0].clone(),
        });
        assert!(!history.can_redo());
    }

    #[test]
    fn test_stack_limit() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        for i in 0..150 {
            circuit.add_gate(GateType::H, vec![0], vec![], i).unwrap();
            let idx = circuit.gates.len() - 1;
            history.push(EditOperation::AddGate {
                index: idx,
                gate: circuit.gates[idx].clone(),
            });
        }

        assert_eq!(history.undo_stack.len(), MAX_HISTORY);
    }

    #[test]
    fn test_undo_on_empty_returns_false() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();
        assert!(!history.undo(&mut circuit));
        assert!(!history.redo(&mut circuit));
    }

    #[test]
    fn test_clear() {
        let mut circuit = Circuit::new(2);
        let mut history = EditHistory::default();

        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        history.push(EditOperation::AddGate {
            index: 0,
            gate: circuit.gates[0].clone(),
        });

        history.clear();
        assert!(!history.can_undo());
        assert!(!history.can_redo());
    }
}
