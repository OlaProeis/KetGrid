//! Quantum state visualization panel (probabilities, amplitudes, Bloch sphere).

use ketgrid_sim::state_vector::StateVector;

/// State visualization panel.
#[derive(Debug, Default)]
pub struct StateView {
    /// Toggle for showing the amplitude table
    show_amplitude_table: bool,
}

impl StateView {
    /// Renders the state visualization.
    pub fn show(&mut self, ui: &mut egui::Ui, state_vector: &StateVector) {
        ui.label(format!("Qubits: {}", state_vector.num_qubits()));
        ui.separator();

        // Amplitude/probability bars
        let probs = state_vector.probabilities();
        let amplitudes = state_vector.data();
        let max_prob = probs.iter().copied().fold(0.0_f64, f64::max).max(1e-10);

        ui.label("Measurement Probabilities:");
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_salt("probabilities")
            .max_height(300.0)
            .show(ui, |ui| {
                for (idx, prob) in probs.iter().enumerate() {
                    let label = format!("|{:0width$b}>", idx, width = state_vector.num_qubits());
                    let bar_width = 200.0 * (*prob / max_prob) as f32;

                    // Get phase for coloring: red = 0°, blue = 180°
                    let phase_color = if *prob > 0.0 {
                        let amp = amplitudes[idx];
                        phase_to_color(amp.arg())
                    } else {
                        egui::Color32::GRAY
                    };

                    ui.horizontal(|ui| {
                        ui.label(&label);
                        ui.add_space(8.0);

                        // Probability bar with phase-aware coloring
                        let (rect, _) = ui.allocate_exact_size(
                            egui::vec2(bar_width.max(2.0), 16.0),
                            egui::Sense::hover(),
                        );
                        ui.painter().rect_filled(rect, 2.0, phase_color);

                        ui.label(format!("{:.1}%", prob * 100.0));
                    });
                }
            });

        ui.add_space(16.0);
        ui.separator();

        // Amplitude table toggle
        ui.checkbox(&mut self.show_amplitude_table, "Show Amplitude Table");

        if self.show_amplitude_table {
            ui.add_space(8.0);
            self.show_amplitude_table_ui(ui, state_vector, &probs);
        }

        ui.add_space(16.0);
        ui.separator();

        // Entropy / purity metrics (placeholder)
        ui.label("State Metrics:");
        ui.label("  Von Neumann entropy: —");
        ui.label("  Purity: —");
    }

    /// Shows the amplitude table with real and imaginary parts.
    fn show_amplitude_table_ui(
        &self,
        ui: &mut egui::Ui,
        state_vector: &StateVector,
        probs: &[f64],
    ) {
        let amplitudes = state_vector.data();
        let num_qubits = state_vector.num_qubits();

        ui.label("Amplitudes (Real + Imaginary):");
        ui.add_space(4.0);

        egui::ScrollArea::vertical()
            .id_salt("amplitudes")
            .max_height(200.0)
            .show(ui, |ui| {
                // Header row
                ui.horizontal(|ui| {
                    ui.label("State").on_hover_text("Basis state |n>");
                    ui.add_space(40.0);
                    ui.label("Real").on_hover_text("Real part of amplitude");
                    ui.add_space(60.0);
                    ui.label("Imaginary").on_hover_text("Imaginary part of amplitude");
                    ui.add_space(40.0);
                    ui.label("Phase").on_hover_text("Phase angle in degrees");
                });
                ui.separator();

                for (idx, (amp, prob)) in amplitudes.iter().zip(probs.iter()).enumerate() {
                    // Skip states with zero probability for cleaner display
                    if *prob < 1e-10 {
                        continue;
                    }

                    let label = format!("|{:0width$b}>", idx, width = num_qubits);
                    let real = amp.re;
                    let imag = amp.im;
                    let phase_deg = amp.arg().to_degrees();

                    ui.horizontal(|ui| {
                        ui.monospace(&label);
                        ui.add_space(16.0);
                        ui.monospace(format!("{:+.4}", real));
                        ui.add_space(16.0);
                        ui.monospace(format!("{:+.4}", imag));
                        ui.add_space(16.0);
                        ui.monospace(format!("{:+.1}°", phase_deg));
                    });
                }
            });
    }
}

/// Converts a phase angle (in radians, range [-π, π]) to a color.
/// - Phase 0 (or ±π) → Red (0° phase reference)
/// - Phase π/2 → Purple (intermediate)
/// - Phase ±π → Blue (180° phase)
fn phase_to_color(phase: f64) -> egui::Color32 {
    // Normalize phase to [0, 2π] range
    let normalized = if phase < 0.0 {
        phase + 2.0 * std::f64::consts::PI
    } else {
        phase
    };

    // Map phase to hue: 0° (red) -> 180° (blue) -> 360°/0° (red)
    // We want: phase 0 = red (0° hue), phase π = blue (240° hue)
    // Using HSL: red = 0°, blue = 240°
    let hue = (normalized / std::f64::consts::PI) * 180.0;
    let hue = hue.clamp(0.0, 360.0) as f32;

    // Use high saturation and medium lightness for vibrant colors
    hsv_to_rgb(hue, 0.85, 0.55)
}

/// Converts HSV color to RGB Color32.
/// h: 0-360, s: 0-1, v: 0-1
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> egui::Color32 {
    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = if h < 60.0 {
        (c, x, 0.0)
    } else if h < 120.0 {
        (x, c, 0.0)
    } else if h < 180.0 {
        (0.0, c, x)
    } else if h < 240.0 {
        (0.0, x, c)
    } else if h < 300.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    egui::Color32::from_rgb(
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}
