//! State vector simulation implementation.

use nalgebra::{DVector, Complex};
use rayon::prelude::*;
use ketgrid_core::{Circuit, GateMatrix};
use crate::simulator::{Simulator, SimulationResult, SimulationError};

/// Minimum qubit count to enable Rayon parallel gate application.
const PARALLEL_THRESHOLD: usize = 12;

/// Maximum qubits for storing column checkpoints (memory trade-off).
/// Each checkpoint stores a full copy of the state vector.
const CHECKPOINT_MAX_QUBITS: usize = 15;

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
}

// ---------------------------------------------------------------------------
// Gate application (free functions operating on raw data)
// ---------------------------------------------------------------------------

/// Pre-computed bit layout for a gate application.
struct GateBitLayout {
    bit_positions: Vec<usize>,
    other_bits: Vec<usize>,
    num_other: usize,
    gate_dim: usize,
    m: usize,
}

impl GateBitLayout {
    fn new(num_qubits: usize, qubits: &[usize]) -> Self {
        let m = qubits.len();
        let gate_dim = 1usize << m;

        let bit_positions: Vec<usize> = qubits.iter()
            .map(|&q| num_qubits - 1 - q)
            .collect();

        let other_bits: Vec<usize> = (0..num_qubits)
            .filter(|b| !bit_positions.contains(b))
            .collect();

        let num_other = other_bits.len();

        Self { bit_positions, other_bits, num_other, gate_dim, m }
    }
}

/// Applies a gate using pre-allocated buffers (sequential, zero-allocation hot loop).
fn apply_gate_sequential(
    data: &mut DVector<Complex<f64>>,
    amps: &mut [Complex<f64>],
    indices: &mut [usize],
    matrix: &GateMatrix,
    layout: &GateBitLayout,
) {
    let GateBitLayout { ref bit_positions, ref other_bits, num_other, gate_dim, m } = *layout;

    for other_combo in 0..(1usize << num_other) {
        let mut base = 0usize;
        for (i, &bit_pos) in other_bits.iter().enumerate() {
            if (other_combo >> i) & 1 == 1 {
                base |= 1 << bit_pos;
            }
        }

        for local_idx in 0..gate_dim {
            let mut full_idx = base;
            for (j, &bit_pos) in bit_positions.iter().enumerate() {
                if (local_idx >> (m - 1 - j)) & 1 == 1 {
                    full_idx |= 1 << bit_pos;
                }
            }
            indices[local_idx] = full_idx;
            amps[local_idx] = data[full_idx];
        }

        for row in 0..gate_dim {
            let mut new_amp = Complex::new(0.0, 0.0);
            for col in 0..gate_dim {
                new_amp += matrix[(row, col)] * amps[col];
            }
            data[indices[row]] = new_amp;
        }
    }
}

/// Applies a gate in parallel using Rayon's `par_iter_mut`.
///
/// Copies pre-gate state to `scratch`, then iterates over every output
/// amplitude in parallel. Each output amplitude reads from the immutable
/// `scratch` copy and writes its result to its own exclusive `&mut` slot.
fn apply_gate_parallel(
    data: &mut DVector<Complex<f64>>,
    scratch: &mut Vec<Complex<f64>>,
    matrix: &GateMatrix,
    layout: &GateBitLayout,
) {
    let GateBitLayout { ref bit_positions, ref other_bits, num_other: _, gate_dim, m } = *layout;
    let dim = data.len();

    scratch.resize(dim, Complex::new(0.0, 0.0));
    scratch.copy_from_slice(data.as_slice());
    let scratch_ref = scratch.as_slice();

    let data_slice = data.as_mut_slice();
    data_slice.par_iter_mut().enumerate().for_each(|(idx, output)| {
        // Determine the gate-matrix row for this state vector index.
        let mut row = 0usize;
        for (j, &bit_pos) in bit_positions.iter().enumerate() {
            if (idx >> bit_pos) & 1 == 1 {
                row |= 1 << (m - 1 - j);
            }
        }

        // Reconstruct the base index (non-gate bit portion).
        let mut base = 0usize;
        for &bit_pos in other_bits.iter() {
            if (idx >> bit_pos) & 1 == 1 {
                base |= 1 << bit_pos;
            }
        }

        // Matrix-vector multiply: new_amp = Σ_col M[row,col] · scratch[col_idx]
        let mut new_amp = Complex::new(0.0, 0.0);
        for col in 0..gate_dim {
            let mut full_idx = base;
            for (j, &bit_pos) in bit_positions.iter().enumerate() {
                if (col >> (m - 1 - j)) & 1 == 1 {
                    full_idx |= 1 << bit_pos;
                }
            }
            new_amp += matrix[(row, col)] * scratch_ref[full_idx];
        }

        *output = new_amp;
    });
}

// ---------------------------------------------------------------------------
// StateVectorSimulator
// ---------------------------------------------------------------------------

/// Saved state at a column boundary for incremental re-simulation.
struct ColumnCheckpoint {
    column: usize,
    state_data: Vec<Complex<f64>>,
}

/// Quantum circuit simulator using full state vector representation.
///
/// Applies gates from `ketgrid-core` using their unitary matrices.
/// Owns pre-allocated buffers to eliminate per-gate allocation overhead
/// and supports Rayon-parallelized gate application for large state vectors.
pub struct StateVectorSimulator {
    state: StateVector,
    /// Reusable amplitude buffer (length ≥ max gate dimension).
    amps_buffer: Vec<Complex<f64>>,
    /// Reusable index buffer (length ≥ max gate dimension).
    indices_buffer: Vec<usize>,
    /// Scratch space for parallel gate application (length = 2^n).
    scratch: Vec<Complex<f64>>,
    /// Column checkpoints for incremental re-simulation.
    column_checkpoints: Vec<ColumnCheckpoint>,
}

impl StateVectorSimulator {
    /// Creates a new simulator initialized to |0…0⟩ with pre-allocated buffers.
    pub fn new(num_qubits: usize) -> Self {
        let dim = 1usize << num_qubits;
        Self {
            state: StateVector::new(num_qubits),
            amps_buffer: vec![Complex::new(0.0, 0.0); dim.max(8)],
            indices_buffer: vec![0; dim.max(8)],
            scratch: vec![Complex::new(0.0, 0.0); dim],
            column_checkpoints: Vec::new(),
        }
    }

    /// Applies a unitary gate matrix to the specified qubits, choosing the
    /// sequential or parallel path based on the state vector size.
    fn apply_gate(&mut self, matrix: &GateMatrix, qubits: &[usize]) {
        let layout = GateBitLayout::new(self.state.num_qubits, qubits);

        if self.state.num_qubits >= PARALLEL_THRESHOLD {
            apply_gate_parallel(
                &mut self.state.data,
                &mut self.scratch,
                matrix,
                &layout,
            );
        } else {
            apply_gate_sequential(
                &mut self.state.data,
                &mut self.amps_buffer[..layout.gate_dim],
                &mut self.indices_buffer[..layout.gate_dim],
                matrix,
                &layout,
            );
        }
    }

    /// Applies all gates in the circuit (column order) to the state vector.
    ///
    /// Saves column checkpoints for circuits ≤ [`CHECKPOINT_MAX_QUBITS`] qubits
    /// to enable incremental re-simulation via [`apply_circuit_from_column`].
    pub fn apply_circuit(&mut self, circuit: &Circuit) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        self.column_checkpoints.clear();
        let save_checkpoints = self.state.num_qubits <= CHECKPOINT_MAX_QUBITS;
        let mut prev_col: Option<usize> = None;

        for placed_gate in circuit.gates_by_column() {
            if save_checkpoints && prev_col != Some(placed_gate.column) {
                self.column_checkpoints.push(ColumnCheckpoint {
                    column: placed_gate.column,
                    state_data: self.state.data.as_slice().to_vec(),
                });
                prev_col = Some(placed_gate.column);
            }

            let Some(matrix) = placed_gate.gate.matrix() else {
                continue;
            };

            let mut qubits = placed_gate.control_qubits.clone();
            qubits.extend(&placed_gate.target_qubits);
            self.apply_gate(&matrix, &qubits);
        }
    }

    /// Incrementally re-simulates the circuit from a stored column checkpoint.
    ///
    /// Restores the state to the latest checkpoint at or before `from_column`,
    /// then re-applies all gates from that point forward. Falls back to full
    /// simulation if no suitable checkpoint exists.
    pub fn apply_circuit_from_column(&mut self, circuit: &Circuit, from_column: usize) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        let restore = self.column_checkpoints.iter()
            .rposition(|cp| cp.column <= from_column);

        if let Some(cp_idx) = restore {
            let restore_col = self.column_checkpoints[cp_idx].column;
            let dim = self.state.data.len();
            for i in 0..dim {
                self.state.data[i] = self.column_checkpoints[cp_idx].state_data[i];
            }
            self.column_checkpoints.truncate(cp_idx);

            let save_checkpoints = self.state.num_qubits <= CHECKPOINT_MAX_QUBITS;
            let mut prev_col: Option<usize> = None;

            for placed_gate in circuit.gates_by_column() {
                if placed_gate.column < restore_col {
                    continue;
                }

                if save_checkpoints && prev_col != Some(placed_gate.column) {
                    self.column_checkpoints.push(ColumnCheckpoint {
                        column: placed_gate.column,
                        state_data: self.state.data.as_slice().to_vec(),
                    });
                    prev_col = Some(placed_gate.column);
                }

                let Some(matrix) = placed_gate.gate.matrix() else {
                    continue;
                };
                let mut qubits = placed_gate.control_qubits.clone();
                qubits.extend(&placed_gate.target_qubits);
                self.apply_gate(&matrix, &qubits);
            }
        } else {
            self.state = StateVector::new(self.state.num_qubits);
            self.apply_circuit(circuit);
        }
    }

    /// Applies the circuit with gate fusion for maximum throughput.
    ///
    /// Consecutive single-qubit gates on the same qubit are composed into a
    /// single fused matrix, reducing the number of state vector traversals.
    /// Does not save column checkpoints (use for one-shot simulation).
    pub fn apply_circuit_optimized(&mut self, circuit: &Circuit) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        self.column_checkpoints.clear();
        let sorted_gates = circuit.gates_by_column();
        let n = self.state.num_qubits;

        // Per-qubit accumulated single-qubit gate matrix (DMatrix 2x2).
        let mut qubit_acc: Vec<Option<GateMatrix>> = vec![None; n];

        for placed_gate in &sorted_gates {
            let Some(matrix) = placed_gate.gate.matrix() else {
                // Non-matrix gate (Barrier, Custom): flush involved qubits.
                for &q in &placed_gate.target_qubits {
                    if let Some(acc) = qubit_acc[q].take() {
                        self.apply_gate(&acc, &[q]);
                    }
                }
                continue;
            };

            let all_qubits = placed_gate.all_qubits();

            if placed_gate.gate.num_qubits() == 1
                && !placed_gate.gate.is_controlled()
                && placed_gate.target_qubits.len() == 1
            {
                let q = placed_gate.target_qubits[0];
                qubit_acc[q] = Some(match qubit_acc[q].take() {
                    Some(acc) => &matrix * &acc,
                    None => matrix,
                });
            } else {
                // Multi-qubit or controlled gate: flush involved qubits first.
                for &q in &all_qubits {
                    if let Some(acc) = qubit_acc[q].take() {
                        self.apply_gate(&acc, &[q]);
                    }
                }
                let mut qubits = placed_gate.control_qubits.clone();
                qubits.extend(&placed_gate.target_qubits);
                self.apply_gate(&matrix, &qubits);
            }
        }

        // Flush remaining accumulated single-qubit gates.
        for q in 0..n {
            if let Some(acc) = qubit_acc[q].take() {
                self.apply_gate(&acc, &[q]);
            }
        }
    }

    /// Applies gates in the circuit up to and including `max_column`.
    pub fn apply_columns_up_to(&mut self, circuit: &Circuit, max_column: usize) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        for placed_gate in circuit.gates_by_column() {
            if placed_gate.column > max_column {
                break;
            }
            let Some(matrix) = placed_gate.gate.matrix() else {
                continue;
            };
            let mut qubits = placed_gate.control_qubits.clone();
            qubits.extend(&placed_gate.target_qubits);
            self.apply_gate(&matrix, &qubits);
        }
    }

    /// Applies only the gates at exactly the given column.
    pub fn apply_column(&mut self, circuit: &Circuit, column: usize) {
        debug_assert_eq!(
            circuit.num_qubits(),
            self.state.num_qubits(),
            "Circuit qubit count ({}) must match simulator ({})",
            circuit.num_qubits(),
            self.state.num_qubits(),
        );

        for placed_gate in circuit.gates_by_column() {
            if placed_gate.column < column {
                continue;
            }
            if placed_gate.column > column {
                break;
            }
            let Some(matrix) = placed_gate.gate.matrix() else {
                continue;
            };
            let mut qubits = placed_gate.control_qubits.clone();
            qubits.extend(&placed_gate.target_qubits);
            self.apply_gate(&matrix, &qubits);
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

/// Entanglement information for a quantum state.
///
/// Groups qubits into entanglement clusters based on pairwise correlations
/// in the reduced density matrices. Qubits that share quantum correlations
/// (purity deviation from product of marginals) are placed in the same cluster.
#[derive(Debug, Clone)]
pub struct EntanglementInfo {
    /// Entanglement clusters — groups of mutually entangled qubits.
    /// Each qubit appears in exactly one cluster. Single-element clusters
    /// represent unentangled (pure) qubits.
    pub clusters: Vec<Vec<usize>>,
    /// Per-qubit purity: Tr(ρ²) where ρ is the single-qubit reduced density matrix.
    /// 1.0 = pure (separable), 0.5 = maximally mixed (maximally entangled with partner).
    pub qubit_purities: Vec<f64>,
    /// Cluster index for each qubit: `qubit_cluster[i]` gives the index into `clusters`.
    pub qubit_cluster: Vec<usize>,
}

impl StateVector {
    /// Computes the purity Tr(ρ²) of the single-qubit reduced density matrix for `qubit`.
    ///
    /// Returns 1.0 for a pure (unentangled) qubit, 0.5 for a maximally mixed qubit.
    pub fn single_qubit_purity(&self, qubit: usize) -> f64 {
        let n = self.num_qubits;
        debug_assert!(qubit < n, "qubit {qubit} out of range for {n}-qubit state");

        let amplitudes = &self.data;
        let bit_pos = n - 1 - qubit;
        let dim = amplitudes.len();

        let mut rho_00: f64 = 0.0;
        let mut rho_11: f64 = 0.0;
        let mut rho_01_re: f64 = 0.0;
        let mut rho_01_im: f64 = 0.0;

        for i in 0..dim {
            if (i >> bit_pos) & 1 == 0 {
                let j = i | (1 << bit_pos);
                let ai = amplitudes[i];
                let aj = amplitudes[j];

                rho_00 += ai.norm_sqr();
                rho_11 += aj.norm_sqr();
                rho_01_re += ai.re * aj.re + ai.im * aj.im;
                rho_01_im += ai.im * aj.re - ai.re * aj.im;
            }
        }

        // Tr(ρ²) = ρ₀₀² + ρ₁₁² + 2|ρ₀₁|²
        rho_00 * rho_00 + rho_11 * rho_11
            + 2.0 * (rho_01_re * rho_01_re + rho_01_im * rho_01_im)
    }

    /// Computes the 4×4 reduced density matrix for qubits `qa` and `qb`,
    /// tracing out all other qubits.
    ///
    /// Row/column indices use the convention: index = qa_val * 2 + qb_val.
    pub fn reduced_density_matrix_2qubit(
        &self,
        qa: usize,
        qb: usize,
    ) -> [[Complex<f64>; 4]; 4] {
        let n = self.num_qubits;
        debug_assert!(qa < n && qb < n && qa != qb);

        let amplitudes = &self.data;
        let bit_a = n - 1 - qa;
        let bit_b = n - 1 - qb;

        let other_bits: Vec<usize> = (0..n)
            .filter(|&b| b != bit_a && b != bit_b)
            .collect();
        let num_other = other_bits.len();

        let zero = Complex::new(0.0, 0.0);
        let mut rho = [[zero; 4]; 4];

        for other_combo in 0..(1usize << num_other) {
            let mut base = 0usize;
            for (i, &bit_pos) in other_bits.iter().enumerate() {
                if (other_combo >> i) & 1 == 1 {
                    base |= 1 << bit_pos;
                }
            }

            for row in 0..4usize {
                let ra = (row >> 1) & 1;
                let rb = row & 1;
                let mut idx_row = base;
                if ra == 1 { idx_row |= 1 << bit_a; }
                if rb == 1 { idx_row |= 1 << bit_b; }

                let amp_row = amplitudes[idx_row];

                for col in 0..4usize {
                    let ca = (col >> 1) & 1;
                    let cb = col & 1;
                    let mut idx_col = base;
                    if ca == 1 { idx_col |= 1 << bit_a; }
                    if cb == 1 { idx_col |= 1 << bit_b; }

                    // ρ[row,col] += ψ[idx_row] · conj(ψ[idx_col])
                    rho[row][col] += amp_row * amplitudes[idx_col].conj();
                }
            }
        }

        rho
    }
}

/// Computes Tr(ρ²) for a 4×4 density matrix (Frobenius norm squared).
fn purity_4x4(rho: &[[Complex<f64>; 4]; 4]) -> f64 {
    let mut sum = 0.0;
    for row in rho {
        for entry in row {
            sum += entry.norm_sqr();
        }
    }
    sum
}

/// Detects entanglement clusters from a state vector.
///
/// Uses pairwise linear mutual information: for each pair (i, j), compares
/// the purity of the 2-qubit reduced density matrix against the product of
/// single-qubit purities. Deviations indicate quantum correlations.
/// Union-find groups correlated qubits into clusters.
pub fn compute_entanglement_info(sv: &StateVector) -> EntanglementInfo {
    let n = sv.num_qubits();
    if n == 0 {
        return EntanglementInfo {
            clusters: Vec::new(),
            qubit_purities: Vec::new(),
            qubit_cluster: Vec::new(),
        };
    }

    let purities: Vec<f64> = (0..n).map(|q| sv.single_qubit_purity(q)).collect();

    // Union-find
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], x: usize) -> usize {
        let mut r = x;
        while parent[r] != r {
            r = parent[r];
        }
        let mut c = x;
        while parent[c] != r {
            let next = parent[c];
            parent[c] = r;
            c = next;
        }
        r
    }

    fn union(parent: &mut [usize], x: usize, y: usize) {
        let rx = find(parent, x);
        let ry = find(parent, y);
        if rx != ry {
            parent[ry] = rx;
        }
    }

    const THRESHOLD: f64 = 0.001;

    for i in 0..n {
        if purities[i] > 1.0 - THRESHOLD {
            continue;
        }
        for j in (i + 1)..n {
            if purities[j] > 1.0 - THRESHOLD {
                continue;
            }

            let rho_ij = sv.reduced_density_matrix_2qubit(i, j);
            let purity_ij = purity_4x4(&rho_ij);
            let product_purity = purities[i] * purities[j];

            if (purity_ij - product_purity).abs() > THRESHOLD {
                union(&mut parent, i, j);
            }
        }
    }

    // Build clusters
    let mut cluster_map: std::collections::HashMap<usize, Vec<usize>> =
        std::collections::HashMap::new();
    for i in 0..n {
        let root = find(&mut parent, i);
        cluster_map.entry(root).or_default().push(i);
    }

    let mut clusters: Vec<Vec<usize>> = cluster_map.into_values().collect();
    clusters.sort_by_key(|c| c[0]);

    let mut qubit_cluster = vec![0; n];
    for (idx, cluster) in clusters.iter().enumerate() {
        for &q in cluster {
            qubit_cluster[q] = idx;
        }
    }

    EntanglementInfo {
        clusters,
        qubit_purities: purities,
        qubit_cluster,
    }
}

impl Simulator for StateVectorSimulator {
    fn run(&mut self, circuit: &Circuit) -> Result<SimulationResult, SimulationError> {
        if circuit.num_qubits() == 0 {
            return Err(SimulationError::InvalidCircuit(
                "Circuit has no qubits".to_string(),
            ));
        }

        let n = circuit.num_qubits();
        if n != self.state.num_qubits {
            *self = Self::new(n);
        } else {
            self.state = StateVector::new(n);
            self.column_checkpoints.clear();
        }
        self.apply_circuit(circuit);

        Ok(SimulationResult {
            state_vector: Some(self.state_vector().to_vec()),
            probabilities: self.probabilities(),
            measurements: Vec::new(),
            num_qubits: n,
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
    // Column-wise stepping: Bell state Step1 H → superposition, Step2 CNOT → entanglement
    // ------------------------------------------------------------------
    #[test]
    fn test_step_through_bell_state() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        // Step 0: |00⟩ (nothing applied)
        let sim0 = StateVectorSimulator::new(2);
        assert_probs_eq(&sim0.probabilities(), &[1.0, 0.0, 0.0, 0.0]);

        // Step 1: apply column 0 (H on q0) → superposition on q0
        let mut sim1 = StateVectorSimulator::new(2);
        sim1.apply_column(&circuit, 0);
        assert_probs_eq(&sim1.probabilities(), &[0.5, 0.0, 0.5, 0.0]);

        // Step 2: apply column 1 (CNOT) on top of step 1 → entangled Bell state
        sim1.apply_column(&circuit, 1);
        assert_probs_eq(&sim1.probabilities(), &[0.5, 0.0, 0.0, 0.5]);
    }

    // ------------------------------------------------------------------
    // apply_columns_up_to: partial circuit execution
    // ------------------------------------------------------------------
    #[test]
    fn test_apply_columns_up_to() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        // Up to column 0 only (H applied, CNOT not)
        let mut sim = StateVectorSimulator::new(2);
        sim.apply_columns_up_to(&circuit, 0);
        assert_probs_eq(&sim.probabilities(), &[0.5, 0.0, 0.5, 0.0]);

        // Up to column 1 (full circuit)
        let mut sim2 = StateVectorSimulator::new(2);
        sim2.apply_columns_up_to(&circuit, 1);
        assert_probs_eq(&sim2.probabilities(), &[0.5, 0.0, 0.0, 0.5]);
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

    // ==================================================================
    // Entanglement detection tests
    // ==================================================================

    // Single-qubit purity for |0⟩ should be 1.0 (pure)
    #[test]
    fn test_single_qubit_purity_pure() {
        let sv = StateVector::new(1);
        let purity = sv.single_qubit_purity(0);
        assert!(
            (purity - 1.0).abs() < TOLERANCE,
            "Pure |0⟩ state should have purity 1.0, got {purity}",
        );
    }

    // Single-qubit purity for Bell state should be 0.5 (maximally mixed)
    #[test]
    fn test_single_qubit_purity_entangled() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        for q in 0..2 {
            let purity = sim.state().single_qubit_purity(q);
            assert!(
                (purity - 0.5).abs() < TOLERANCE,
                "Bell state qubit {q} should have purity 0.5, got {purity}",
            );
        }
    }

    // H|0⟩ on qubit 0 only → q0 pure, q1 pure (product state)
    #[test]
    fn test_single_qubit_purity_superposition() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let purity_q0 = sim.state().single_qubit_purity(0);
        let purity_q1 = sim.state().single_qubit_purity(1);
        assert!(
            (purity_q0 - 1.0).abs() < TOLERANCE,
            "H|0> on q0: q0 should be pure (purity=1.0), got {purity_q0}",
        );
        assert!(
            (purity_q1 - 1.0).abs() < TOLERANCE,
            "H|0> on q0: q1 should be pure (purity=1.0), got {purity_q1}",
        );
    }

    // 2-qubit reduced density matrix: Bell state should be maximally entangled
    #[test]
    fn test_reduced_density_matrix_bell() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let rho = sim.state().reduced_density_matrix_2qubit(0, 1);
        let purity = purity_4x4(&rho);
        // Bell state as a 2-qubit system is pure → purity should be 1.0
        assert!(
            (purity - 1.0).abs() < TOLERANCE,
            "Bell state 2-qubit reduced density matrix should have purity 1.0, got {purity}",
        );
    }

    // Entanglement clusters: Bell state → q0, q1 in same cluster
    #[test]
    fn test_entanglement_clusters_bell() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let info = compute_entanglement_info(sim.state());

        assert_eq!(info.clusters.len(), 1, "Bell state should have 1 cluster");
        assert_eq!(
            info.clusters[0].len(),
            2,
            "Bell state cluster should contain 2 qubits",
        );
        assert!(
            info.clusters[0].contains(&0) && info.clusters[0].contains(&1),
            "Bell state cluster should contain q0 and q1",
        );
        assert_eq!(
            info.qubit_cluster[0], info.qubit_cluster[1],
            "q0 and q1 should be in the same cluster",
        );
    }

    // Entanglement clusters: H on q0 only → each qubit in its own cluster
    #[test]
    fn test_entanglement_clusters_no_entanglement() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();

        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        let info = compute_entanglement_info(sim.state());

        assert_eq!(
            info.clusters.len(),
            2,
            "Product state should have 2 separate clusters",
        );
        for cluster in &info.clusters {
            assert_eq!(
                cluster.len(),
                1,
                "Each cluster should contain exactly 1 qubit",
            );
        }
        assert_ne!(
            info.qubit_cluster[0], info.qubit_cluster[1],
            "q0 and q1 should be in different clusters",
        );
    }

    // GHZ state: all 3 qubits entangled in one cluster
    #[test]
    fn test_entanglement_clusters_ghz() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_gate(GateType::Cnot, vec![2], vec![0], 2).unwrap();

        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        let info = compute_entanglement_info(sim.state());

        assert_eq!(info.clusters.len(), 1, "GHZ state should have 1 cluster");
        assert_eq!(
            info.clusters[0].len(),
            3,
            "GHZ cluster should contain all 3 qubits",
        );
    }

    // Mixed entanglement: Bell(q0, q1) ⊗ |0⟩_q2 → two clusters
    #[test]
    fn test_entanglement_clusters_partial() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        let info = compute_entanglement_info(sim.state());

        assert_eq!(
            info.clusters.len(),
            2,
            "Bell(q0,q1) ⊗ |0⟩_q2 should have 2 clusters",
        );

        let entangled_cluster = info.clusters.iter().find(|c| c.len() == 2).unwrap();
        assert!(
            entangled_cluster.contains(&0) && entangled_cluster.contains(&1),
            "Entangled cluster should contain q0 and q1",
        );

        let solo_cluster = info.clusters.iter().find(|c| c.len() == 1).unwrap();
        assert_eq!(solo_cluster[0], 2, "Solo cluster should contain q2");
    }

    // Initial |000⟩ state → all qubits pure, no entanglement
    #[test]
    fn test_entanglement_initial_state() {
        let sim = StateVectorSimulator::new(3);
        let info = compute_entanglement_info(sim.state());

        assert_eq!(info.clusters.len(), 3);
        for &purity in &info.qubit_purities {
            assert!(
                (purity - 1.0).abs() < TOLERANCE,
                "Initial state qubits should be pure, got purity {purity}",
            );
        }
    }

    // ==================================================================
    // Optimization-specific tests
    // ==================================================================

    // Gate fusion: H-Z-H on same qubit should produce same result as unfused
    #[test]
    fn test_gate_fusion_hzh_equals_x() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Z, vec![0], vec![], 1).unwrap();
        circuit.add_gate(GateType::H, vec![0], vec![], 2).unwrap();

        // Standard simulation
        let mut sim_std = StateVectorSimulator::new(1);
        sim_std.apply_circuit(&circuit);

        // Optimized (fused) simulation
        let mut sim_opt = StateVectorSimulator::new(1);
        sim_opt.apply_circuit_optimized(&circuit);

        assert_probs_eq(&sim_std.probabilities(), &sim_opt.probabilities());
        // H-Z-H = X, so |0⟩ → |1⟩
        assert_probs_eq(&sim_opt.probabilities(), &[0.0, 1.0]);
    }

    // Gate fusion with mixed single/multi-qubit gates
    #[test]
    fn test_gate_fusion_mixed() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 1).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 2).unwrap();
        circuit.add_gate(GateType::Z, vec![0], vec![], 3).unwrap();

        let mut sim_std = StateVectorSimulator::new(2);
        sim_std.apply_circuit(&circuit);

        let mut sim_opt = StateVectorSimulator::new(2);
        sim_opt.apply_circuit_optimized(&circuit);

        for (i, (a, b)) in sim_std.state_vector().iter()
            .zip(sim_opt.state_vector().iter())
            .enumerate()
        {
            assert!(
                (a - b).norm() < TOLERANCE,
                "Amplitude mismatch at index {i}: std={a}, opt={b}",
            );
        }
    }

    // Incremental simulation produces same result as full simulation
    #[test]
    fn test_incremental_simulation() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 2).unwrap();

        // Full simulation
        let mut sim_full = StateVectorSimulator::new(2);
        sim_full.apply_circuit(&circuit);

        // Simulate first two columns, then add third and do incremental
        let mut circuit2 = Circuit::new(2);
        circuit2.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit2.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let mut sim_inc = StateVectorSimulator::new(2);
        sim_inc.apply_circuit(&circuit2);

        // Now simulate the full circuit incrementally from column 2
        sim_inc.apply_circuit_from_column(&circuit, 2);

        for (i, (a, b)) in sim_full.state_vector().iter()
            .zip(sim_inc.state_vector().iter())
            .enumerate()
        {
            assert!(
                (a - b).norm() < TOLERANCE,
                "Incremental mismatch at index {i}: full={a}, inc={b}",
            );
        }
    }
}
