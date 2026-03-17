//! Bloch sphere visualization widget for per-qubit state display.
//!
//! Computes reduced density matrices via partial trace, extracts Bloch
//! vectors, and renders interactive 2D-projected spheres with drag-to-rotate.

use ketgrid_sim::state_vector::StateVector;

const SPHERE_RADIUS: f32 = 55.0;
const SPHERE_WIDGET_SIZE: f32 = 130.0;
const WIREFRAME_SEGMENTS: usize = 64;
const MAX_DISPLAY_QUBITS: usize = 8;
const DRAG_SENSITIVITY: f32 = 0.01;
const ELEVATION_LIMIT: f32 = 1.3; // ~75 degrees

/// Interactive Bloch sphere panel showing per-qubit state on the Bloch sphere.
///
/// Renders a 2D-projected sphere per qubit with the Bloch vector arrow.
/// Drag any sphere to rotate the shared viewing angle.
#[derive(Debug)]
pub struct BlochSphere {
    /// Azimuth angle for 3D view rotation (radians).
    view_azimuth: f32,
    /// Elevation angle for 3D view rotation (radians).
    view_elevation: f32,
    /// Whether the Bloch sphere section is visible.
    show: bool,
}

impl Default for BlochSphere {
    fn default() -> Self {
        Self {
            view_azimuth: 0.0,
            view_elevation: std::f32::consts::FRAC_PI_6,
            show: true,
        }
    }
}

impl BlochSphere {
    /// Renders the Bloch sphere section for all qubits in the state vector.
    pub fn show(&mut self, ui: &mut egui::Ui, state_vector: &StateVector) {
        let n = state_vector.num_qubits();
        if n == 0 {
            return;
        }

        ui.checkbox(&mut self.show, "Show Bloch Spheres");

        if !self.show {
            return;
        }

        ui.add_space(4.0);

        let display_count = n.min(MAX_DISPLAY_QUBITS);
        if n > MAX_DISPLAY_QUBITS {
            ui.colored_label(
                egui::Color32::from_rgb(200, 200, 100),
                format!("Showing first {} of {} qubits", MAX_DISPLAY_QUBITS, n),
            );
        }

        ui.horizontal_wrapped(|ui| {
            for qubit in 0..display_count {
                let bv = compute_bloch_vector(state_vector, qubit);
                self.render_qubit_sphere(ui, bv, qubit);
            }
        });
    }

    fn render_qubit_sphere(&mut self, ui: &mut egui::Ui, bv: [f64; 3], qubit: usize) {
        let size = egui::vec2(SPHERE_WIDGET_SIZE, SPHERE_WIDGET_SIZE + 18.0);
        let (rect, response) = ui.allocate_exact_size(size, egui::Sense::drag());

        if response.dragged() {
            let delta = response.drag_delta();
            self.view_azimuth -= delta.x * DRAG_SENSITIVITY;
            self.view_elevation = (self.view_elevation - delta.y * DRAG_SENSITIVITY)
                .clamp(-ELEVATION_LIMIT, ELEVATION_LIMIT);
        }

        let painter = ui.painter();
        let center = egui::pos2(rect.center().x, rect.min.y + SPHERE_RADIUS + 4.0);
        let r = SPHERE_RADIUS;

        let bg = egui::Color32::from_gray(25);
        let outline = egui::Color32::from_gray(70);
        let wire_front = egui::Stroke::new(0.7, egui::Color32::from_gray(85));
        let wire_back = egui::Stroke::new(0.5, egui::Color32::from_gray(38));
        let label_color = egui::Color32::from_gray(150);
        let font_sm = egui::FontId::proportional(9.5);

        // Sphere background and outline
        painter.circle_filled(center, r, bg);
        painter.circle_stroke(center, r, egui::Stroke::new(1.0, outline));

        // Equator (XY plane)
        self.draw_great_circle(
            painter,
            center,
            r,
            |t| [t.cos(), t.sin(), 0.0],
            wire_front,
            wire_back,
        );
        // Meridian (XZ plane)
        self.draw_great_circle(
            painter,
            center,
            r,
            |t| [t.cos(), 0.0, t.sin()],
            wire_front,
            wire_back,
        );

        // Axis labels
        let north = self.to_screen([0.0, 0.0, 1.0], center, r);
        let south = self.to_screen([0.0, 0.0, -1.0], center, r);
        painter.text(
            north + egui::vec2(0.0, -7.0),
            egui::Align2::CENTER_BOTTOM,
            "|0>",
            font_sm.clone(),
            label_color,
        );
        painter.text(
            south + egui::vec2(0.0, 7.0),
            egui::Align2::CENTER_TOP,
            "|1>",
            font_sm.clone(),
            label_color,
        );

        // |+> and |-> labels only when facing the viewer
        if self.point_depth([1.0, 0.0, 0.0]) < 0.15 {
            let p = self.to_screen([1.0, 0.0, 0.0], center, r);
            painter.text(
                p + egui::vec2(6.0, 0.0),
                egui::Align2::LEFT_CENTER,
                "|+>",
                font_sm.clone(),
                label_color,
            );
        }
        if self.point_depth([-1.0, 0.0, 0.0]) < 0.15 {
            let p = self.to_screen([-1.0, 0.0, 0.0], center, r);
            painter.text(
                p + egui::vec2(-6.0, 0.0),
                egui::Align2::RIGHT_CENTER,
                "|->",
                font_sm.clone(),
                label_color,
            );
        }

        // Center dot
        painter.circle_filled(center, 1.5, egui::Color32::from_gray(55));

        // Bloch vector
        let bv_f32 = [bv[0] as f32, bv[1] as f32, bv[2] as f32];
        let purity =
            (bv_f32[0] * bv_f32[0] + bv_f32[1] * bv_f32[1] + bv_f32[2] * bv_f32[2]).sqrt();
        let tip = self.to_screen(bv_f32, center, r);
        let vec_2d = tip - center;

        let p = purity.min(1.0);
        let vec_color = egui::Color32::from_rgb(
            (60.0 + 40.0 * (1.0 - p)) as u8,
            (190.0 * p + 80.0 * (1.0 - p)) as u8,
            (255.0 * p + 100.0 * (1.0 - p)) as u8,
        );

        if vec_2d.length() > 3.0 {
            painter.arrow(center, vec_2d, egui::Stroke::new(2.0, vec_color));
        }
        painter.circle_filled(tip, 3.5, vec_color);

        // Qubit label below sphere
        let label_pos = egui::pos2(rect.center().x, rect.max.y - 2.0);
        painter.text(
            label_pos,
            egui::Align2::CENTER_BOTTOM,
            format!("q{}", qubit),
            egui::FontId::proportional(11.0),
            egui::Color32::from_gray(180),
        );

        // Hover tooltip
        let purity_pct = purity * 100.0;
        response.on_hover_text(format!(
            "q{qubit} Bloch vector\nx: {:+.3}\ny: {:+.3}\nz: {:+.3}\nPurity: {purity_pct:.0}%",
            bv[0], bv[1], bv[2],
        ));
    }

    /// Projects a 3D point to 2D screen coordinates via orthographic projection.
    fn to_screen(&self, p: [f32; 3], center: egui::Pos2, radius: f32) -> egui::Pos2 {
        let (sx, sy, _) = self.project(p);
        egui::pos2(center.x + sx * radius, center.y - sy * radius)
    }

    /// Returns the depth of a 3D point (positive = behind sphere center).
    fn point_depth(&self, p: [f32; 3]) -> f32 {
        let (_, _, depth) = self.project(p);
        depth
    }

    /// Applies view rotation: azimuth around Z, elevation around X.
    ///
    /// Returns (screen_x, screen_y_up, depth) in normalized sphere coordinates.
    fn project(&self, p: [f32; 3]) -> (f32, f32, f32) {
        let (sa, ca) = self.view_azimuth.sin_cos();
        let (se, ce) = self.view_elevation.sin_cos();

        // Rotate around Z by azimuth
        let x1 = p[0] * ca + p[1] * sa;
        let y1 = -p[0] * sa + p[1] * ca;

        // Rotate around X by elevation
        let screen_x = x1;
        let screen_y = y1 * se + p[2] * ce;
        let depth = y1 * ce - p[2] * se;

        (screen_x, screen_y, depth)
    }

    /// Draws a great circle on the sphere, distinguishing front vs back segments.
    fn draw_great_circle(
        &self,
        painter: &egui::Painter,
        center: egui::Pos2,
        radius: f32,
        point_fn: impl Fn(f32) -> [f32; 3],
        front_stroke: egui::Stroke,
        back_stroke: egui::Stroke,
    ) {
        let tau = std::f32::consts::TAU;
        for i in 0..WIREFRAME_SEGMENTS {
            let t0 = tau * i as f32 / WIREFRAME_SEGMENTS as f32;
            let t1 = tau * (i + 1) as f32 / WIREFRAME_SEGMENTS as f32;

            let p0 = point_fn(t0);
            let p1 = point_fn(t1);

            let avg_depth = (self.point_depth(p0) + self.point_depth(p1)) * 0.5;
            let stroke = if avg_depth < 0.0 {
                front_stroke
            } else {
                back_stroke
            };

            let s0 = self.to_screen(p0, center, radius);
            let s1 = self.to_screen(p1, center, radius);

            painter.line_segment([s0, s1], stroke);
        }
    }
}

/// Computes the Bloch vector for qubit `target` in an n-qubit state via partial trace.
///
/// Traces out all qubits except `target` to obtain the 2x2 reduced density matrix,
/// then extracts: r_x = 2 Re(rho_01), r_y = -2 Im(rho_01), r_z = rho_00 - rho_11.
///
/// Convention: |0> = north pole (+z), |1> = south pole (-z),
/// |+> = +x equator, |+i> = +y equator.
pub fn compute_bloch_vector(state_vector: &StateVector, target: usize) -> [f64; 3] {
    let n = state_vector.num_qubits();
    debug_assert!(target < n, "target qubit {target} out of range for {n}-qubit state");

    let amplitudes = state_vector.data();
    let bit_pos = n - 1 - target;
    let dim = amplitudes.len();

    let mut rho_00: f64 = 0.0;
    let mut rho_11: f64 = 0.0;
    let mut rho_01_re: f64 = 0.0;
    let mut rho_01_im: f64 = 0.0;

    for i in 0..dim {
        if (i >> bit_pos) & 1 == 0 {
            let j = i | (1 << bit_pos);
            let ai_re = amplitudes[i].re;
            let ai_im = amplitudes[i].im;
            let aj_re = amplitudes[j].re;
            let aj_im = amplitudes[j].im;

            rho_00 += ai_re * ai_re + ai_im * ai_im;
            rho_11 += aj_re * aj_re + aj_im * aj_im;
            // rho_01 = sum_k a_k0 * conj(a_k1) where k0/k1 differ only at target qubit
            rho_01_re += ai_re * aj_re + ai_im * aj_im;
            rho_01_im += ai_im * aj_re - ai_re * aj_im;
        }
    }

    [2.0 * rho_01_re, -2.0 * rho_01_im, rho_00 - rho_11]
}

#[cfg(test)]
mod tests {
    use super::*;
    use ketgrid_core::{Circuit, GateType};
    use ketgrid_sim::StateVectorSimulator;

    const TOL: f64 = 1e-10;

    fn assert_bloch_eq(actual: [f64; 3], expected: [f64; 3], label: &str) {
        let labels = ["x", "y", "z"];
        for i in 0..3 {
            assert!(
                (actual[i] - expected[i]).abs() < TOL,
                "{label}: component {} mismatch: got {}, expected {}",
                labels[i],
                actual[i],
                expected[i],
            );
        }
    }

    // |0> -> north pole (0, 0, 1)
    #[test]
    fn test_zero_state_north_pole() {
        let sim = StateVectorSimulator::new(1);
        let bv = compute_bloch_vector(sim.state(), 0);
        assert_bloch_eq(bv, [0.0, 0.0, 1.0], "|0>");
    }

    // X|0> = |1> -> south pole (0, 0, -1)
    #[test]
    fn test_one_state_south_pole() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::X, vec![0], vec![], 0).unwrap();
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);
        let bv = compute_bloch_vector(sim.state(), 0);
        assert_bloch_eq(bv, [0.0, 0.0, -1.0], "|1>");
    }

    // H|0> = |+> -> equator +X (1, 0, 0)
    #[test]
    fn test_plus_state_equator_x() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);
        let bv = compute_bloch_vector(sim.state(), 0);
        assert_bloch_eq(bv, [1.0, 0.0, 0.0], "H|0> = |+>");
    }

    // S*H|0> = (|0> + i|1>)/sqrt(2) -> equator +Y (0, 1, 0)
    #[test]
    fn test_plus_i_state_equator_y() {
        let mut circuit = Circuit::new(1);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::S, vec![0], vec![], 1).unwrap();
        let mut sim = StateVectorSimulator::new(1);
        sim.apply_circuit(&circuit);
        let bv = compute_bloch_vector(sim.state(), 0);
        assert_bloch_eq(bv, [0.0, 1.0, 0.0], "S*H|0> = |+i>");
    }

    // Bell state: each qubit is maximally mixed -> Bloch vector (0, 0, 0)
    #[test]
    fn test_bell_state_maximally_mixed() {
        let mut circuit = Circuit::new(2);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit
            .add_gate(GateType::Cnot, vec![1], vec![0], 1)
            .unwrap();
        let mut sim = StateVectorSimulator::new(2);
        sim.apply_circuit(&circuit);

        for qubit in 0..2 {
            let bv = compute_bloch_vector(sim.state(), qubit);
            let len = (bv[0] * bv[0] + bv[1] * bv[1] + bv[2] * bv[2]).sqrt();
            assert!(
                len < TOL,
                "Bell state qubit {qubit} should be maximally mixed, got |r|={len}",
            );
        }
    }

    // Product state |+> x |0> x |1>: partial trace should give independent Bloch vectors
    #[test]
    fn test_three_qubit_product_state() {
        let mut circuit = Circuit::new(3);
        circuit.add_gate(GateType::H, vec![0], vec![], 0).unwrap();
        circuit.add_gate(GateType::X, vec![2], vec![], 0).unwrap();
        let mut sim = StateVectorSimulator::new(3);
        sim.apply_circuit(&circuit);

        assert_bloch_eq(
            compute_bloch_vector(sim.state(), 0),
            [1.0, 0.0, 0.0],
            "q0: |+>",
        );
        assert_bloch_eq(
            compute_bloch_vector(sim.state(), 1),
            [0.0, 0.0, 1.0],
            "q1: |0>",
        );
        assert_bloch_eq(
            compute_bloch_vector(sim.state(), 2),
            [0.0, 0.0, -1.0],
            "q2: |1>",
        );
    }

    // Purity: pure states should have |r| = 1.0
    #[test]
    fn test_pure_state_purity() {
        let sim = StateVectorSimulator::new(1);
        let bv = compute_bloch_vector(sim.state(), 0);
        let purity = (bv[0] * bv[0] + bv[1] * bv[1] + bv[2] * bv[2]).sqrt();
        assert!(
            (purity - 1.0).abs() < TOL,
            "Pure state purity should be 1.0, got {purity}",
        );
    }
}
