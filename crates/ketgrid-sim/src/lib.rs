//! KetGrid Sim — Quantum simulation engine (state vector, stabilizer, noise).

pub mod state_vector;
pub mod simulator;

pub use simulator::{SimulationError, SimulationResult, Simulator};
pub use state_vector::{EntanglementInfo, StateVectorSimulator, compute_entanglement_info};
