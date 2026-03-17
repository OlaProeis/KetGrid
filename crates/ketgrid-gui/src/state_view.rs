//! Quantum state visualization panel (probabilities, amplitudes, Bloch sphere).

use crate::bloch::BlochSphere;
use ketgrid_sim::state_vector::StateVector;
use ketgrid_sim::EntanglementInfo;

/// State visualization panel.
#[derive(Debug, Default)]
pub struct StateView {
    /// Toggle for showing the amplitude table.
    show_amplitude_table: bool,
    /// Bloch sphere visualization widget.
    bloch_sphere: BlochSphere,
}

impl StateView {
    /// Renders the state visualization.
    ///
    /// `entanglement`: optional entanglement info and per-qubit wire colors for display.
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        state_vector: &StateVector,
        entanglement: Option<(&EntanglementInfo, &[Option<egui::Color32>])>,
    ) {
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

        self.bloch_sphere.show(ui, state_vector);

        ui.add_space(16.0);
        ui.separator();

        if let Some((info, colors)) = entanglement {
            self.show_entanglement_ui(ui, info, colors);
        }
    }

    /// Renders the entanglement clusters section.
    fn show_entanglement_ui(
        &self,
        ui: &mut egui::Ui,
        info: &EntanglementInfo,
        colors: &[Option<egui::Color32>],
    ) {
        let entangled_clusters: Vec<&Vec<usize>> =
            info.clusters.iter().filter(|c| c.len() > 1).collect();

        ui.label("Entanglement:");
        ui.add_space(4.0);

        if entangled_clusters.is_empty() {
            ui.colored_label(
                egui::Color32::from_gray(140),
                "  No entanglement detected",
            );
        } else {
            for cluster in &entangled_clusters {
                let first_qubit = cluster[0];
                let color = colors
                    .get(first_qubit)
                    .and_then(|c| *c)
                    .unwrap_or(egui::Color32::WHITE);

                let qubit_names: Vec<String> =
                    cluster.iter().map(|&q| format!("q{q}")).collect();

                ui.horizontal(|ui| {
                    let (dot_rect, _) =
                        ui.allocate_exact_size(egui::vec2(10.0, 10.0), egui::Sense::hover());
                    ui.painter().circle_filled(dot_rect.center(), 4.0, color);

                    ui.colored_label(color, qubit_names.join(", "));
                });
            }
        }

        ui.add_space(8.0);

        // Per-qubit purity
        ui.label("Qubit Purity:");
        ui.add_space(2.0);
        for (q, &purity) in info.qubit_purities.iter().enumerate() {
            let color = colors
                .get(q)
                .and_then(|c| *c)
                .unwrap_or_else(|| ui.visuals().text_color());
            let purity_pct = purity * 100.0;
            let label = if purity > 0.999 {
                format!("  q{q}: pure")
            } else {
                format!("  q{q}: {purity_pct:.0}%")
            };
            ui.colored_label(color, label);
        }
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

        // Use a table-like grid with consistent spacing
        let state_col_width = 70.0;
        let real_col_width = 90.0;
        let imag_col_width = 90.0;
        let phase_col_width = 65.0;

        let text_color = ui.visuals().text_color();
        let weak_color = ui.visuals().weak_text_color();

        egui::ScrollArea::vertical()
            .id_salt("amplitudes")
            .max_height(200.0)
            .show(ui, |ui| {
                // Build the table as a single vertical layout with horizontal rows
                // Header
                let header_text = format!(
                    "{: <state_width$} | {: <real_width$} | {: <imag_width$} | {: <phase_width$}",
                    "State",
                    "Real",
                    "Imaginary",
                    "Phase",
                    state_width = (state_col_width / 8.0) as usize,
                    real_width = (real_col_width / 8.0) as usize,
                    imag_width = (imag_col_width / 8.0) as usize,
                    phase_width = (phase_col_width / 8.0) as usize,
                );
                ui.label(egui::RichText::new(&header_text).strong().monospace().color(text_color));

                // Separator line
                let separator = format!(
                    "{:-<state_width$}-+-{:-<real_width$}-+-{:-<imag_width$}-+-{:-<phase_width$}",
                    "",
                    "",
                    "",
                    "",
                    state_width = (state_col_width / 8.0) as usize,
                    real_width = (real_col_width / 8.0) as usize,
                    imag_width = (imag_col_width / 8.0) as usize,
                    phase_width = (phase_col_width / 8.0) as usize,
                );
                ui.label(egui::RichText::new(&separator).monospace().color(weak_color));

                // Data rows
                for (idx, (amp, prob)) in amplitudes.iter().zip(probs.iter()).enumerate() {
                    // Skip states with zero probability for cleaner display
                    if *prob < 1e-10 {
                        continue;
                    }

                    let label = format!("|{:0width$b}>", idx, width = num_qubits);
                    let real_str = format!("{:+.4}", amp.re);
                    let imag_str = format!("{:+.4}", amp.im);
                    let phase_str = format!("{:+.1}°", amp.arg().to_degrees());

                    let row_text = format!(
                        "{: <state_width$} | {: <real_width$} | {: <imag_width$} | {: <phase_width$}",
                        label,
                        real_str,
                        imag_str,
                        phase_str,
                        state_width = (state_col_width / 8.0) as usize,
                        real_width = (real_col_width / 8.0) as usize,
                        imag_width = (imag_col_width / 8.0) as usize,
                        phase_width = (phase_col_width / 8.0) as usize,
                    );
                    ui.label(egui::RichText::new(&row_text).monospace().color(text_color));
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
