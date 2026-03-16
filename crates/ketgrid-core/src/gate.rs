//! Gate type definitions and metadata.

use nalgebra::{Complex, DMatrix, Matrix2, Matrix4, SMatrix};
use serde::{Deserialize, Serialize};

/// Complex number type for gate matrices (f64 precision).
pub type C = Complex<f64>;

/// 2x2 complex matrix for single-qubit gates.
pub type GateMatrix2 = Matrix2<C>;

/// 4x4 complex matrix for two-qubit gates.
pub type GateMatrix4 = Matrix4<C>;

/// 8x8 complex matrix for three-qubit gates.
pub type GateMatrix8 = SMatrix<C, 8, 8>;

/// Dynamic complex matrix for gates of any size.
pub type GateMatrix = DMatrix<C>;

/// Supported quantum gate types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum GateType {
    // Single-qubit gates
    /// Hadamard gate (creates superposition).
    H,
    /// Pauli-X gate (bit flip / NOT).
    X,
    /// Pauli-Y gate.
    Y,
    /// Pauli-Z gate (phase flip).
    Z,
    /// S gate (π/2 phase).
    S,
    /// T gate (π/4 phase).
    T,
    /// Rotation around X axis.
    Rx(f64),
    /// Rotation around Y axis.
    Ry(f64),
    /// Rotation around Z axis.
    Rz(f64),

    // Multi-qubit gates
    /// Controlled-NOT gate (dot + ⊕).
    Cnot,
    /// Controlled-Z gate.
    Cz,
    /// Swap gate (✕ + ✕ connected).
    Swap,
    /// Toffoli gate (CC-NOT — two dots + ⊕).
    Toffoli,

    // Meta gates
    /// Visual separator (no operation).
    Barrier,
    /// Identity / no-op gate.
    Identity,
    /// User-defined custom gate.
    Custom(String),
}

impl GateType {
    /// Returns the number of qubits the gate operates on.
    pub fn num_qubits(&self) -> usize {
        match self {
            GateType::H | GateType::X | GateType::Y | GateType::Z => 1,
            GateType::S | GateType::T => 1,
            GateType::Rx(_) | GateType::Ry(_) | GateType::Rz(_) => 1,
            GateType::Cnot | GateType::Cz | GateType::Swap => 2,
            GateType::Toffoli => 3,
            GateType::Barrier | GateType::Identity => 1,
            GateType::Custom(_) => 1, // Default, may vary based on definition
        }
    }

    /// Returns true if the gate requires control qubits.
    pub fn is_controlled(&self) -> bool {
        matches!(self, GateType::Cnot | GateType::Cz | GateType::Toffoli)
    }

    /// Returns the number of control qubits required.
    pub fn num_controls(&self) -> usize {
        match self {
            GateType::Cnot | GateType::Cz => 1,
            GateType::Toffoli => 2,
            _ => 0,
        }
    }

    /// Returns true if the gate has parameters (Rx, Ry, Rz).
    pub fn is_parameterized(&self) -> bool {
        matches!(self, GateType::Rx(_) | GateType::Ry(_) | GateType::Rz(_))
    }

    /// Returns the parameters for parameterized gates.
    pub fn parameters(&self) -> Vec<f64> {
        match self {
            GateType::Rx(theta) | GateType::Ry(theta) | GateType::Rz(theta) => vec![*theta],
            _ => Vec::new(),
        }
    }

    /// Returns a display-friendly name.
    pub fn display_name(&self) -> String {
        match self {
            GateType::H => "H".to_string(),
            GateType::X => "X".to_string(),
            GateType::Y => "Y".to_string(),
            GateType::Z => "Z".to_string(),
            GateType::S => "S".to_string(),
            GateType::T => "T".to_string(),
            GateType::Rx(theta) => format!("Rx({:.2})", theta),
            GateType::Ry(theta) => format!("Ry({:.2})", theta),
            GateType::Rz(theta) => format!("Rz({:.2})", theta),
            GateType::Cnot => "C+".to_string(),
            GateType::Cz => "CZ".to_string(),
            GateType::Swap => "Swap".to_string(),
            GateType::Toffoli => "Toffoli".to_string(),
            GateType::Barrier => "|".to_string(),
            GateType::Identity => "I".to_string(),
            GateType::Custom(name) => name.clone(),
        }
    }

    /// Returns the unitary matrix representation of the gate.
    ///
    /// For single-qubit gates, returns a 2x2 matrix.
    /// For two-qubit gates (CNOT, CZ, SWAP), returns a 4x4 matrix.
    /// For three-qubit gates (Toffoli), returns an 8x8 matrix.
    /// For Identity, returns the 2x2 identity matrix.
    /// For Barrier and Custom gates, returns None.
    pub fn matrix(&self) -> Option<GateMatrix> {
        match self {
            GateType::H => {
                let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;
                Some(GateMatrix::from_row_slice(2, 2, &[
                    C::new(sqrt2_inv, 0.0),
                    C::new(sqrt2_inv, 0.0),
                    C::new(sqrt2_inv, 0.0),
                    C::new(-sqrt2_inv, 0.0),
                ]))
            }
            GateType::X => Some(GateMatrix::from_row_slice(2, 2, &[
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
            ])),
            GateType::Y => Some(GateMatrix::from_row_slice(2, 2, &[
                C::new(0.0, 0.0),
                C::new(0.0, -1.0),
                C::new(0.0, 1.0),
                C::new(0.0, 0.0),
            ])),
            GateType::Z => Some(GateMatrix::from_row_slice(2, 2, &[
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(-1.0, 0.0),
            ])),
            GateType::S => Some(GateMatrix::from_row_slice(2, 2, &[
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 1.0),
            ])),
            GateType::T => {
                let phase = std::f64::consts::FRAC_PI_4.cos();
                Some(GateMatrix::from_row_slice(2, 2, &[
                    C::new(1.0, 0.0),
                    C::new(0.0, 0.0),
                    C::new(0.0, 0.0),
                    C::new(phase, phase),
                ]))
            }
            GateType::Rx(theta) => {
                let half_theta = theta / 2.0;
                let cos = half_theta.cos();
                let sin = half_theta.sin();
                Some(GateMatrix::from_row_slice(2, 2, &[
                    C::new(cos, 0.0),
                    C::new(0.0, -sin),
                    C::new(0.0, -sin),
                    C::new(cos, 0.0),
                ]))
            }
            GateType::Ry(theta) => {
                let half_theta = theta / 2.0;
                let cos = half_theta.cos();
                let sin = half_theta.sin();
                Some(GateMatrix::from_row_slice(2, 2, &[
                    C::new(cos, 0.0),
                    C::new(-sin, 0.0),
                    C::new(sin, 0.0),
                    C::new(cos, 0.0),
                ]))
            }
            GateType::Rz(theta) => {
                let half_theta = theta / 2.0;
                let neg_phase = C::from_polar(1.0, -half_theta);
                let pos_phase = C::from_polar(1.0, half_theta);
                Some(GateMatrix::from_row_slice(2, 2, &[
                    neg_phase,
                    C::new(0.0, 0.0),
                    C::new(0.0, 0.0),
                    pos_phase,
                ]))
            }
            GateType::Cnot => Some(GateMatrix::from_row_slice(4, 4, &[
                // |00⟩ -> |00⟩
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |01⟩ -> |01⟩
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |10⟩ -> |11⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                // |11⟩ -> |10⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
            ])),
            GateType::Cz => Some(GateMatrix::from_row_slice(4, 4, &[
                // |00⟩ -> |00⟩
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |01⟩ -> |01⟩
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |10⟩ -> |10⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                // |11⟩ -> -|11⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(-1.0, 0.0),
            ])),
            GateType::Swap => Some(GateMatrix::from_row_slice(4, 4, &[
                // |00⟩ -> |00⟩
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |01⟩ -> |10⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                // |10⟩ -> |01⟩
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                // |11⟩ -> |11⟩
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
            ])),
            GateType::Toffoli => {
                // 8x8 matrix for CCNOT: controlled-controlled-NOT
                // Flips the third qubit when first two qubits are both |1⟩
                let mut mat = GateMatrix::identity(8, 8);
                // Swap |110⟩ (index 6) and |111⟩ (index 7)
                mat[(6, 6)] = C::new(0.0, 0.0);
                mat[(6, 7)] = C::new(1.0, 0.0);
                mat[(7, 6)] = C::new(1.0, 0.0);
                mat[(7, 7)] = C::new(0.0, 0.0);
                Some(mat)
            }
            GateType::Identity => Some(GateMatrix::identity(2, 2)),
            GateType::Barrier | GateType::Custom(_) => None,
        }
    }

    /// Returns the 2x2 matrix for single-qubit gates.
    /// Returns None for multi-qubit gates or non-matrix gates.
    pub fn matrix2(&self) -> Option<GateMatrix2> {
        match self {
            GateType::H => {
                let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;
                Some(GateMatrix2::new(
                    C::new(sqrt2_inv, 0.0),
                    C::new(sqrt2_inv, 0.0),
                    C::new(sqrt2_inv, 0.0),
                    C::new(-sqrt2_inv, 0.0),
                ))
            }
            GateType::X => Some(GateMatrix2::new(
                C::new(0.0, 0.0),
                C::new(1.0, 0.0),
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
            )),
            GateType::Y => Some(GateMatrix2::new(
                C::new(0.0, 0.0),
                C::new(0.0, -1.0),
                C::new(0.0, 1.0),
                C::new(0.0, 0.0),
            )),
            GateType::Z => Some(GateMatrix2::new(
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(-1.0, 0.0),
            )),
            GateType::S => Some(GateMatrix2::new(
                C::new(1.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 0.0),
                C::new(0.0, 1.0),
            )),
            GateType::T => {
                let phase = std::f64::consts::FRAC_PI_4.cos();
                Some(GateMatrix2::new(
                    C::new(1.0, 0.0),
                    C::new(0.0, 0.0),
                    C::new(0.0, 0.0),
                    C::new(phase, phase),
                ))
            }
            GateType::Rx(theta) => {
                let half_theta = theta / 2.0;
                let cos = half_theta.cos();
                let sin = half_theta.sin();
                Some(GateMatrix2::new(
                    C::new(cos, 0.0),
                    C::new(0.0, -sin),
                    C::new(0.0, -sin),
                    C::new(cos, 0.0),
                ))
            }
            GateType::Ry(theta) => {
                let half_theta = theta / 2.0;
                let cos = half_theta.cos();
                let sin = half_theta.sin();
                Some(GateMatrix2::new(
                    C::new(cos, 0.0),
                    C::new(-sin, 0.0),
                    C::new(sin, 0.0),
                    C::new(cos, 0.0),
                ))
            }
            GateType::Rz(theta) => {
                let half_theta = theta / 2.0;
                let neg_phase = C::from_polar(1.0, -half_theta);
                let pos_phase = C::from_polar(1.0, half_theta);
                Some(GateMatrix2::new(
                    neg_phase,
                    C::new(0.0, 0.0),
                    C::new(0.0, 0.0),
                    pos_phase,
                ))
            }
            GateType::Identity => Some(GateMatrix2::identity()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gate_type_variants() {
        // Test that all basic variants exist and serialize correctly
        let gates = vec![
            GateType::H,
            GateType::X,
            GateType::Y,
            GateType::Z,
            GateType::S,
            GateType::T,
            GateType::Cnot,
            GateType::Cz,
            GateType::Swap,
            GateType::Toffoli,
            GateType::Barrier,
            GateType::Identity,
        ];

        for gate in gates {
            let serialized = serde_json::to_string(&gate).unwrap();
            let deserialized: GateType = serde_json::from_str(&serialized).unwrap();
            assert_eq!(gate, deserialized);
        }
    }

    #[test]
    fn test_parameterized_gates() {
        let rx = GateType::Rx(std::f64::consts::PI);
        assert!(rx.is_parameterized());
        assert_eq!(rx.parameters(), vec![std::f64::consts::PI]);
        assert_eq!(rx.display_name(), "Rx(3.14)");

        let ry = GateType::Ry(std::f64::consts::FRAC_PI_2);
        assert!(ry.is_parameterized());
        assert_eq!(ry.parameters(), vec![std::f64::consts::FRAC_PI_2]);

        let rz = GateType::Rz(0.5);
        assert!(rz.is_parameterized());
        assert_eq!(rz.parameters(), vec![0.5]);
    }

    #[test]
    fn test_custom_gate() {
        let custom = GateType::Custom("MyGate".to_string());
        assert_eq!(custom.display_name(), "MyGate");
        assert_eq!(custom.num_qubits(), 1);
        assert!(!custom.is_controlled());
    }

    #[test]
    fn test_controlled_gates() {
        assert!(GateType::Cnot.is_controlled());
        assert_eq!(GateType::Cnot.num_controls(), 1);
        assert_eq!(GateType::Cnot.num_qubits(), 2);

        assert!(GateType::Cz.is_controlled());
        assert_eq!(GateType::Cz.num_controls(), 1);

        assert!(GateType::Toffoli.is_controlled());
        assert_eq!(GateType::Toffoli.num_controls(), 2);
        assert_eq!(GateType::Toffoli.num_qubits(), 3);

        assert!(!GateType::H.is_controlled());
        assert!(!GateType::X.is_controlled());
        assert!(!GateType::Swap.is_controlled());
    }

    #[test]
    fn test_single_qubit_gates() {
        assert_eq!(GateType::H.num_qubits(), 1);
        assert_eq!(GateType::X.num_qubits(), 1);
        assert_eq!(GateType::Y.num_qubits(), 1);
        assert_eq!(GateType::Z.num_qubits(), 1);
        assert_eq!(GateType::S.num_qubits(), 1);
        assert_eq!(GateType::T.num_qubits(), 1);
    }

    // --- Matrix Tests ---

    #[test]
    fn test_hadamard_matrix() {
        let h = GateType::H;
        let mat = h.matrix().expect("H gate should have a matrix");

        assert_eq!(mat.nrows(), 2);
        assert_eq!(mat.ncols(), 2);

        let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;

        // H = 1/sqrt(2) * [[1, 1], [1, -1]]
        assert!((mat[(0, 0)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(0, 1)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(1, 0)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(1, 1)] - C::new(-sqrt2_inv, 0.0)).norm() < 1e-10);
    }

    #[test]
    fn test_pauli_x_matrix() {
        let x = GateType::X;
        let mat = x.matrix2().expect("X gate should have a 2x2 matrix");

        // X = [[0, 1], [1, 0]]
        assert_eq!(mat[(0, 0)], C::new(0.0, 0.0));
        assert_eq!(mat[(0, 1)], C::new(1.0, 0.0));
        assert_eq!(mat[(1, 0)], C::new(1.0, 0.0));
        assert_eq!(mat[(1, 1)], C::new(0.0, 0.0));
    }

    #[test]
    fn test_pauli_y_matrix() {
        let y = GateType::Y;
        let mat = y.matrix2().expect("Y gate should have a 2x2 matrix");

        // Y = [[0, -i], [i, 0]]
        assert_eq!(mat[(0, 0)], C::new(0.0, 0.0));
        assert_eq!(mat[(0, 1)], C::new(0.0, -1.0));
        assert_eq!(mat[(1, 0)], C::new(0.0, 1.0));
        assert_eq!(mat[(1, 1)], C::new(0.0, 0.0));
    }

    #[test]
    fn test_pauli_z_matrix() {
        let z = GateType::Z;
        let mat = z.matrix2().expect("Z gate should have a 2x2 matrix");

        // Z = [[1, 0], [0, -1]]
        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0));
        assert_eq!(mat[(0, 1)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 0)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 1)], C::new(-1.0, 0.0));
    }

    #[test]
    fn test_s_gate_matrix() {
        let s = GateType::S;
        let mat = s.matrix2().expect("S gate should have a 2x2 matrix");

        // S = [[1, 0], [0, i]]
        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0));
        assert_eq!(mat[(0, 1)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 0)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 1)], C::new(0.0, 1.0));
    }

    #[test]
    fn test_identity_matrix() {
        let i = GateType::Identity;
        let mat = i.matrix2().expect("Identity gate should have a 2x2 matrix");

        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0));
        assert_eq!(mat[(0, 1)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 0)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 1)], C::new(1.0, 0.0));
    }

    #[test]
    fn test_cnot_matrix() {
        let cnot = GateType::Cnot;
        let mat = cnot.matrix().expect("CNOT gate should have a matrix");

        assert_eq!(mat.nrows(), 4);
        assert_eq!(mat.ncols(), 4);

        // CNOT: |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |11⟩, |11⟩ → |10⟩
        // Matrix form (row-major):
        // [1, 0, 0, 0]
        // [0, 1, 0, 0]
        // [0, 0, 0, 1]
        // [0, 0, 1, 0]

        // |00⟩ (index 0) should stay |00⟩
        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0));

        // |01⟩ (index 1) should stay |01⟩
        assert_eq!(mat[(1, 1)], C::new(1.0, 0.0));

        // |10⟩ (index 2) should go to |11⟩ (index 3)
        assert_eq!(mat[(3, 2)], C::new(1.0, 0.0));

        // |11⟩ (index 3) should go to |10⟩ (index 2)
        assert_eq!(mat[(2, 3)], C::new(1.0, 0.0));
    }

    #[test]
    fn test_cz_matrix() {
        let cz = GateType::Cz;
        let mat = cz.matrix().expect("CZ gate should have a matrix");

        assert_eq!(mat.nrows(), 4);
        assert_eq!(mat.ncols(), 4);

        // CZ: applies Z when control is |1⟩
        // |00⟩ → |00⟩, |01⟩ → |01⟩, |10⟩ → |10⟩, |11⟩ → -|11⟩

        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0));
        assert_eq!(mat[(1, 1)], C::new(1.0, 0.0));
        assert_eq!(mat[(2, 2)], C::new(1.0, 0.0));
        assert_eq!(mat[(3, 3)], C::new(-1.0, 0.0));
    }

    #[test]
    fn test_swap_matrix() {
        let swap = GateType::Swap;
        let mat = swap.matrix().expect("SWAP gate should have a matrix");

        assert_eq!(mat.nrows(), 4);
        assert_eq!(mat.ncols(), 4);

        // SWAP: |00⟩ → |00⟩, |01⟩ → |10⟩, |10⟩ → |01⟩, |11⟩ → |11⟩

        assert_eq!(mat[(0, 0)], C::new(1.0, 0.0)); // |00⟩ stays
        assert_eq!(mat[(1, 2)], C::new(1.0, 0.0)); // |10⟩ → |01⟩ (row 1, col 2)
        assert_eq!(mat[(2, 1)], C::new(1.0, 0.0)); // |01⟩ → |10⟩ (row 2, col 1)
        assert_eq!(mat[(3, 3)], C::new(1.0, 0.0)); // |11⟩ stays
    }

    #[test]
    fn test_toffoli_matrix() {
        let toffoli = GateType::Toffoli;
        let mat = toffoli.matrix().expect("Toffoli gate should have a matrix");

        assert_eq!(mat.nrows(), 8);
        assert_eq!(mat.ncols(), 8);

        // Toffoli (CCNOT): flips third qubit when first two are |1⟩
        // |110⟩ (index 6) ↔ |111⟩ (index 7)

        // Check identity on non-affected states
        for i in 0..6 {
            assert_eq!(
                mat[(i, i)],
                C::new(1.0, 0.0),
                "State |{}⟩ should be unchanged",
                i
            );
        }

        // Check that |110⟩ (6) and |111⟩ (7) are swapped
        assert_eq!(mat[(6, 6)], C::new(0.0, 0.0));
        assert_eq!(mat[(6, 7)], C::new(1.0, 0.0));
        assert_eq!(mat[(7, 6)], C::new(1.0, 0.0));
        assert_eq!(mat[(7, 7)], C::new(0.0, 0.0));
    }

    #[test]
    fn test_rx_matrix() {
        // Rx(π) = [[cos(π/2), -i*sin(π/2)], [-i*sin(π/2), cos(π/2)]]
        //        = [[0, -i], [-i, 0]]
        let rx_pi = GateType::Rx(std::f64::consts::PI);
        let mat = rx_pi.matrix2().expect("Rx gate should have a matrix");

        assert!((mat[(0, 0)]).norm() < 1e-10); // cos(π/2) ≈ 0
        assert!((mat[(0, 1)] - C::new(0.0, -1.0)).norm() < 1e-10); // -i
        assert!((mat[(1, 0)] - C::new(0.0, -1.0)).norm() < 1e-10); // -i
        assert!((mat[(1, 1)]).norm() < 1e-10); // cos(π/2) ≈ 0
    }

    #[test]
    fn test_ry_matrix() {
        // Ry(π/2) = [[cos(π/4), -sin(π/4)], [sin(π/4), cos(π/4)]]
        //         = [[1/√2, -1/√2], [1/√2, 1/√2]]
        let ry_half_pi = GateType::Ry(std::f64::consts::FRAC_PI_2);
        let mat = ry_half_pi.matrix2().expect("Ry gate should have a matrix");

        let sqrt2_inv = 1.0 / std::f64::consts::SQRT_2;

        assert!((mat[(0, 0)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(0, 1)] - C::new(-sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(1, 0)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
        assert!((mat[(1, 1)] - C::new(sqrt2_inv, 0.0)).norm() < 1e-10);
    }

    #[test]
    fn test_rz_matrix() {
        // Rz(θ) = [[e^(-iθ/2), 0], [0, e^(iθ/2)]]
        let theta = 1.0;
        let rz = GateType::Rz(theta);
        let mat = rz.matrix2().expect("Rz gate should have a matrix");

        let expected_neg = C::from_polar(1.0, -theta / 2.0);
        let expected_pos = C::from_polar(1.0, theta / 2.0);

        assert!((mat[(0, 0)] - expected_neg).norm() < 1e-10);
        assert_eq!(mat[(0, 1)], C::new(0.0, 0.0));
        assert_eq!(mat[(1, 0)], C::new(0.0, 0.0));
        assert!((mat[(1, 1)] - expected_pos).norm() < 1e-10);
    }

    #[test]
    fn test_barrier_no_matrix() {
        let barrier = GateType::Barrier;
        assert!(barrier.matrix().is_none());
        assert!(barrier.matrix2().is_none());
    }

    #[test]
    fn test_custom_gate_no_matrix() {
        let custom = GateType::Custom("U3".to_string());
        assert!(custom.matrix().is_none());
        assert!(custom.matrix2().is_none());
    }

    #[test]
    fn test_multi_qubit_gates_no_matrix2() {
        // Multi-qubit gates should not have a matrix2() representation
        assert!(GateType::Cnot.matrix2().is_none());
        assert!(GateType::Cz.matrix2().is_none());
        assert!(GateType::Swap.matrix2().is_none());
        assert!(GateType::Toffoli.matrix2().is_none());
    }

    #[test]
    fn test_matrix_dimensions_match_num_qubits() {
        // Verify that matrix dimensions match the number of qubits
        let single_qubit_gates = vec![
            GateType::H,
            GateType::X,
            GateType::Y,
            GateType::Z,
            GateType::S,
            GateType::T,
            GateType::Rx(1.0),
            GateType::Ry(1.0),
            GateType::Rz(1.0),
            GateType::Identity,
        ];

        for gate in &single_qubit_gates {
            let mat = gate.matrix().expect(&format!("{:?} should have a matrix", gate));
            assert_eq!(
                mat.nrows(),
                2,
                "{:?} should have 2 rows (single-qubit)",
                gate
            );
            assert_eq!(
                mat.ncols(),
                2,
                "{:?} should have 2 cols (single-qubit)",
                gate
            );
        }

        let two_qubit_gates = vec![GateType::Cnot, GateType::Cz, GateType::Swap];

        for gate in &two_qubit_gates {
            let mat = gate.matrix().expect(&format!("{:?} should have a matrix", gate));
            assert_eq!(
                mat.nrows(),
                4,
                "{:?} should have 4 rows (two-qubit)",
                gate
            );
            assert_eq!(
                mat.ncols(),
                4,
                "{:?} should have 4 cols (two-qubit)",
                gate
            );
        }

        let mat = GateType::Toffoli.matrix().expect("Toffoli should have a matrix");
        assert_eq!(mat.nrows(), 8, "Toffoli should have 8 rows (three-qubit)");
        assert_eq!(mat.ncols(), 8, "Toffoli should have 8 cols (three-qubit)");
    }
}
