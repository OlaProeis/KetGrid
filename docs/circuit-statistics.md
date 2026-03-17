# Circuit Statistics Panel

Detailed circuit metrics and resource estimates for quantum circuit analysis.

## Overview

The circuit statistics panel provides comprehensive metrics about the current circuit, including gate counts by type, circuit depth, qubit usage, and resource estimates for simulation and hardware execution.

## Features

### Basic Circuit Info
- **Qubits**: Total number of qubit wires in the circuit
- **Total gates**: Count of all gates (excluding measurements)
- **Measurements**: Number of measurement operations
- **Circuit depth**: Maximum column index (sequential execution steps)

### Gate Counts by Type
- Breakdown of gates by their display name
- Sorted by count (descending), then alphabetically
- Shows all gate types including parameterized gates (Rx, Ry, Rz with their angles)

### Resource Estimates
- **State vector memory**: Estimated RAM required for simulation
  - Formula: `2^n * 16` bytes (Complex<f64>)
  - Visual warnings for high memory usage (>100MB yellow, >1GB red)
- **Hardware time estimate**: Rough execution time on typical quantum hardware
  - Single-qubit gates: ~50-100ns
  - Two-qubit gates: ~500ns
  - Three-qubit gates: ~2000ns
  - Measurements: ~1µs each

## Implementation

### Location
- **UI**: Right panel in the "State" sidebar, collapsible section
- **Source**: `crates/ketgrid-gui/src/stats_panel.rs`

### Key Types

```rust
/// Circuit statistics summary.
pub struct CircuitStats {
    pub num_qubits: usize,
    pub total_gates: usize,
    pub num_measurements: usize,
    pub depth: usize,
    pub gate_counts: HashMap<String, usize>,
    pub memory_bytes: usize,
    pub hardware_time_estimate_us: f64,
}

/// Circuit statistics panel UI.
pub struct StatsPanel;
```

### Computing Statistics

```rust
impl CircuitStats {
    pub fn from_circuit(circuit: &Circuit) -> Self {
        // Count gates by display name
        let mut gate_counts: HashMap<String, usize> = HashMap::new();
        for placed_gate in &circuit.gates {
            let name = placed_gate.gate.display_name();
            *gate_counts.entry(name).or_insert(0) += 1;
        }

        // Memory: 2^n * 16 bytes
        let memory_bytes = (1usize << num_qubits) * 16;

        // Hardware time based on gate type latencies
        let hardware_time_estimate_ns = /* ... */;

        Self { /* ... */ }
    }
}
```

## Dependencies

- Task 21 (Status bar with metrics) — the stats panel extends the basic metrics shown in the status bar with more detailed analysis

## Test Strategy

The panel includes tests verifying:
- Empty circuit statistics (zeros and empty counts)
- Grover 2-qubit circuit (depth=6, gates=12, H=6, X=4, C+=2)
- Circuits with measurements (separate count)
- Memory formatting (bytes → KB → MB → GB)
- Hardware time estimation (based on gate mix)
