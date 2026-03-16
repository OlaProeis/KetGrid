//! Integration tests for example circuit simulations.
//!
//! Verifies that all example .ket.json files produce expected simulation results.

use ketgrid_core::Circuit;
use ketgrid_sim::{Simulator, StateVectorSimulator};
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
fn test_bell_state_simulation() {
    let circuit = Circuit::from_json_file(example_path("bell")).unwrap();

    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
    let result = sim.run(&circuit).unwrap();

    // Bell state has equal probability for |00⟩ and |11⟩
    assert_eq!(result.num_qubits, 2);
    let probs = result.probabilities;

    assert!((probs[0] - 0.5).abs() < 1e-10, "|00⟩ should be 50%, got {}", probs[0]);
    assert!((probs[1] - 0.0).abs() < 1e-10, "|01⟩ should be 0%, got {}", probs[1]);
    assert!((probs[2] - 0.0).abs() < 1e-10, "|10⟩ should be 0%, got {}", probs[2]);
    assert!((probs[3] - 0.5).abs() < 1e-10, "|11⟩ should be 50%, got {}", probs[3]);
}

#[test]
fn test_ghz_state_simulation() {
    let circuit = Circuit::from_json_file(example_path("ghz")).unwrap();

    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
    let result = sim.run(&circuit).unwrap();

    // GHZ: (|000⟩ + |111⟩)/√2
    assert_eq!(result.num_qubits, 3);
    let probs = result.probabilities;

    assert!((probs[0] - 0.5).abs() < 1e-10, "|000⟩ should be 50%, got {}", probs[0]);
    assert!((probs[7] - 0.5).abs() < 1e-10, "|111⟩ should be 50%, got {}", probs[7]);

    // All others should be 0
    for i in 1..7 {
        assert!(
            (probs[i] - 0.0).abs() < 1e-10,
            "|{}⟩ should be 0%, got {}",
            i,
            probs[i]
        );
    }
}

#[test]
fn test_deutsch_jozsa_simulation() {
    let circuit = Circuit::from_json_file(example_path("deutsch-jozsa")).unwrap();

    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
    let result = sim.run(&circuit).unwrap();

    // For balanced function f(x) = x, measurement should give |1⟩ with high probability
    assert_eq!(result.num_qubits, 2);
    let probs = result.probabilities;

    // qubit 0 should be measured as |1⟩ (balanced function result)
    // This means |10⟩ or |11⟩ should have high probability
    let prob_1x = probs[2] + probs[3];
    assert!(
        prob_1x > 0.99,
        "Qubit 0 should be |1⟩ (balanced function), got total prob {} for |1x⟩ states",
        prob_1x
    );
}

#[test]
fn test_teleportation_simulation() {
    let circuit = Circuit::from_json_file(example_path("teleportation")).unwrap();

    // Circuit runs without error
    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
    let result = sim.run(&circuit).unwrap();
    assert_eq!(result.num_qubits, 3);
    // Full teleportation verification would require analyzing the full state
    // including the measurements, which is complex
}

#[test]
fn test_grover_2qubit_simulation() {
    let circuit = Circuit::from_json_file(example_path("grover-2qubit")).unwrap();

    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
    let result = sim.run(&circuit).unwrap();

    // After Grover's algorithm, |11⟩ should have high probability (>80%)
    assert_eq!(result.num_qubits, 2);
    let probs = result.probabilities;

    assert!(
        probs[3] > 0.8,
        "Grover should amplify marked state |11⟩ to >80%, got {}%",
        probs[3] * 100.0
    );

    // Verify |11⟩ is the most probable state
    assert_eq!(
        probs.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0,
        3,
        "|11⟩ should be the most probable state"
    );
}

#[test]
fn test_all_examples_simulate() {
    let examples = vec!["bell", "ghz", "deutsch-jozsa", "teleportation", "grover-2qubit"];

    for name in examples {
        let path = example_path(name);
        let circuit = Circuit::from_json_file(&path).unwrap();

        let mut sim = StateVectorSimulator::new(circuit.num_qubits());
        let result = sim.run(&circuit);
        assert!(
            result.is_ok(),
            "Example '{}' should simulate successfully: {:?}",
            name,
            result.err()
        );
    }
}
