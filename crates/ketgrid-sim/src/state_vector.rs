//! State vector simulation implementation.

use nalgebra::{DVector, Complex};
use ketgrid_core::{Circuit, GateMatrix};
use crate::simulator::{Simulator, SimulationResult, SimulationError};

/// A state vector representing the quantum state of n qubits.
///
/// Uses big-endian qubit ordering: qubit 0 is the most significant bit.
/// Basis state |q₀ q₁ … qₙ₋₁⟩ maps to index q₀·2ⁿ⁻¹ + q₁·2ⁿ⁻² + … + qₙ₋₁.
pub struct StateVector {
    /// The underlying complex vector (length = 2^n).
    data: DVector<Complex<f64>>,
    /// Number of qubits.
    num_qubits: usize,
}

impl StateVector {
    /// Creates a new state vector initialized to |0…0⟩.
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1usize << num_qubits;
        let mut data = DVector::zeros(dim);
        data[0] = Complex::new(1.0, 0.0);
        Self { data, num_qubits }
    }

    /// Returns the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Returns a reference to the underlying data.
    pub fn data(&self) -> &DVector<Complex<f64>> {
        &self.data
    }

    /// Returns |amplitude|² for each computational basis state.
    pub fn probabilities(&self) -> Vec<f64> {
        self.data.iter()
            .map(|c| c.norm_sqr())
            .collect()
    }

    /// Applies a unitary gate matrix to the specified qubits.
    ///
    /// `qubits` lists circuit qubit indices in gate-matrix order (MSB first).
    /// For controlled gates this means controls first, then targets.
    fn apply_gate(&mut self, matrix: &GateMatrix, qubits: &[usize]) {
        let n = self.num_qubits;
        let m = qubits.len();
        let gate_dim = 1usize << m;

        // Bit positions for gate qubits in the state vector index (big-endian).
        let bit_positions: Vec<usize> = qubits.iter()
            .map(|&q| n - 1 - q)
            .collect();

        // Bit positions for qubits NOT involved in the gate.
        let other_bits: Vec<usize> = (0..n)
            .filter(|b| !bit_positions.contains(b))
            .collect();

        let num_other = other_bits.len();

        // Iterate over every combination of "other" qubit values.
        for other_combo in 0..(1usize << num_other) {
            // Base index with gate-qubit bits zeroed, other bits from other_combo.
            let mut base = 0usize;
            for (i, &bit_pos) in other_bits.iter().enumerate() {
                if (other_combo >> i) & 1 == 1 {
                    base |= 1 << bit_pos;
                }
            }

            // Collect amplitudes for all 2^m gate-qubit combinations.
            let mut indices = Vec::with_capacity(gate_dim);
            let mut amps = Vec::with_capacity(gate_dim);
            for local_idx in 0..gate_dim {
                let mut full_idx = base;
                for (j, &bit_pos) in bit_positions.iter().enumerate() {
                    if (local_idx >> (m - 1 - j)) & 1 == 1 {
                        full_idx |= 1 << bit_pos;
                    }
                }
                indices.push(full_idx);
                amps.push(self.data[full_idx]);
            }

            // Matrix-vector multiply within this subspace.
            for row in 0..gate_dim {
                let mut new_amp = Complex::new(0.0, 0.0);
                for col in 0..gate_dim {
                    new_amp += matrix[(row, col)] * amps[col];
                }
                self.data[indices[row]] = new_amp;
            }
        }
    }
}

/// Quantum circuit simulator using full state vector representation.
///
/// Applies gates from `ketgrid-core` using their unitary matrices.
/// This is the custom fallback simulator (always available, no external
/// simulation library required).
pub struct StateVectorSimulator {
    state: StateVector,
}

impl StateVectorSimulator {
    /// Creates a new simulator initialized to |0…0⟩.
    pub fn new(num_qubits: usize) -> Self {
        Self {
            state: StateVector::new(num_qubits),
        }
    }

    /// Applies all gates in the circuit (column order) to the state vector.
    ///
    /// Gates without a matrix representation (Barrier, Custom) are skipped.
    /// Panics (debug) if the circuit qubit count doesn't match the simulator.
    pub fn apply_circuit(&mut self, circuit: &Circuit) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        for placed_gate in circuit.gates_by_column() {
            let Some(matrix) = placed_gate.gate.matrix() else {
                continue;
            };

            // Gate matrix expects controls (MSB) then targets (LSB).
            let mut qubits = placed_gate.control_qubits.clone();
            qubits.extend(&placed_gate.target_qubits);

            self.state.apply_gate(&matrix, &qubits);
        }
    }

    /// Returns the current state vector amplitudes.
    pub fn state_vector(&self) -> &[Complex<f64>] {
        self.state.data().as_slice()
    }

    /// Returns |amplitude|² for each computational basis state.
    pub fn probabilities(&self) -> Vec<f64> {
        self.state.probabilities()
    }

    /// Returns the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.state.num_qubits()
    }

    /// Returns a reference to the underlying state vector.
    pub fn state(&self) -> &StateVector {
        &self.state
    }
}

impl Simulator for StateVectorSimulator {
    fn run(&mut self, circuit: &Circuit) -> Result<SimulationResult, SimulationError> {
        if circuit.num_qubits() == 0 {
            return Err(SimulationError::InvalidCircuit(
                "Circuit has no qubits".to_string(),
            ));
        }

        // Reset state to |0…0⟩ matching the circuit size.
        self.state = StateVector::new(circuit.num_qubits());
        self.apply_circuit(circuit);

        Ok(SimulationResult {
            state_vector: Some(self.state_vector().to_vec()),
            probabilities: self.probabilities(),
            measurements: Vec::new(),
            num_qubits: circuit.num_qubits(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ketgrid_core::GateType;

    const TOLERANCE: f64 = 1e-10;

    fn assert_probs_eq(actual: &[f64], expected: &[f64]) {
        assert_eq!(actual.len(), expected.len(), "probability vector length mismatch");
        for (i, (a, e)) in actual.iter().zip(expected).enumerate() {
            assert!(
                (a - e).abs() < TOLERANCE,
                "probability mismatch at |{i}⟩: got {a}, expected {e}",
            );
        }
    }

    fn assert_amp_eq(actual: Complex<f64>, expected: Complex<f64>, label: &str) {
        assert!(
            (actual - expected).norm() < TOLERANCE,
            "amplitude mismatch for {label}: got {actual}, expected {expected}",
        );
    }

    // ------------------------------------------------------------------
    // Bell state: H q0, CNOT q0→q1  →  |Φ⁺⟩ = (|00⟩ + |11⟩) / √2
    // ------------------------------------------------------------------
    #[test]
    fn test_bell_state() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let sv = sim.state_vector();
        let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;
        assert_amp_eq(sv[0], Complex::new(sqrt2_inv, 0.0), "|00⟩");
        assert_amp_eq(sv[1], Complex::new(0.0, 0.0), "|01⟩");
        assert_amp_eq(sv[2], Complex::new(0.0, 0.0), "|10⟩");
        assert_amp_eq(sv[3], Complex::new(sqrt2_inv, 0.0), "|11⟩");

        assert_probs_eq(&sim.probabilities(), &[0.5, 0.0, 0.0, 0.5]);
    }

    // ------------------------------------------------------------------
    // X gate: |0⟩ → |1⟩
    // ------------------------------------------------------------------
    #[test]
    fn test_x_gate() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[0.0, 1.0]);
    }

    // ------------------------------------------------------------------
    // H gate: |0⟩ → (|0⟩ + |1⟩)/√2
    // ------------------------------------------------------------------
    #[test]
    fn test_hadamard_superposition() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[0.5, 0.5]);
    }

    // ------------------------------------------------------------------
    // HH = I: applying Hadamard twice returns to |0⟩
    // ------------------------------------------------------------------
    #[test]
    fn test_hadamard_self_inverse() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::H, vec![0], vec![], 1).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[1.0, 0.0]);
    }

    // ------------------------------------------------------------------
    // XX = I: Pauli-X is its own inverse
    // ------------------------------------------------------------------
    #[test]
    fn test_x_self_inverse() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 1).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[1.0, 0.0]);
    }

    // ------------------------------------------------------------------
    // Z|0⟩ = |0⟩ (no observable probability change)
    // Z|1⟩ = -|1⟩ (phase only, same probability)
    // ------------------------------------------------------------------
    #[test]
    fn test_z_gate() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::Z, vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[1.0, 0.0]);

        let sv = sim.state_vector();
        assert_amp_eq(sv[0], Complex::new(1.0, 0.0), "|0⟩");
    }

    // ------------------------------------------------------------------
    // SWAP: |01⟩ → |10⟩
    // ------------------------------------------------------------------
    #[test]
    fn test_swap_gate() {
        let mut circuit = Circuit::new(2);
        // Prepare |01⟩: apply X to qubit 1
        circuit.add_gate(GateType::X, vec![1], vec![], 0).unwrap();
        circuit.add_gate(GateType::Swap, vec![0, 1], vec![], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        // After SWAP, should be |10⟩ (index 2)
        assert_probs_eq(&sim.probabilities(), &[0.0, 0.0, 1.0, 0.0]);
    }

    // ------------------------------------------------------------------
    // CZ: applies -1 phase to |11⟩ only
    // ------------------------------------------------------------------
    #[test]
    fn test_cz_gate() {
        let mut circuit = Circuit::new(2);
        // Prepare |11⟩
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![1], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cz, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let sv = sim.state_vector();
        assert_amp_eq(sv[3], Complex::new(-1.0, 0.0), "|11⟩ after CZ");
        assert_probs_eq(&sim.probabilities(), &[0.0, 0.0, 0.0, 1.0]);
    }

    // ------------------------------------------------------------------
    // 3-qubit GHZ: H q0, CNOT q0→q1, CNOT q0→q2
    // ------------------------------------------------------------------
    #[test]
    fn test_ghz_state() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_gate(GateType::Cnot, vec![2], vec![0], 2).unwrap();

        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        // GHZ = (|000⟩ + |111⟩) / √2
        let expected_probs = [0.5, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.5];
        assert_probs_eq(&sim.probabilities(), &expected_probs);
    }

    // ------------------------------------------------------------------
    // Toffoli: flips target only when both controls are |1⟩
    // ------------------------------------------------------------------
    #[test]
    fn test_toffoli_gate() {
        let mut circuit = Circuit::new(3);
        // Prepare |110⟩: X on q0 and q1
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![1], vec![], 0).unwrap();
        // Toffoli: controls=[0,1], target=[2]
        circuit.add_gate(GateType::Toffoli, vec![2], vec![0, 1], 1).unwrap();

        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        // |110⟩ → |111⟩ (index 7)
        assert_probs_eq(
            &sim.probabilities(),
            &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0],
        );
    }

    // ------------------------------------------------------------------
    // Toffoli: no flip when only one control is set
    // ------------------------------------------------------------------
    #[test]
    fn test_toffoli_no_flip_single_control() {
        let mut circuit = Circuit::new(3);
        // Prepare |100⟩: X on q0 only
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Toffoli, vec![2], vec![0, 1], 1).unwrap();

        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        // Should stay |100⟩ (index 4)
        assert_probs_eq(
            &sim.probabilities(),
            &[0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0],
        );
    }

    // ------------------------------------------------------------------
    // Barrier/Identity are harmless
    // ------------------------------------------------------------------
    #[test]
    fn test_barrier_and_identity() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::Barrier, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Identity, vec![0], vec![], 1).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[1.0, 0.0]);
    }

    // ------------------------------------------------------------------
    // Simulator trait: run() resets state and returns result
    // ------------------------------------------------------------------
    #[test]
    fn test_simulator_trait_run() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        let result = sim.run(&circuit).unwrap();

        assert_eq!(result.num_qubits, 2);
        assert_probs_eq(&result.probabilities, &[0.5, 0.0, 0.0, 0.5]);
        assert!(result.state_vector.is_some());
        assert_eq!(result.state_vector.unwrap().len(), 4);
    }

    // ------------------------------------------------------------------
    // Simulator trait: run() on empty circuit is an error
    // ------------------------------------------------------------------
    #[test]
    fn test_simulator_empty_circuit_error() {
        let circuit = Circuit::new(0);
        let mut sim = StateVectorSimulator::new(1);

        let result = sim.run(&circuit);
        assert!(result.is_err());
    }

    // ------------------------------------------------------------------
    // Rx(π) ≈ -iX: |0⟩ → -i|1⟩
    // ------------------------------------------------------------------
    #[test]
    fn test_rx_gate() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::Rx(std::f64::consts::PI), vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);

        assert_probs_eq(&sim.probabilities(), &[0.0, 1.0]);

        let sv = sim.state_vector();
        assert_amp_eq(sv[1], Complex::new(0.0, -1.0), "|1⟩ after Rx(π)");
    }
}
