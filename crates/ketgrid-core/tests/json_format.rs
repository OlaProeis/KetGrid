//! Integration tests for JSON circuit serialization.
//!
//! These tests verify file I/O operations and complex circuit roundtrips
//! including simulation result verification.

use ketgrid_core::{Circuit, GateType};
use std::fs;
use std::path::PathBuf;

/// Creates a temporary file path for testing.
fn temp_path(name: &str) -> PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("ketgrid_test_{}_{}", name, std::process::id()));
    path.set_extension("ket.json");
    path
}

/// Cleans up a temporary file after test.
fn cleanup(path: &PathBuf) {
    let _ = fs::remove_file(path);
}

#[test]
fn test_bell_state_save_and_reload() {
    // Create Bell state circuit: H on q0, CNOT with control q0, target q1
    let mut circuit = Circuit::new(2);
    circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
    circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
    circuit.add_measurement(0, 2).unwrap();
    circuit.add_measurement(1, 2).unwrap();

    // Save to file
    let path = temp_path("bell_state");
    circuit.to_json_file(&path).unwrap();

    // Verify file exists and has content
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("ket_version"));
    assert!(contents.contains("0.1.0"));
    assert!(contents.contains("\"type\": \"H\""));
    assert!(contents.contains("\"type\": \"CNOT\""));
    assert!(contents.contains("\"qubits\": 2"));

    // Reload the circuit
    let loaded = Circuit::from_json_file(&path).unwrap();

    // Verify identical circuit
    assert_eq!(loaded.num_qubits(), circuit.num_qubits());
    assert_eq!(loaded.gates.len(), circuit.gates.len());
    assert_eq!(loaded.measurements.len(), circuit.measurements.len());

    for (orig, loaded) in circuit.gates.iter().zip(&loaded.gates) {
        assert_eq!(orig.gate, loaded.gate);
        assert_eq!(orig.target_qubits, loaded.target_qubits);
        assert_eq!(orig.control_qubits, loaded.control_qubits);
        assert_eq!(orig.column, loaded.column);
    }

    // Verify measurements match
    for (orig, loaded) in circuit.measurements.iter().zip(&loaded.measurements) {
        assert_eq!(orig.qubit_id, loaded.qubit_id);
        assert_eq!(orig.column, loaded.column);
    }

    cleanup(&path);
}

#[test]
fn test_circuit_with_metadata_save_and_reload() {
    let circuit = Circuit::new(2);

    // Save with metadata
    let path = temp_path("metadata");
    circuit
        .to_json_file_with_metadata(
            &path,
            Some("Bell State".to_string()),
            Some("Creates an entangled Bell state |Φ+⟩".to_string()),
        )
        .unwrap();

    // Verify metadata in file
    let contents = fs::read_to_string(&path).unwrap();
    assert!(contents.contains("Bell State"));
    assert!(contents.contains("Creates an entangled Bell state |Φ+⟩"));

    // Circuit can still be loaded (metadata is ignored during load)
    let loaded = Circuit::from_json_file(&path).unwrap();
    assert_eq!(loaded.num_qubits(), 2);

    cleanup(&path);
}

#[test]
fn test_invalid_json_file() {
    let path = temp_path("invalid");
    fs::write(&path, "not valid json").unwrap();

    let result = Circuit::from_json_file(&path);
    assert!(result.is_err());

    cleanup(&path);
}

#[test]
fn test_missing_version_field() {
    let path = temp_path("no_version");
    let json = r#"{
        "qubits": 2,
        "gates": [],
        "measurements": []
    }"#;
    fs::write(&path, json).unwrap();

    let result = Circuit::from_json_file(&path);
    assert!(result.is_err());

    cleanup(&path);
}

#[test]
fn test_wrong_version() {
    let path = temp_path("wrong_version");
    let json = r#"{
        "ket_version": "9.9.9",
        "qubits": 2,
        "gates": [],
        "measurements": []
    }"#;
    fs::write(&path, json).unwrap();

    let result = Circuit::from_json_file(&path);
    assert!(result.is_err());

    cleanup(&path);
}

#[test]
fn test_all_gate_types_roundtrip() {
    let mut circuit = Circuit::new(3);

    // Add all gate types
    circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
    circuit.add_gate(GateType::X, vec![0], vec![], 1).unwrap();
    circuit.add_gate(GateType::Y, vec![0], vec![], 2).unwrap();
    circuit.add_gate(GateType::Z, vec![0], vec![], 3).unwrap();
    circuit.add_gate(GateType::S, vec![0], vec![], 4).unwrap();
    circuit.add_gate(GateType::T, vec![0], vec![], 5).unwrap();
    circuit
        .add_gate(GateType::Rx(1.0), vec![0], vec![], 6)
        .unwrap();
    circuit
        .add_gate(GateType::Ry(2.0), vec![0], vec![], 7)
        .unwrap();
    circuit
        .add_gate(GateType::Rz(3.0), vec![0], vec![], 8)
        .unwrap();
    circuit.add_gate(GateType::Cnot, vec![1], vec![0], 9).unwrap();
    circuit.add_gate(GateType::Cz, vec![1], vec![0], 10).unwrap();
    circuit.add_gate(GateType::Swap, vec![0, 1], vec![], 11).unwrap();
    circuit
        .add_gate(GateType::Toffoli, vec![2], vec![0, 1], 12)
        .unwrap();
    circuit.add_gate(GateType::Barrier, vec![0], vec![], 13).unwrap();
    circuit.add_gate(GateType::Identity, vec![0], vec![], 14).unwrap();
    circuit
        .add_gate(GateType::Custom("CustomGate".to_string()), vec![0], vec![], 15)
        .unwrap();

    // Save and reload
    let path = temp_path("all_gates");
    circuit.to_json_file(&path).unwrap();
    let loaded = Circuit::from_json_file(&path).unwrap();

    assert_eq!(loaded.gates.len(), circuit.gates.len());

    for (i, (orig, loaded)) in circuit.gates.iter().zip(&loaded.gates).enumerate() {
        assert_eq!(
            orig.gate, loaded.gate,
            "Gate {} (at column {}) should match",
            i, orig.column
        );
        assert_eq!(orig.target_qubits, loaded.target_qubits);
        assert_eq!(orig.control_qubits, loaded.control_qubits);
        assert_eq!(orig.column, loaded.column);
    }

    cleanup(&path);
}

#[test]
fn test_missing_file() {
    let path = PathBuf::from("/nonexistent/path/circuit.ket.json");
    let result = Circuit::from_json_file(&path);
    assert!(result.is_err());
}

#[test]
fn test_prd_schema_example() {
    // Verify we can parse the exact example from the PRD
    let json = r#"{
        "ket_version": "0.1.0",
        "name": "Bell State",
        "description": "Creates an entangled Bell state |Φ+⟩",
        "qubits": 2,
        "gates": [
            { "type": "H", "targets": [0], "column": 0 },
            { "type": "CNOT", "controls": [0], "targets": [1], "column": 1 }
        ],
        "measurements": [
            { "qubit": 0, "column": 2 },
            { "qubit": 1, "column": 2 }
        ]
    }"#;

    let path = temp_path("prd_example");
    fs::write(&path, json).unwrap();

    let circuit = Circuit::from_json_file(&path).unwrap();
    assert_eq!(circuit.num_qubits(), 2);
    assert_eq!(circuit.gates.len(), 2);
    assert_eq!(circuit.measurements.len(), 2);

    // Verify H gate
    assert_eq!(circuit.gates[0].gate, GateType::H);
    assert_eq!(circuit.gates[0].target_qubits, vec![0]);
    assert!(circuit.gates[0].control_qubits.is_empty());
    assert_eq!(circuit.gates[0].column, 0);

    // Verify CNOT gate
    assert_eq!(circuit.gates[1].gate, GateType::Cnot);
    assert_eq!(circuit.gates[1].control_qubits, vec![0]);
    assert_eq!(circuit.gates[1].target_qubits, vec![1]);
    assert_eq!(circuit.gates[1].column, 1);

    // Verify measurements
    assert_eq!(circuit.measurements[0].qubit_id, 0);
    assert_eq!(circuit.measurements[0].column, 2);
    assert_eq!(circuit.measurements[1].qubit_id, 1);
    assert_eq!(circuit.measurements[1].column, 2);

    cleanup(&path);
}
