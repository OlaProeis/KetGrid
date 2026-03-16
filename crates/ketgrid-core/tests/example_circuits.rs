//! Integration tests for example circuit files.
//!
//! Verifies that all example .ket.json files load correctly.
//! Simulation verification is done in the ketgrid-sim crate tests.

use ketgrid_core::{Circuit, GateType};
use std::path::PathBuf;

/// Returns the path to an example circuit file.
fn example_path(name: &str) -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("..");
    path.push("..");
    path.push("examples");
    path.push(format!("{}.ket.json", name));
    path
}

#[test]
fn test_bell_state_example_loads() {
    let circuit = Circuit::from_json_file(example_path("bell")).unwrap();

    assert_eq!(circuit.num_qubits(), 2);
    assert_eq!(circuit.gates.len(), 2);
    assert_eq!(circuit.measurements.len(), 2);

    // Verify gates
    assert_eq!(circuit.gates[0].gate, GateType::H);
    assert_eq!(circuit.gates[0].target_qubits, vec![0]);
    assert_eq!(circuit.gates[1].gate, GateType::Cnot);
    assert_eq!(circuit.gates[1].control_qubits, vec![0]);
    assert_eq!(circuit.gates[1].target_qubits, vec![1]);
}

#[test]
fn test_ghz_state_example_loads() {
    let circuit = Circuit::from_json_file(example_path("ghz")).unwrap();

    assert_eq!(circuit.num_qubits(), 3);
    assert_eq!(circuit.gates.len(), 3);
    assert_eq!(circuit.measurements.len(), 3);

    // Verify H on q0, then two CNOTs
    assert_eq!(circuit.gates[0].gate, GateType::H);
    assert_eq!(circuit.gates[1].gate, GateType::Cnot);
    assert_eq!(circuit.gates[2].gate, GateType::Cnot);
}

#[test]
fn test_deutsch_jozsa_example_loads() {
    let circuit = Circuit::from_json_file(example_path("deutsch-jozsa")).unwrap();

    assert_eq!(circuit.num_qubits(), 2);
    // X, H, H, CNOT, H = 5 gates
    assert_eq!(circuit.gates.len(), 5);
    assert_eq!(circuit.measurements.len(), 1);

    // Verify structure
    assert_eq!(circuit.gates[0].gate, GateType::X);
    assert_eq!(circuit.gates[0].target_qubits, vec![1]);
    assert_eq!(circuit.gates[3].gate, GateType::Cnot);
    assert_eq!(circuit.gates[3].control_qubits, vec![0]);
}

#[test]
fn test_teleportation_example_loads() {
    let circuit = Circuit::from_json_file(example_path("teleportation")).unwrap();

    assert_eq!(circuit.num_qubits(), 3);
    // H, CNOT, Barrier, CNOT, H, Barrier = 6 gates
    assert_eq!(circuit.gates.len(), 6);
    assert_eq!(circuit.measurements.len(), 2);

    // Verify Bell state preparation on q1, q2
    assert_eq!(circuit.gates[0].gate, GateType::H);
    assert_eq!(circuit.gates[0].target_qubits, vec![1]);
    assert_eq!(circuit.gates[1].gate, GateType::Cnot);
    assert_eq!(circuit.gates[1].control_qubits, vec![1]);
    assert_eq!(circuit.gates[1].target_qubits, vec![2]);

    // Verify Alice's operations
    assert_eq!(circuit.gates[3].gate, GateType::Cnot);
    assert_eq!(circuit.gates[3].control_qubits, vec![0]);
    assert_eq!(circuit.gates[3].target_qubits, vec![1]);
    assert_eq!(circuit.gates[4].gate, GateType::H);
    assert_eq!(circuit.gates[4].target_qubits, vec![0]);
}

#[test]
fn test_grover_2qubit_example_loads() {
    let circuit = Circuit::from_json_file(example_path("grover-2qubit")).unwrap();

    assert_eq!(circuit.num_qubits(), 2);
    // Multiple gates for Grover iteration
    assert!(circuit.gates.len() > 10);
    assert_eq!(circuit.measurements.len(), 2);

    // Verify initial superposition
    assert_eq!(circuit.gates[0].gate, GateType::H);
    assert_eq!(circuit.gates[1].gate, GateType::H);

    // Verify oracle uses CZ
    let has_cz = circuit.gates.iter().any(|g| g.gate == GateType::Cz);
    assert!(has_cz, "Grover circuit should have CZ gates for oracle and diffusion");
}

#[test]
fn test_all_examples_load() {
    let examples = vec!["bell", "ghz", "deutsch-jozsa", "teleportation", "grover-2qubit"];

    for name in examples {
        let path = example_path(name);
        assert!(
            path.exists(),
            "Example file should exist: {:?}",
            path
        );

        let circuit = Circuit::from_json_file(&path);
        assert!(
            circuit.is_ok(),
            "Example '{}' should load successfully: {:?}",
            name,
            circuit.err()
        );
    }
}
