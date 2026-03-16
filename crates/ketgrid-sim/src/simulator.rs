//! Simulator interface and result types.

use ketgrid_core::Circuit;
use nalgebra::Complex;
use std::fmt;

/// Quantum simulation result containing state and metadata.
#[derive(Debug, Clone)]
pub struct SimulationResult {
    /// Final state vector as complex amplitudes (length = 2^num_qubits).
    pub state_vector: Option<Vec<Complex<f64>>>,
    /// |amplitude|² for each computational basis state.
    pub probabilities: Vec<f64>,
    /// Measurement outcomes (if applicable).
    pub measurements: Vec<u8>,
    /// Number of qubits in the simulation.
    pub num_qubits: usize,
}

/// Trait for quantum circuit simulators.
pub trait Simulator {
    /// Runs the simulator on the given circuit.
    fn run(&mut self, circuit: &Circuit) -> Result<SimulationResult, SimulationError>;
}

/// Errors that can occur during simulation.
#[derive(Debug, Clone)]
pub enum SimulationError {
    InvalidCircuit(String),
    SimulationFailed(String),
}

impl fmt::Display for SimulationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SimulationError::InvalidCircuit(msg) => write!(f, "Invalid circuit: {msg}"),
            SimulationError::SimulationFailed(msg) => write!(f, "Simulation failed: {msg}"),
        }
    }
}

impl std::error::Error for SimulationError {}
