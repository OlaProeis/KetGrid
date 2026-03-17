//! Circuit statistics panel showing gate counts, depth, and resource estimates.

use ketgrid_core::circuit::Circuit;
use ketgrid_core::gate::GateType;
use std::collections::HashMap;

/// Circuit statistics summary.
#[derive(Debug, Clone, Default)]
pub struct CircuitStats {
    /// Number of qubits in the circuit.
    pub num_qubits: usize,
    /// Total number of gates (excluding measurements).
    pub total_gates: usize,
    /// Number of measurements.
    pub num_measurements: usize,
    /// Circuit depth (max column index).
    pub depth: usize,
    /// Gate counts by type.
    pub gate_counts: HashMap<String, usize>,
    /// Estimated memory for state vector simulation.
    pub memory_bytes: usize,
    /// Estimated execution time on typical quantum hardware (microseconds).
    pub hardware_time_estimate_us: f64,
}

impl CircuitStats {
    /// Computes statistics from a circuit.
    pub fn from_circuit(circuit: &Circuit) -> Self {
        let num_qubits = circuit.num_qubits();
        let total_gates = circuit.gates.len();
        let num_measurements = circuit.measurements.len();
        let depth = circuit.max_column();

        // Count gates by type
        let mut gate_counts: HashMap<String, usize> = HashMap::new();
        for placed_gate in &circuit.gates {
            let name = placed_gate.gate.display_name();
            *gate_counts.entry(name).or_insert(0) += 1;
        }

        // Memory estimate: 2^n * 16 bytes (Complex<f64>)
        let memory_bytes = if num_qubits >= 64 {
            usize::MAX
        } else {
            (1usize << num_qubits) * 16
        };

        // Hardware time estimate: rough approximation
        // Single-qubit gates: ~50ns, two-qubit: ~500ns, three-qubit: ~2000ns
        let mut hardware_time_estimate_ns: f64 = 0.0;
        for placed_gate in &circuit.gates {
            hardware_time_estimate_ns += match placed_gate.gate {
                GateType::H | GateType::X | GateType::Y | GateType::Z | GateType::S | GateType::T => 50.0,
                GateType::Rx(_) | GateType::Ry(_) | GateType::Rz(_) => 100.0,
                GateType::Cnot | GateType::Cz | GateType::Swap => 500.0,
                GateType::Toffoli => 2000.0,
                GateType::Barrier | GateType::Identity => 0.0,
                GateType::Custom(_) => 200.0,
            };
        }
        // Measurements: ~1 microsecond each
        hardware_time_estimate_ns += num_measurements as f64 * 1000.0;

        let hardware_time_estimate_us = hardware_time_estimate_ns / 1000.0;

        Self {
            num_qubits,
            total_gates,
            num_measurements,
            depth,
            gate_counts,
            memory_bytes,
            hardware_time_estimate_us,
        }
    }
}

/// Circuit statistics panel UI.
#[derive(Debug, Default)]
pub struct StatsPanel;

impl StatsPanel {
    /// Renders the statistics panel.
    pub fn show(&mut self, ui: &mut egui::Ui, circuit: &Circuit) {
        let stats = CircuitStats::from_circuit(circuit);

        egui::CollapsingHeader::new("📊 Circuit Statistics")
            .default_open(true)
            .show(ui, |ui| {

                ui.add_space(4.0);

                // Basic circuit info
                egui::Grid::new("circuit_basic_stats")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Qubits:");
                        ui.label(format!("{}", stats.num_qubits));
                        ui.end_row();

                        ui.label("Total gates:");
                        ui.label(format!("{}", stats.total_gates));
                        ui.end_row();

                        ui.label("Measurements:");
                        ui.label(format!("{}", stats.num_measurements));
                        ui.end_row();

                        ui.label("Circuit depth:");
                        ui.label(format!("{}", stats.depth));
                        ui.end_row();
                    });

                ui.add_space(8.0);
                ui.separator();

                // Gate counts by type
                if stats.gate_counts.is_empty() {
                    ui.colored_label(
                        ui.visuals().weak_text_color(),
                        "No gates in circuit",
                    );
                } else {
                    ui.label("Gate counts by type:");
                    ui.add_space(4.0);

                    // Sort by count descending, then by name
                    let mut sorted_counts: Vec<_> = stats.gate_counts.iter().collect();
                    sorted_counts.sort_by(|a, b| {
                        let count_cmp = b.1.cmp(a.1);
                        if count_cmp == std::cmp::Ordering::Equal {
                            a.0.cmp(b.0)
                        } else {
                            count_cmp
                        }
                    });

                    egui::Grid::new("gate_counts")
                        .num_columns(2)
                        .spacing([8.0, 2.0])
                        .show(ui, |ui| {
                            for (gate_name, count) in sorted_counts {
                                ui.label(format!("  {}", gate_name));
                                ui.monospace(format!("{}", count));
                                ui.end_row();
                            }
                        });
                }

                ui.add_space(8.0);
                ui.separator();

                // Resource estimates
                ui.label("Resource estimates:");
                ui.add_space(4.0);

                egui::Grid::new("resource_estimates")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("State vector memory:");
                        let memory_text = format_memory(stats.memory_bytes);
                        let memory_color = if stats.memory_bytes > 1024 * 1024 * 1024 {
                            egui::Color32::from_rgb(255, 100, 100) // Red for >1GB
                        } else if stats.memory_bytes > 1024 * 1024 * 100 {
                            egui::Color32::from_rgb(255, 200, 100) // Yellow for >100MB
                        } else {
                            ui.visuals().text_color()
                        };
                        ui.colored_label(memory_color, memory_text);
                        ui.end_row();

                        ui.label("Hardware time (est):");
                        let time_text = if stats.hardware_time_estimate_us < 1.0 {
                            format!("{:.0} ns", stats.hardware_time_estimate_us * 1000.0)
                        } else if stats.hardware_time_estimate_us < 1000.0 {
                            format!("{:.1} µs", stats.hardware_time_estimate_us)
                        } else if stats.hardware_time_estimate_us < 1_000_000.0 {
                            format!("{:.1} ms", stats.hardware_time_estimate_us / 1000.0)
                        } else {
                            format!("{:.2} s", stats.hardware_time_estimate_us / 1_000_000.0)
                        };
                        ui.label(time_text);
                        ui.end_row();
                    });
            });
    }
}

/// Formats bytes into human-readable string (KB, MB, GB).
fn format_memory(bytes: usize) -> String {
    if bytes >= 1024 * 1024 * 1024 {
        format!("{:.1} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
    } else if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.0} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ketgrid_core::Circuit;

    #[test]
    fn test_empty_circuit_stats() {
        let circuit = Circuit::new(2);
        let stats = CircuitStats::from_circuit(&circuit);

        assert_eq!(stats.num_qubits, 2);
        assert_eq!(stats.total_gates, 0);
        assert_eq!(stats.num_measurements, 0);
        assert_eq!(stats.depth, 0);
        assert!(stats.gate_counts.is_empty());
        assert_eq!(stats.memory_bytes, 64); // 2^2 * 16 = 64 bytes
    }

    #[test]
    fn test_grover_2qubit_stats() {
        // Grover 2-qubit: depth=5, gates=10 (H=4, CNOT=4, etc)
        let mut circuit = Circuit::new(2);

        // Initial H gates on both qubits
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::H, vec![1], vec![], 0).unwrap();

        // Oracle: CNOT with control on q0, target on q1
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 1).unwrap();

        // Diffusion: H, X, CNOT, X, H
        circuit.add_gate(GateType::H, vec![0], vec![], 2).unwrap();
        circuit.add_gate(GateType::H, vec![1], vec![], 2).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 3).unwrap();
        circuit.add_gate(GateType::X, vec![1], vec![], 3).unwrap();
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 4).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 5).unwrap();
        circuit.add_gate(GateType::X, vec![1], vec![], 5).unwrap();
        circuit.add_gate(GateType::H, vec![0], vec![], 6).unwrap();
        circuit.add_gate(GateType::H, vec![1], vec![], 6).unwrap();

        let stats = CircuitStats::from_circuit(&circuit);

        assert_eq!(stats.num_qubits, 2);
        assert_eq!(stats.total_gates, 12); // All gates added
        assert_eq!(stats.depth, 6);

        // Check gate counts
        assert_eq!(*stats.gate_counts.get("H").unwrap_or(&0), 6);
        assert_eq!(*stats.gate_counts.get("X").unwrap_or(&0), 4);
        assert_eq!(*stats.gate_counts.get("C+").unwrap_or(&0), 2);
    }

    #[test]
    fn test_circuit_with_measurements() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_measurement(0, 1).unwrap();
        circuit.add_measurement(1, 1).unwrap();

        let stats = CircuitStats::from_circuit(&circuit);

        assert_eq!(stats.total_gates, 1);
        assert_eq!(stats.num_measurements, 2);
        assert_eq!(stats.depth, 1);
    }

    #[test]
    fn test_memory_formatting() {
        assert_eq!(format_memory(100), "100 B");
        assert_eq!(format_memory(1024), "1 KB");
        assert_eq!(format_memory(1536), "2 KB");
        assert_eq!(format_memory(1024 * 1024), "1.0 MB");
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn test_hardware_time_estimate() {
        let mut circuit = Circuit::new(2);
        // Single qubit gates
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![0], vec![], 1).unwrap();
        // Two qubit gate
        circuit.add_gate(GateType::Cnot, vec![1], vec![0], 2).unwrap();
        // Measurements
        circuit.add_measurement(0, 3).unwrap();
        circuit.add_measurement(1, 3).unwrap();

        let stats = CircuitStats::from_circuit(&circuit);

        // Expected: 2 * 50ns + 1 * 500ns + 2 * 1000ns = 2600ns = 2.6 µs
        assert!(stats.hardware_time_estimate_us > 2.0);
        assert!(stats.hardware_time_estimate_us < 3.0);
    }
}
