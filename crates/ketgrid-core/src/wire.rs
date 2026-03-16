//! Qubit wire types and management.

use serde::{Deserialize, Serialize};

/// A qubit wire in the circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QubitWire {
    /// Wire index (0-based).
    pub id: usize,
    /// Display label (e.g., "|q₀⟩").
    pub label: String,
}

impl QubitWire {
    /// Creates a new qubit wire with the given id and label.
    pub fn new(id: usize, label: impl Into<String>) -> Self {
        Self {
            id,
            label: label.into(),
        }
    }

    /// Creates a new qubit wire with a default label based on its id.
    pub fn with_default_label(id: usize) -> Self {
        Self {
            id,
            label: format!("|q{}⟩", id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qubit_wire_new() {
        let wire = QubitWire::new(0, "Alice");
        assert_eq!(wire.id, 0);
        assert_eq!(wire.label, "Alice");
    }

    #[test]
    fn test_qubit_wire_default_label() {
        let wire = QubitWire::with_default_label(5);
        assert_eq!(wire.id, 5);
        assert_eq!(wire.label, "|q5⟩");
    }

    #[test]
    fn test_qubit_wire_serialization() {
        let wire = QubitWire::new(1, "|q1⟩");
        let serialized = serde_json::to_string(&wire).unwrap();
        let deserialized: QubitWire = serde_json::from_str(&serialized).unwrap();
        assert_eq!(wire, deserialized);
    }
}
