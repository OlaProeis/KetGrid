//! JSON serialization for the native KetGrid circuit format.
//!
//! This module implements serialization and deserialization for the `.ket.json`
//! file format as specified in the PRD. The format includes:
//! - `ket_version`: Version string for format compatibility
//! - `name` and `description`: Metadata fields
//! - `qubits`: Number of qubits (validated against wire count)
//! - `gates`: Array of gate operations with type, targets, controls, and parameters
//! - `measurements`: Array of measurement operations

use crate::circuit::{Circuit, Measurement, PlacedGate};
use crate::gate::GateType;
use crate::wire::QubitWire;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::Path;

/// The current version of the Ket JSON format.
pub const CURRENT_KET_VERSION: &str = "0.1.0";

/// Error type for JSON format operations.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonError {
    /// IO error during file operation.
    Io(String),
    /// JSON parsing error.
    Parse(String),
    /// Version mismatch or unsupported version.
    InvalidVersion { expected: String, found: String },
    /// Missing required field in JSON.
    MissingField(String),
    /// Invalid qubit count (doesn't match wire count).
    InvalidQubitCount { expected: usize, found: usize },
    /// Invalid gate type string.
    InvalidGateType(String),
    /// Invalid gate structure.
    InvalidGate(String),
    /// Serialization error.
    Serialization(String),
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            JsonError::Io(msg) => write!(f, "IO error: {}", msg),
            JsonError::Parse(msg) => write!(f, "JSON parse error: {}", msg),
            JsonError::InvalidVersion { expected, found } => {
                write!(f, "Invalid version: expected {}, found {}", expected, found)
            }
            JsonError::MissingField(field) => write!(f, "Missing required field: {}", field),
            JsonError::InvalidQubitCount { expected, found } => {
                write!(f, "Qubit count mismatch: expected {}, found {}", expected, found)
            }
            JsonError::InvalidGateType(gate) => write!(f, "Invalid gate type: {}", gate),
            JsonError::InvalidGate(msg) => write!(f, "Invalid gate: {}", msg),
            JsonError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for JsonError {}

impl From<io::Error> for JsonError {
    fn from(err: io::Error) -> Self {
        JsonError::Io(err.to_string())
    }
}

impl From<serde_json::Error> for JsonError {
    fn from(err: serde_json::Error) -> Self {
        JsonError::Parse(err.to_string())
    }
}

/// JSON representation of a gate in the circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonGate {
    /// Gate type string (e.g., "H", "CNOT", "Rx").
    #[serde(rename = "type")]
    gate_type: String,
    /// Target qubit indices.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    targets: Vec<usize>,
    /// Control qubit indices (for controlled gates).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    controls: Vec<usize>,
    /// Column/time step position.
    column: usize,
    /// Parameters for parameterized gates (e.g., rotation angle).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    params: Vec<f64>,
}

/// JSON representation of a measurement in the circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct JsonMeasurement {
    /// Qubit being measured.
    qubit: usize,
    /// Column/time step position.
    column: usize,
}

/// The root JSON structure for a Ket circuit file.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
struct KetJson {
    /// Format version for compatibility checking.
    ket_version: String,
    /// Circuit name (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    /// Circuit description (optional).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    description: Option<String>,
    /// Number of qubits in the circuit.
    qubits: usize,
    /// Array of gates in the circuit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    gates: Vec<JsonGate>,
    /// Array of measurements in the circuit.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    measurements: Vec<JsonMeasurement>,
}

/// Parse a gate type string into a `GateType`.
fn parse_gate_type(type_str: &str, params: &[f64]) -> Result<GateType, JsonError> {
    match type_str {
        "H" => Ok(GateType::H),
        "X" => Ok(GateType::X),
        "Y" => Ok(GateType::Y),
        "Z" => Ok(GateType::Z),
        "S" => Ok(GateType::S),
        "T" => Ok(GateType::T),
        "Rx" => {
            if params.is_empty() {
                return Err(JsonError::InvalidGate(
                    "Rx gate requires a rotation parameter".to_string(),
                ));
            }
            Ok(GateType::Rx(params[0]))
        }
        "Ry" => {
            if params.is_empty() {
                return Err(JsonError::InvalidGate(
                    "Ry gate requires a rotation parameter".to_string(),
                ));
            }
            Ok(GateType::Ry(params[0]))
        }
        "Rz" => {
            if params.is_empty() {
                return Err(JsonError::InvalidGate(
                    "Rz gate requires a rotation parameter".to_string(),
                ));
            }
            Ok(GateType::Rz(params[0]))
        }
        "CNOT" | "Cnot" => Ok(GateType::Cnot),
        "CZ" | "Cz" => Ok(GateType::Cz),
        "SWAP" | "Swap" => Ok(GateType::Swap),
        "Toffoli" | "CCNOT" => Ok(GateType::Toffoli),
        "Barrier" => Ok(GateType::Barrier),
        "I" | "Identity" => Ok(GateType::Identity),
        custom => Ok(GateType::Custom(custom.to_string())),
    }
}

/// Convert a `GateType` to its string representation.
fn gate_type_to_string(gate: &GateType) -> String {
    match gate {
        GateType::H => "H".to_string(),
        GateType::X => "X".to_string(),
        GateType::Y => "Y".to_string(),
        GateType::Z => "Z".to_string(),
        GateType::S => "S".to_string(),
        GateType::T => "T".to_string(),
        GateType::Rx(_) => "Rx".to_string(),
        GateType::Ry(_) => "Ry".to_string(),
        GateType::Rz(_) => "Rz".to_string(),
        GateType::Cnot => "CNOT".to_string(),
        GateType::Cz => "CZ".to_string(),
        GateType::Swap => "SWAP".to_string(),
        GateType::Toffoli => "Toffoli".to_string(),
        GateType::Barrier => "Barrier".to_string(),
        GateType::Identity => "I".to_string(),
        GateType::Custom(name) => name.clone(),
    }
}

/// Get parameters from a `GateType`.
fn gate_params(gate: &GateType) -> Vec<f64> {
    match gate {
        GateType::Rx(theta) => vec![*theta],
        GateType::Ry(theta) => vec![*theta],
        GateType::Rz(theta) => vec![*theta],
        _ => vec![],
    }
}

/// Convert a `PlacedGate` to a `JsonGate`.
fn placed_gate_to_json(gate: &PlacedGate) -> JsonGate {
    JsonGate {
        gate_type: gate_type_to_string(&gate.gate),
        targets: gate.target_qubits.clone(),
        controls: gate.control_qubits.clone(),
        column: gate.column,
        params: if gate.parameters.is_empty() {
            gate_params(&gate.gate)
        } else {
            gate.parameters.clone()
        },
    }
}

/// Convert a `Measurement` to a `JsonMeasurement`.
fn measurement_to_json(measurement: &Measurement) -> JsonMeasurement {
    JsonMeasurement {
        qubit: measurement.qubit_id,
        column: measurement.column,
    }
}

/// Convert a `JsonGate` to a `PlacedGate`.
fn json_gate_to_placed(json_gate: &JsonGate) -> Result<PlacedGate, JsonError> {
    let gate_type = parse_gate_type(&json_gate.gate_type, &json_gate.params)?;

    Ok(PlacedGate {
        gate: gate_type,
        target_qubits: json_gate.targets.clone(),
        control_qubits: json_gate.controls.clone(),
        column: json_gate.column,
        parameters: json_gate.params.clone(),
    })
}

/// Convert a `JsonMeasurement` to a `Measurement`.
fn json_measurement_to_measurement(json: &JsonMeasurement) -> Measurement {
    Measurement {
        qubit_id: json.qubit,
        column: json.column,
    }
}

/// Deserialize a circuit from a JSON string.
pub fn circuit_from_json(json_str: &str) -> Result<Circuit, JsonError> {
    let ket_json: KetJson = serde_json::from_str(json_str)?;

    // Validate version
    if ket_json.ket_version != CURRENT_KET_VERSION {
        return Err(JsonError::InvalidVersion {
            expected: CURRENT_KET_VERSION.to_string(),
            found: ket_json.ket_version,
        });
    }

    // Create qubit wires
    let qubits: Vec<QubitWire> = (0..ket_json.qubits)
        .map(QubitWire::with_default_label)
        .collect();

    // Convert gates
    let mut gates = Vec::with_capacity(ket_json.gates.len());
    for json_gate in &ket_json.gates {
        gates.push(json_gate_to_placed(json_gate)?);
    }

    // Convert measurements
    let measurements: Vec<Measurement> = ket_json
        .measurements
        .iter()
        .map(json_measurement_to_measurement)
        .collect();

    Ok(Circuit {
        qubits,
        gates,
        measurements,
    })
}

/// Serialize a circuit to a JSON string.
pub fn circuit_to_json(circuit: &Circuit) -> Result<String, JsonError> {
    let ket_json = KetJson {
        ket_version: CURRENT_KET_VERSION.to_string(),
        name: None,
        description: None,
        qubits: circuit.num_qubits(),
        gates: circuit.gates.iter().map(placed_gate_to_json).collect(),
        measurements: circuit.measurements.iter().map(measurement_to_json).collect(),
    };

    serde_json::to_string_pretty(&ket_json)
        .map_err(|e| JsonError::Serialization(e.to_string()))
}

/// Serialize a circuit to a JSON string with metadata.
pub fn circuit_to_json_with_metadata(
    circuit: &Circuit,
    name: Option<String>,
    description: Option<String>,
) -> Result<String, JsonError> {
    let ket_json = KetJson {
        ket_version: CURRENT_KET_VERSION.to_string(),
        name,
        description,
        qubits: circuit.num_qubits(),
        gates: circuit.gates.iter().map(placed_gate_to_json).collect(),
        measurements: circuit.measurements.iter().map(measurement_to_json).collect(),
    };

    serde_json::to_string_pretty(&ket_json)
        .map_err(|e| JsonError::Serialization(e.to_string()))
}

impl Circuit {
    /// Load a circuit from a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path to the `.ket.json` file.
    ///
    /// # Returns
    /// * `Ok(Circuit)` - The loaded circuit.
    /// * `Err(JsonError)` - If the file cannot be read or the JSON is invalid.
    ///
    /// # Example
    /// ```no_run
    /// use ketgrid_core::Circuit;
    ///
    /// let circuit = Circuit::from_json_file("bell_state.ket.json").unwrap();
    /// ```
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self, JsonError> {
        let contents = fs::read_to_string(path)?;
        circuit_from_json(&contents)
    }

    /// Save a circuit to a JSON file.
    ///
    /// # Arguments
    /// * `path` - Path where the `.ket.json` file should be saved.
    ///
    /// # Returns
    /// * `Ok(())` - If the file was saved successfully.
    /// * `Err(JsonError)` - If the circuit cannot be serialized or the file cannot be written.
    ///
    /// # Example
    /// ```no_run
    /// use ketgrid_core::Circuit;
    ///
    /// let circuit = Circuit::new(2);
    /// circuit.to_json_file("circuit.ket.json").unwrap();
    /// ```
    pub fn to_json_file<P: AsRef<Path>>(&self, path: P) -> Result<(), JsonError> {
        let json_str = circuit_to_json(self)?;
        fs::write(path, json_str)?;
        Ok(())
    }

    /// Save a circuit to a JSON file with metadata.
    ///
    /// # Arguments
    /// * `path` - Path where the `.ket.json` file should be saved.
    /// * `name` - Optional circuit name.
    /// * `description` - Optional circuit description.
    ///
    /// # Returns
    /// * `Ok(())` - If the file was saved successfully.
    /// * `Err(JsonError)` - If the circuit cannot be serialized or the file cannot be written.
    pub fn to_json_file_with_metadata<P: AsRef<Path>>(
        &self,
        path: P,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<(), JsonError> {
        let json_str = circuit_to_json_with_metadata(self, name, description)?;
        fs::write(path, json_str)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gate::GateType;

    #[test]
    fn test_parse_gate_type() {
        assert_eq!(parse_gate_type("H", &[]).unwrap(), GateType::H);
        assert_eq!(parse_gate_type("X", &[]).unwrap(), GateType::X);
        assert_eq!(parse_gate_type("Y", &[]).unwrap(), GateType::Y);
        assert_eq!(parse_gate_type("Z", &[]).unwrap(), GateType::Z);
        assert_eq!(parse_gate_type("S", &[]).unwrap(), GateType::S);
        assert_eq!(parse_gate_type("T", &[]).unwrap(), GateType::T);
        assert_eq!(parse_gate_type("CNOT", &[]).unwrap(), GateType::Cnot);
        assert_eq!(parse_gate_type("CZ", &[]).unwrap(), GateType::Cz);
        assert_eq!(parse_gate_type("SWAP", &[]).unwrap(), GateType::Swap);
        assert_eq!(parse_gate_type("Toffoli", &[]).unwrap(), GateType::Toffoli);
        assert_eq!(parse_gate_type("Barrier", &[]).unwrap(), GateType::Barrier);
        assert_eq!(parse_gate_type("I", &[]).unwrap(), GateType::Identity);
        assert_eq!(
            parse_gate_type("CustomGate", &[]).unwrap(),
            GateType::Custom("CustomGate".to_string())
        );

        // Parameterized gates
        assert_eq!(
            parse_gate_type("Rx", &[std::f64::consts::PI]).unwrap(),
            GateType::Rx(std::f64::consts::PI)
        );
        assert_eq!(
            parse_gate_type("Ry", &[std::f64::consts::FRAC_PI_2]).unwrap(),
            GateType::Ry(std::f64::consts::FRAC_PI_2)
        );
        assert_eq!(parse_gate_type("Rz", &[1.0]).unwrap(), GateType::Rz(1.0));
    }

    #[test]
    fn test_parse_gate_type_missing_params() {
        let result = parse_gate_type("Rx", &[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), JsonError::InvalidGate(_)));
    }

    #[test]
    fn test_gate_type_to_string() {
        assert_eq!(gate_type_to_string(&GateType::H), "H");
        assert_eq!(gate_type_to_string(&GateType::X), "X");
        assert_eq!(gate_type_to_string(&GateType::Cnot), "CNOT");
        assert_eq!(gate_type_to_string(&GateType::Swap), "SWAP");
        assert_eq!(gate_type_to_string(&GateType::Toffoli), "Toffoli");
        assert_eq!(
            gate_type_to_string(&GateType::Custom("MyGate".to_string())),
            "MyGate"
        );
        assert_eq!(gate_type_to_string(&GateType::Rx(1.0)), "Rx");
    }

    #[test]
    fn test_gate_params() {
        assert!(gate_params(&GateType::H).is_empty());
        assert_eq!(gate_params(&GateType::Rx(1.5)), vec![1.5]);
        assert_eq!(gate_params(&GateType::Ry(2.0)), vec![2.0]);
        assert_eq!(gate_params(&GateType::Rz(3.0)), vec![3.0]);
    }

    #[test]
    fn test_circuit_to_json() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        let json_str = circuit_to_json(&circuit).unwrap();
        assert!(json_str.contains("ket_version"));
        assert!(json_str.contains("0.1.0"));
        assert!(json_str.contains("\"type\": \"H\""));
        assert!(json_str.contains("\"type\": \"CNOT\""));
        assert!(json_str.contains("\"qubits\": 2"));
    }

    #[test]
    fn test_circuit_from_json() {
        let json = r#"{
            "ket_version": "0.1.0",
            "qubits": 2,
            "gates": [
                { "type": "H", "targets": [0], "column": 0 },
                { "type": "CNOT", "controls": [0], "targets": [1], "column": 1 }
            ],
            "measurements": []
        }"#;

        let circuit = circuit_from_json(json).unwrap();
        assert_eq!(circuit.num_qubits(), 2);
        assert_eq!(circuit.gates.len(), 2);
        assert_eq!(circuit.gates[0].gate, GateType::H);
        assert_eq!(circuit.gates[0].target_qubits, vec![0]);
        assert_eq!(circuit.gates[1].gate, GateType::Cnot);
        assert_eq!(circuit.gates[1].control_qubits, vec![0]);
        assert_eq!(circuit.gates[1].target_qubits, vec![1]);
    }

    #[test]
    fn test_circuit_roundtrip() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();
        circuit.add_gate(GateType::Rx(1.5), vec![2], vec![], 2).unwrap();
        circuit.add_measurement(0, 3).unwrap();

        let json_str = circuit_to_json(&circuit).unwrap();
        let loaded = circuit_from_json(&json_str).unwrap();

        assert_eq!(circuit.num_qubits(), loaded.num_qubits());
        assert_eq!(circuit.gates.len(), loaded.gates.len());
        assert_eq!(circuit.measurements.len(), loaded.measurements.len());

        for (orig, loaded) in circuit.gates.iter().zip(&loaded.gates) {
            assert_eq!(orig.gate, loaded.gate);
            assert_eq!(orig.target_qubits, loaded.target_qubits);
            assert_eq!(orig.control_qubits, loaded.control_qubits);
            assert_eq!(orig.column, loaded.column);
        }
    }

    #[test]
    fn test_invalid_version() {
        let json = r#"{
            "ket_version": "0.2.0",
            "qubits": 2,
            "gates": [],
            "measurements": []
        }"#;

        let result = circuit_from_json(json);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            JsonError::InvalidVersion { expected, found } if expected == "0.1.0" && found == "0.2.0"
        ));
    }

    #[test]
    fn test_missing_version() {
        let json = r#"{
            "qubits": 2,
            "gates": [],
            "measurements": []
        }"#;

        let result = circuit_from_json(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_json() {
        let result = circuit_from_json("not valid json");
        assert!(result.is_err());
    }

    #[test]
    fn test_parameterized_gate_roundtrip() {
        let mut circuit = Circuit::new(2);
        circuit
            .add_gate(GateType::Rx(std::f64::consts::PI), vec![0], vec![], 0)
            .unwrap();
        circuit
            .add_gate(GateType::Ry(std::f64::consts::FRAC_PI_2), vec![1], vec![], 1)
            .unwrap();
        circuit.add_gate(GateType::Rz(0.5), vec![0], vec![], 2).unwrap();

        let json_str = circuit_to_json(&circuit).unwrap();
        let loaded = circuit_from_json(&json_str).unwrap();

        assert_eq!(loaded.gates[0].gate, GateType::Rx(std::f64::consts::PI));
        assert_eq!(
            loaded.gates[1].gate,
            GateType::Ry(std::f64::consts::FRAC_PI_2)
        );
        assert_eq!(loaded.gates[2].gate, GateType::Rz(0.5));
    }

    #[test]
    fn test_toffoli_gate() {
        let json = r#"{
            "ket_version": "0.1.0",
            "qubits": 3,
            "gates": [
                { "type": "Toffoli", "controls": [0, 1], "targets": [2], "column": 0 }
            ],
            "measurements": []
        }"#;

        let circuit = circuit_from_json(json).unwrap();
        assert_eq!(circuit.gates[0].gate, GateType::Toffoli);
        assert_eq!(circuit.gates[0].control_qubits, vec![0, 1]);
        assert_eq!(circuit.gates[0].target_qubits, vec![2]);
    }

    #[test]
    fn test_swap_gate() {
        let json = r#"{
            "ket_version": "0.1.0",
            "qubits": 2,
            "gates": [
                { "type": "SWAP", "targets": [0, 1], "column": 0 }
            ],
            "measurements": []
        }"#;

        let circuit = circuit_from_json(json).unwrap();
        assert_eq!(circuit.gates[0].gate, GateType::Swap);
        assert_eq!(circuit.gates[0].target_qubits, vec![0, 1]);
    }

    #[test]
    fn test_cz_gate() {
        let json = r#"{
            "ket_version": "0.1.0",
            "qubits": 2,
            "gates": [
                { "type": "CZ", "controls": [0], "targets": [1], "column": 0 }
            ],
            "measurements": []
        }"#;

        let circuit = circuit_from_json(json).unwrap();
        assert_eq!(circuit.gates[0].gate, GateType::Cz);
    }

    #[test]
    fn test_with_metadata() {
        let circuit = Circuit::new(2);
        let json_str = circuit_to_json_with_metadata(
            &circuit,
            Some("Bell State".to_string()),
            Some("Creates an entangled Bell state".to_string()),
        )
        .unwrap();

        assert!(json_str.contains("Bell State"));
        assert!(json_str.contains("entangled Bell state"));
        assert!(json_str.contains("\"name\""));
        assert!(json_str.contains("\"description\""));
    }

    #[test]
    fn test_parse_with_metadata() {
        let json = r#"{
            "ket_version": "0.1.0",
            "name": "Test Circuit",
            "description": "A test circuit",
            "qubits": 1,
            "gates": [{ "type": "H", "targets": [0], "column": 0 }],
            "measurements": []
        }"#;

        let circuit = circuit_from_json(json).unwrap();
        assert_eq!(circuit.num_qubits(), 1);
        assert_eq!(circuit.gates[0].gate, GateType::H);
    }
}
