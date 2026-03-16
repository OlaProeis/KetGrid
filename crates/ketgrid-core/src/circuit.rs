//! Circuit data model and serialization.

use crate::gate::GateType;
use crate::wire::QubitWire;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error type for circuit validation and operations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitError {
    /// Gate target qubit index is out of bounds.
    InvalidQubitIndex { index: usize, num_qubits: usize },
    /// Control qubit index is out of bounds.
    InvalidControlIndex { index: usize, num_qubits: usize },
    /// Gate target and control qubits overlap.
    OverlappingTargetsAndControls,
    /// Gate column is invalid (too large or conflicts with existing gate).
    InvalidColumn { column: usize },
    /// Number of targets doesn't match gate type requirements.
    InvalidTargetCount { expected: usize, actual: usize },
    /// Number of controls doesn't match gate type requirements.
    InvalidControlCount { expected: usize, actual: usize },
    /// Missing parameters for parameterized gate.
    MissingParameters { gate: GateType },
    /// Cannot remove qubit that has gates targeting it.
    QubitInUse { qubit_id: usize },
    /// Invalid reordering permutation.
    InvalidPermutation { message: String },
    /// Generic error with message.
    Message(String),
}

impl fmt::Display for CircuitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CircuitError::InvalidQubitIndex { index, num_qubits } => {
                write!(f, "Qubit index {} out of bounds (circuit has {} qubits)", index, num_qubits)
            }
            CircuitError::InvalidControlIndex { index, num_qubits } => {
                write!(f, "Control qubit index {} out of bounds (circuit has {} qubits)", index, num_qubits)
            }
            CircuitError::OverlappingTargetsAndControls => {
                write!(f, "Gate target and control qubits overlap")
            }
            CircuitError::InvalidColumn { column } => {
                write!(f, "Invalid column position: {}", column)
            }
            CircuitError::InvalidTargetCount { expected, actual } => {
                write!(f, "Expected {} target qubits, got {}", expected, actual)
            }
            CircuitError::InvalidControlCount { expected, actual } => {
                write!(f, "Expected {} control qubits, got {}", expected, actual)
            }
            CircuitError::MissingParameters { gate } => {
                write!(f, "Missing parameters for gate: {:?}", gate)
            }
            CircuitError::QubitInUse { qubit_id } => {
                write!(f, "Cannot remove qubit {}: it has gates targeting it", qubit_id)
            }
            CircuitError::InvalidPermutation { message } => {
                write!(f, "Invalid permutation: {}", message)
            }
            CircuitError::Message(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for CircuitError {}

/// A quantum circuit with qubit wires and placed gates.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Circuit {
    /// Named qubit wires in the circuit.
    pub qubits: Vec<QubitWire>,
    /// Gates positioned on the circuit.
    pub gates: Vec<PlacedGate>,
    /// Measurements applied to qubits.
    pub measurements: Vec<Measurement>,
}

impl Circuit {
    /// Creates a new empty circuit with the specified number of qubits.
    /// Qubits are labeled |q₀⟩, |q₁⟩, etc.
    pub fn new(num_qubits: usize) -> Self {
        let qubits: Vec<QubitWire> = (0..num_qubits)
            .map(QubitWire::with_default_label)
            .collect();

        Self {
            qubits,
            gates: Vec::new(),
            measurements: Vec::new(),
        }
    }

    /// Creates a new circuit with custom qubit labels.
    pub fn with_labels(labels: Vec<String>) -> Self {
        let qubits: Vec<QubitWire> = labels
            .into_iter()
            .enumerate()
            .map(|(id, label)| QubitWire::new(id, label))
            .collect();

        Self {
            qubits,
            gates: Vec::new(),
            measurements: Vec::new(),
        }
    }

    /// Returns the number of qubits in the circuit.
    pub fn num_qubits(&self) -> usize {
        self.qubits.len()
    }

    /// Adds a new qubit wire to the circuit.
    pub fn add_qubit(&mut self) {
        let id = self.qubits.len();
        self.qubits.push(QubitWire::with_default_label(id));
    }

    /// Removes a qubit wire from the circuit.
    /// Returns error if the qubit has gates targeting it.
    /// Updates all gate indices to account for the removed qubit.
    pub fn remove_qubit(&mut self, qubit_id: usize) -> Result<(), CircuitError> {
        // Check if qubit exists
        if qubit_id >= self.num_qubits() {
            return Err(CircuitError::InvalidQubitIndex {
                index: qubit_id,
                num_qubits: self.num_qubits(),
            });
        }

        // Check if qubit has gates targeting it
        let has_gates = self.gates.iter().any(|g| {
            g.target_qubits.contains(&qubit_id) || g.control_qubits.contains(&qubit_id)
        });

        if has_gates {
            return Err(CircuitError::QubitInUse { qubit_id });
        }

        // Check if qubit has measurements
        let has_measurements = self.measurements.iter().any(|m| m.qubit_id == qubit_id);

        if has_measurements {
            return Err(CircuitError::QubitInUse { qubit_id });
        }

        // Remove the qubit
        self.qubits.remove(qubit_id);

        // Update remaining qubit IDs
        for (new_id, qubit) in self.qubits.iter_mut().enumerate() {
            qubit.id = new_id;
        }

        // Update gate indices: any index > qubit_id needs to be decremented
        for gate in &mut self.gates {
            gate.target_qubits = gate
                .target_qubits
                .iter()
                .map(|&idx| if idx > qubit_id { idx - 1 } else { idx })
                .collect();
            gate.control_qubits = gate
                .control_qubits
                .iter()
                .map(|&idx| if idx > qubit_id { idx - 1 } else { idx })
                .collect();
        }

        // Update measurement indices
        for measurement in &mut self.measurements {
            if measurement.qubit_id > qubit_id {
                measurement.qubit_id -= 1;
            }
        }

        Ok(())
    }

    /// Renames a qubit wire.
    pub fn rename_qubit(&mut self, qubit_id: usize, new_label: impl Into<String>) -> Result<(), CircuitError> {
        if qubit_id >= self.num_qubits() {
            return Err(CircuitError::InvalidQubitIndex {
                index: qubit_id,
                num_qubits: self.num_qubits(),
            });
        }

        self.qubits[qubit_id].label = new_label.into();
        Ok(())
    }

    /// Reorders qubit wires according to the given permutation.
    /// The permutation vector should contain each qubit index exactly once.
    /// For example, [2, 0, 1] means: old qubit 2 becomes new qubit 0,
    /// old qubit 0 becomes new qubit 1, old qubit 1 becomes new qubit 2.
    pub fn reorder_qubits(&mut self, permutation: &[usize]) -> Result<(), CircuitError> {
        let n = self.num_qubits();

        // Validate permutation length
        if permutation.len() != n {
            return Err(CircuitError::InvalidPermutation {
                message: format!("Expected {} indices, got {}", n, permutation.len()),
            });
        }

        // Validate that permutation contains each index exactly once
        let mut seen = vec![false; n];
        for &idx in permutation {
            if idx >= n {
                return Err(CircuitError::InvalidPermutation {
                    message: format!("Index {} out of bounds (max: {})", idx, n - 1),
                });
            }
            if seen[idx] {
                return Err(CircuitError::InvalidPermutation {
                    message: format!("Index {} appears multiple times", idx),
                });
            }
            seen[idx] = true;
        }

        // Create inverse mapping: old_index -> new_index
        let mut old_to_new: Vec<usize> = vec![0; n];
        for (new_idx, &old_idx) in permutation.iter().enumerate() {
            old_to_new[old_idx] = new_idx;
        }

        // Reorder qubits
        let mut new_qubits: Vec<QubitWire> = Vec::with_capacity(n);
        for &old_idx in permutation {
            let mut qubit = self.qubits[old_idx].clone();
            qubit.id = old_to_new[old_idx];
            new_qubits.push(qubit);
        }
        self.qubits = new_qubits;

        // Update qubit IDs to match new positions
        for (id, qubit) in self.qubits.iter_mut().enumerate() {
            qubit.id = id;
        }

        // Update gate indices
        for gate in &mut self.gates {
            gate.target_qubits = gate
                .target_qubits
                .iter()
                .map(|&idx| old_to_new[idx])
                .collect();
            gate.control_qubits = gate
                .control_qubits
                .iter()
                .map(|&idx| old_to_new[idx])
                .collect();
        }

        // Update measurement indices
        for measurement in &mut self.measurements {
            measurement.qubit_id = old_to_new[measurement.qubit_id];
        }

        Ok(())
    }

    /// Adds a gate to the circuit at the specified position.
    /// Validates that all qubit indices are within bounds.
    pub fn add_gate(
        &mut self,
        gate: GateType,
        target_qubits: Vec<usize>,
        control_qubits: Vec<usize>,
        column: usize,
    ) -> Result<(), CircuitError> {
        self.validate_gate(&gate, &target_qubits, &control_qubits)?;

        let placed_gate = PlacedGate {
            gate,
            target_qubits,
            control_qubits,
            column,
            parameters: Vec::new(), // Extracted from gate type if parameterized
        };

        self.gates.push(placed_gate);
        Ok(())
    }

    /// Adds a measurement to a qubit.
    pub fn add_measurement(&mut self, qubit_id: usize, column: usize) -> Result<(), CircuitError> {
        if qubit_id >= self.num_qubits() {
            return Err(CircuitError::InvalidQubitIndex {
                index: qubit_id,
                num_qubits: self.num_qubits(),
            });
        }

        let measurement = Measurement { qubit_id, column };
        self.measurements.push(measurement);
        Ok(())
    }

    /// Removes a measurement by its index in the measurements vector.
    /// Returns the removed measurement if successful.
    pub fn remove_measurement(&mut self, measurement_index: usize) -> Option<Measurement> {
        if measurement_index < self.measurements.len() {
            Some(self.measurements.remove(measurement_index))
        } else {
            None
        }
    }

    /// Finds a measurement at the specified column and qubit.
    /// Returns the index of the measurement in the measurements vector if found.
    pub fn find_measurement_at(&self, column: usize, qubit_idx: usize) -> Option<usize> {
        self.measurements.iter().enumerate().find_map(|(idx, meas)| {
            if meas.column == column && meas.qubit_id == qubit_idx {
                Some(idx)
            } else {
                None
            }
        })
    }

    /// Validates a gate placement.
    fn validate_gate(
        &self,
        gate: &GateType,
        targets: &[usize],
        controls: &[usize],
    ) -> Result<(), CircuitError> {
        let num_qubits = self.num_qubits();

        // Validate target qubits
        for &idx in targets {
            if idx >= num_qubits {
                return Err(CircuitError::InvalidQubitIndex {
                    index: idx,
                    num_qubits,
                });
            }
        }

        // Validate control qubits
        for &idx in controls {
            if idx >= num_qubits {
                return Err(CircuitError::InvalidControlIndex {
                    index: idx,
                    num_qubits,
                });
            }
        }

        // Check for overlap between targets and controls
        for target in targets {
            if controls.contains(target) {
                return Err(CircuitError::OverlappingTargetsAndControls);
            }
        }

        // Validate target count
        let expected_targets = gate.num_qubits() - gate.num_controls();
        if targets.len() != expected_targets {
            return Err(CircuitError::InvalidTargetCount {
                expected: expected_targets,
                actual: targets.len(),
            });
        }

        // Validate control count
        let expected_controls = gate.num_controls();
        if controls.len() != expected_controls {
            return Err(CircuitError::InvalidControlCount {
                expected: expected_controls,
                actual: controls.len(),
            });
        }

        Ok(())
    }

    /// Returns the maximum column index used in the circuit.
    pub fn max_column(&self) -> usize {
        let gate_max = self.gates.iter().map(|g| g.column).max().unwrap_or(0);
        let meas_max = self.measurements.iter().map(|m| m.column).max().unwrap_or(0);
        gate_max.max(meas_max)
    }

    /// Returns gates sorted by column (left-to-right execution order).
    pub fn gates_by_column(&self) -> Vec<&PlacedGate> {
        let mut sorted: Vec<&PlacedGate> = self.gates.iter().collect();
        sorted.sort_by_key(|g| g.column);
        sorted
    }

    /// Removes a gate by its index in the gates vector.
    /// Returns the removed gate if successful.
    pub fn remove_gate(&mut self, gate_index: usize) -> Option<PlacedGate> {
        if gate_index < self.gates.len() {
            Some(self.gates.remove(gate_index))
        } else {
            None
        }
    }

    /// Updates a gate's parameters.
    /// Only works for parameterized gates (Rx, Ry, Rz).
    /// Returns true if the gate was updated.
    pub fn update_gate_parameters(
        &mut self,
        gate_index: usize,
        new_gate: GateType,
    ) -> Result<(), CircuitError> {
        if gate_index >= self.gates.len() {
            return Err(CircuitError::InvalidQubitIndex {
                index: gate_index,
                num_qubits: self.gates.len(),
            });
        }

        let gate = &mut self.gates[gate_index];

        // Ensure the new gate type is compatible (same number of qubits)
        if gate.gate.num_qubits() != new_gate.num_qubits() {
            return Err(CircuitError::InvalidTargetCount {
                expected: gate.gate.num_qubits(),
                actual: new_gate.num_qubits(),
            });
        }

        // Ensure the new gate type has the same control structure
        if gate.gate.num_controls() != new_gate.num_controls() {
            return Err(CircuitError::InvalidControlCount {
                expected: gate.gate.num_controls(),
                actual: new_gate.num_controls(),
            });
        }

        gate.gate = new_gate;
        Ok(())
    }

    /// Finds a gate at the specified column and qubit.
    /// Returns the index of the gate in the gates vector if found.
    pub fn find_gate_at(&self, column: usize, qubit_idx: usize) -> Option<usize> {
        self.gates.iter().enumerate().find_map(|(idx, gate)| {
            if gate.column == column && gate.all_qubits().contains(&qubit_idx) {
                Some(idx)
            } else {
                None
            }
        })
    }
}

/// A gate placed at a specific position in the circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PlacedGate {
    /// The type of gate (H, X, CNOT, etc.).
    pub gate: GateType,
    /// Which wires this gate spans (target qubits).
    pub target_qubits: Vec<usize>,
    /// Control dots (for CNOT, Toffoli, etc.).
    pub control_qubits: Vec<usize>,
    /// Time step position (left-to-right).
    pub column: usize,
    /// For parameterized gates (Rx, Ry, Rz) - stored for serialization convenience.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameters: Vec<f64>,
}

impl PlacedGate {
    /// Returns true if this gate is controlled (has control qubits).
    pub fn is_controlled(&self) -> bool {
        !self.control_qubits.is_empty()
    }

    /// Returns all qubits involved in this gate (controls + targets).
    pub fn all_qubits(&self) -> Vec<usize> {
        let mut all = self.control_qubits.clone();
        all.extend(&self.target_qubits);
        all
    }
}

/// A measurement operation on a qubit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measurement {
    /// Which qubit is being measured.
    pub qubit_id: usize,
    /// Time step position.
    pub column: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_creation() {
        let circuit = Circuit::new(2);
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.qubits.len(), 2);
        assert_eq!(circuit.gates.len(), 0);
        assert_eq!(circuit.measurements.len(), 0);
        assert_eq!(circuit.qubits[0].label, "|q0⟩");
        assert_eq!(circuit.qubits[1].label, "|q1⟩");
    }

    #[test]
    fn test_circuit_with_custom_labels() {
        let circuit = Circuit::with_labels(vec!["Alice".to_string(), "Bob".to_string()]);
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.qubits[0].label, "Alice");
        assert_eq!(circuit.qubits[1].label, "Bob");
    }

    #[test]
    fn test_add_h_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::H);
        assert_eq!(circuit.gates[0].target_qubits, vec![0]);
        assert_eq!(circuit.gates[0].column, 0);
    }

    #[test]
    fn test_add_cnot_gate() {
        let mut circuit = Circuit::new(2);
        // CNOT with control on q0, target on q1
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::Cnot);
        assert_eq!(circuit.gates[0].control_qubits, vec![0]);
        assert_eq!(circuit.gates[0].target_qubits, vec![1]);
        assert_eq!(circuit.gates[0].column, 1);
    }

    #[test]
    fn test_invalid_qubit_index() {
        let mut circuit = Circuit::new(2);
        let result = circuit.add_gate(GateType::H, vec![5], vec![], 0);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CircuitError::InvalidQubitIndex {
                index: 5,
                num_qubits: 2,
            }
        );
    }

    #[test]
    fn test_invalid_control_index() {
        let mut circuit = Circuit::new(2);
        let result = circuit.add_gate(GateType::Cnot, vec![0], vec![5], 0);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CircuitError::InvalidControlIndex {
                index: 5,
                num_qubits: 2,
            }
        );
    }

    #[test]
    fn test_overlapping_targets_and_controls() {
        let mut circuit = Circuit::new(2);
        let result = circuit.add_gate(GateType::Cnot, vec![0], vec![0], 0);
        
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), CircuitError::OverlappingTargetsAndControls);
    }

    #[test]
    fn test_invalid_target_count() {
        let mut circuit = Circuit::new(2);
        // H gate expects 1 target
        let result = circuit.add_gate(GateType::H, vec![0, 1], vec![], 0);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CircuitError::InvalidTargetCount {
                expected: 1,
                actual: 2,
            }
        );
    }

    #[test]
    fn test_invalid_control_count() {
        let mut circuit = Circuit::new(3);
        // CNOT expects 1 control
        let result = circuit.add_gate(GateType::Cnot, vec![2], vec![0, 1], 0);
        
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err(),
            CircuitError::InvalidControlCount {
                expected: 1,
                actual: 2,
            }
        );
    }

    #[test]
    fn test_add_measurement() {
        let mut circuit = Circuit::new(2);
        circuit.add_measurement(0, 2).unwrap();
        
        assert_eq!(circuit.measurements.len(), 1);
        assert_eq!(circuit.measurements[0].qubit_id, 0);
        assert_eq!(circuit.measurements[0].column, 2);
    }

    #[test]
    fn test_invalid_measurement_qubit() {
        let mut circuit = Circuit::new(2);
        let result = circuit.add_measurement(5, 0);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_serialize_deserialize_roundtrip() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_measurement(0, 2).unwrap();

        let serialized = serde_json::to_string(&circuit).unwrap();
        let deserialized: Circuit = serde_json::from_str(&serialized).unwrap();

        assert_eq!(circuit, deserialized);
    }

    #[test]
    fn test_gates_by_column() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::X, vec![0], vec![], 2).unwrap();
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Z, vec![1], vec![], 1).unwrap();

        let sorted = circuit.gates_by_column();
        assert_eq!(sorted.len(), 3);
        assert_eq!(sorted[0].column, 0);
        assert_eq!(sorted[1].column, 1);
        assert_eq!(sorted[2].column, 2);
    }

    #[test]
    fn test_max_column() {
        let mut circuit = Circuit::new(2);
        assert_eq!(circuit.max_column(), 0);

        circuit.add_gate(GateType::H, vec![0], vec![], 5).unwrap();
        assert_eq!(circuit.max_column(), 5);

        circuit.add_measurement(1, 3).unwrap();
        assert_eq!(circuit.max_column(), 5); // Still 5 from the gate

        circuit.add_measurement(0, 10).unwrap();
        assert_eq!(circuit.max_column(), 10);
    }

    #[test]
    fn test_toffoli_gate() {
        let mut circuit = Circuit::new(3);
        // Toffoli: 2 controls, 1 target
        circuit.add_gate(GateType::Toffoli, vec![2], vec![0, 1], 0).unwrap();
        
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].control_qubits, vec![0, 1]);
        assert_eq!(circuit.gates[0].target_qubits, vec![2]);
    }

    #[test]
    fn test_parameterized_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Rx(std::f64::consts::PI), vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Ry(std::f64::consts::FRAC_PI_2), vec![1], vec![], 1).unwrap();
        
        assert_eq!(circuit.gates.len(), 2);
        assert!(circuit.gates[0].gate.is_parameterized());
        assert_eq!(circuit.gates[0].gate.parameters(), vec![std::f64::consts::PI]);
    }

    #[test]
    fn test_swap_gate() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Swap, vec![0, 1], vec![], 0).unwrap();
        
        assert_eq!(circuit.gates.len(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::Swap);
        assert_eq!(circuit.gates[0].target_qubits, vec![0, 1]);
    }

    #[test]
    fn test_barrier_and_identity() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::Barrier, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Identity, vec![1], vec![], 1).unwrap();
        
        assert_eq!(circuit.gates.len(), 2);
        assert_eq!(circuit.gates[0].gate, GateType::Barrier);
        assert_eq!(circuit.gates[1].gate, GateType::Identity);
    }

    #[test]
    fn test_placed_gate_all_qubits() {
        let gate = PlacedGate {
            gate: GateType::Cnot,
            target_qubits: vec![1],
            control_qubits: vec![0],
            column: 0,
            parameters: vec![],
        };
        
        let all = gate.all_qubits();
        assert_eq!(all, vec![0, 1]);
        assert!(gate.is_controlled());
    }

    #[test]
    fn test_circuit_error_display() {
        let err = CircuitError::InvalidQubitIndex { index: 5, num_qubits: 2 };
        assert_eq!(err.to_string(), "Qubit index 5 out of bounds (circuit has 2 qubits)");

        let err = CircuitError::OverlappingTargetsAndControls;
        assert_eq!(err.to_string(), "Gate target and control qubits overlap");
    }

    // =========================================================================
    // Qubit Management Tests
    // =========================================================================

    #[test]
    fn test_remove_qubit_no_gates() {
        let mut circuit = Circuit::new(3);
        // Add qubits: q0, q1, q2
        assert_eq!(circuit.num_qubits(), 3);

        // Remove q0
        circuit.remove_qubit(0).unwrap();
        assert_eq!(circuit.num_qubits(), 2);
        // Remaining qubits should have IDs 0, 1 (renumbered)
        assert_eq!(circuit.qubits[0].id, 0);
        assert_eq!(circuit.qubits[1].id, 1);
    }

    #[test]
    fn test_remove_qubit_with_gate_fails() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

        // Should fail because qubit 0 has a gate targeting it
        let result = circuit.remove_qubit(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::QubitInUse { qubit_id: 0 }));
    }

    #[test]
    fn test_remove_qubit_with_control_fails() {
        let mut circuit = Circuit::new(3);
        // CNOT with control on q1, target on q2
        circuit.add_gate(GateType::Cnot, vec![2], vec![1], 0).unwrap();

        // Should fail because qubit 1 is a control
        let result = circuit.remove_qubit(1);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::QubitInUse { qubit_id: 1 }));
    }

    #[test]
    fn test_remove_qubit_updates_gate_indices() {
        let mut circuit = Circuit::new(3);
        // Add gates: H on q1, X on q2
        circuit.add_gate(GateType::H, vec![1], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![2], vec![], 1).unwrap();

        // Remove q0 (has no gates)
        circuit.remove_qubit(0).unwrap();

        // Gates should now target qubits 0 and 1 (shifted down)
        assert_eq!(circuit.gates.len(), 2);
        assert_eq!(circuit.gates[0].target_qubits, vec![0]); // was q1, now q0
        assert_eq!(circuit.gates[1].target_qubits, vec![1]); // was q2, now q1
    }

    #[test]
    fn test_remove_qubit_updates_control_indices() {
        let mut circuit = Circuit::new(4);
        // CNOT with control on q2, target on q3
        circuit.add_gate(GateType::Cnot, vec![3], vec![2], 0).unwrap();

        // Remove q0 and q1 (have no gates)
        circuit.remove_qubit(0).unwrap();
        circuit.remove_qubit(0).unwrap(); // was q1, now at index 0

        // Remaining qubits: old q2 -> new q0, old q3 -> new q1
        // CNOT control should now be q0, target should be q1
        assert_eq!(circuit.gates[0].control_qubits, vec![0]);
        assert_eq!(circuit.gates[0].target_qubits, vec![1]);
    }

    #[test]
    fn test_remove_qubit_invalid_index() {
        let mut circuit = Circuit::new(2);
        let result = circuit.remove_qubit(5);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            CircuitError::InvalidQubitIndex { index: 5, num_qubits: 2 }
        ));
    }

    #[test]
    fn test_remove_qubit_with_measurement_fails() {
        let mut circuit = Circuit::new(3);
        circuit.add_measurement(0, 0).unwrap();

        let result = circuit.remove_qubit(0);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::QubitInUse { qubit_id: 0 }));
    }

    #[test]
    fn test_remove_qubit_updates_measurement_indices() {
        let mut circuit = Circuit::new(3);
        // Measurement on q2
        circuit.add_measurement(2, 0).unwrap();

        // Remove q0 (has no gates or measurements)
        circuit.remove_qubit(0).unwrap();

        // Measurement should now target qubit 1 (shifted down)
        assert_eq!(circuit.measurements[0].qubit_id, 1);
    }

    #[test]
    fn test_rename_qubit() {
        let mut circuit = Circuit::new(2);
        circuit.rename_qubit(0, "|ancilla⟩").unwrap();
        assert_eq!(circuit.qubits[0].label, "|ancilla⟩");
        assert_eq!(circuit.qubits[1].label, "|q1⟩");
    }

    #[test]
    fn test_rename_qubit_invalid_index() {
        let mut circuit = Circuit::new(2);
        let result = circuit.rename_qubit(5, "foo");
        assert!(result.is_err());
    }

    #[test]
    fn test_reorder_qubits() {
        let mut circuit = Circuit::new(3);
        // Add gates: H on q0, X on q1, Z on q2
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![1], vec![], 0).unwrap();
        circuit.add_gate(GateType::Z, vec![2], vec![], 0).unwrap();

        // Reorder: [2, 0, 1] means:
        // old q2 -> new q0, old q0 -> new q1, old q1 -> new q2
        circuit.reorder_qubits(&[2, 0, 1]).unwrap();

        // Check qubit IDs
        assert_eq!(circuit.qubits[0].id, 0); // was q2
        assert_eq!(circuit.qubits[1].id, 1); // was q0
        assert_eq!(circuit.qubits[2].id, 2); // was q1

        // Check gate indices updated
        assert_eq!(circuit.gates[0].target_qubits, vec![1]); // H was on q0, now on q1
        assert_eq!(circuit.gates[1].target_qubits, vec![2]); // X was on q1, now on q2
        assert_eq!(circuit.gates[2].target_qubits, vec![0]); // Z was on q2, now on q0
    }

    #[test]
    fn test_reorder_qubits_with_controls() {
        let mut circuit = Circuit::new(4);
        // CNOT with control on q0, target on q3
        circuit.add_gate(GateType::Cnot, vec![3], vec![0], 0).unwrap();

        // Reorder: [3, 2, 1, 0] (reverse)
        circuit.reorder_qubits(&[3, 2, 1, 0]).unwrap();

        // Now control should be q3, target should be q0
        assert_eq!(circuit.gates[0].control_qubits, vec![3]);
        assert_eq!(circuit.gates[0].target_qubits, vec![0]);
    }

    #[test]
    fn test_reorder_qubits_invalid_length() {
        let mut circuit = Circuit::new(3);
        let result = circuit.reorder_qubits(&[0, 1]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::InvalidPermutation { .. }));
    }

    #[test]
    fn test_reorder_qubits_duplicate_index() {
        let mut circuit = Circuit::new(3);
        let result = circuit.reorder_qubits(&[0, 0, 1]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::InvalidPermutation { .. }));
    }

    #[test]
    fn test_reorder_qubits_out_of_bounds() {
        let mut circuit = Circuit::new(3);
        let result = circuit.reorder_qubits(&[0, 1, 5]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), CircuitError::InvalidPermutation { .. }));
    }

    #[test]
    fn test_reorder_qubits_updates_measurements() {
        let mut circuit = Circuit::new(3);
        circuit.add_measurement(0, 0).unwrap();
        circuit.add_measurement(2, 1).unwrap();

        // Reorder: [2, 0, 1]
        circuit.reorder_qubits(&[2, 0, 1]).unwrap();

        // Measurement on q0 -> now on q1
        // Measurement on q2 -> now on q0
        assert_eq!(circuit.measurements[0].qubit_id, 1);
        assert_eq!(circuit.measurements[1].qubit_id, 0);
    }

    #[test]
    fn test_qubit_management_full_workflow() {
        // Test the full workflow from the task description
        let mut circuit = Circuit::new(0);

        // Add 3 qubits
        circuit.add_qubit();
        circuit.add_qubit();
        circuit.add_qubit();
        assert_eq!(circuit.num_qubits(), 3);

        // Rename q1 to '|ancilla⟩'
        circuit.rename_qubit(1, "|ancilla⟩").unwrap();
        assert_eq!(circuit.qubits[1].label, "|ancilla⟩");

        // Add gates on q1 and q2 so we can test index shifting
        circuit.add_gate(GateType::H, vec![1], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![2], vec![], 0).unwrap();

        // Remove q0 (no gates)
        circuit.remove_qubit(0).unwrap();
        assert_eq!(circuit.num_qubits(), 2);

        // Gates should have shifted: H was on q1 -> now on q0, X was on q2 -> now on q1
        assert_eq!(circuit.gates[0].target_qubits, vec![0]);
        assert_eq!(circuit.gates[1].target_qubits, vec![1]);

        // Reorder [1, 0] (swap the two remaining qubits)
        circuit.reorder_qubits(&[1, 0]).unwrap();

        // After swap: H should be on q1, X should be on q0
        assert_eq!(circuit.gates[0].target_qubits, vec![1]);
        assert_eq!(circuit.gates[1].target_qubits, vec![0]);
    }
}
