//! Gate palette UI for dragging gates onto the circuit.

use ketgrid_core::gate::GateType;

/// Type of item selected in the palette.
#[derive(Debug, Clone, PartialEq)]
pub enum PaletteSelection {
    /// A quantum gate.
    Gate(GateType),
    /// A measurement operation.
    Measurement,
}

/// Gate palette panel state.
#[derive(Debug)]
pub struct GatePalette {
    /// Currently selected item for drag placement.
    selected: Option<PaletteSelection>,
    /// Collapsible section states.
    basic_open: bool,
    phase_open: bool,
    rotation_open: bool,
    multi_qubit_open: bool,
    meta_open: bool,
    measurement_open: bool,
}

impl Default for GatePalette {
    fn default() -> Self {
        Self {
            selected: None,
            basic_open: true,
            phase_open: true,
            rotation_open: true,
            multi_qubit_open: true,
            meta_open: false,
            measurement_open: true,
        }
    }
}

impl GatePalette {
    /// Renders the gate palette with collapsible sections.
    pub fn show(&mut self, ui: &mut egui::Ui) {
        egui::ScrollArea::vertical().show(ui, |ui| {
            // Basic Gates Section
            self.basic_section(ui);

            ui.add_space(8.0);

            // Phase Gates Section
            self.phase_section(ui);

            ui.add_space(8.0);

            // Rotation Gates Section
            self.rotation_section(ui);

            ui.add_space(8.0);

            // Multi-Qubit Gates Section
            self.multi_qubit_section(ui);

            ui.add_space(8.0);

            // Meta Gates Section
            self.meta_section(ui);

            ui.add_space(8.0);

            // Measurement Section
            self.measurement_section(ui);
        });
    }

    /// Returns the currently selected item for drag operations.
    pub fn selected(&self) -> Option<&PaletteSelection> {
        self.selected.as_ref()
    }

    /// Returns the currently selected gate (if a gate is selected).
    pub fn selected_gate(&self) -> Option<&GateType> {
        match self.selected {
            Some(PaletteSelection::Gate(ref gate)) => Some(gate),
            _ => None,
        }
    }

    /// Returns true if measurement placement mode is selected.
    pub fn is_measurement_selected(&self) -> bool {
        matches!(self.selected, Some(PaletteSelection::Measurement))
    }

    /// Clears the current selection (call when drag completes).
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// Basic gates: H, X, Y, Z
    fn basic_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Basic Gates")
            .default_open(self.basic_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.gate_button(ui, GateType::H, "H");
                    self.gate_button(ui, GateType::X, "X");
                    self.gate_button(ui, GateType::Y, "Y");
                    self.gate_button(ui, GateType::Z, "Z");
                });
            });
    }

    /// Phase gates: S, T
    fn phase_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Phase Gates")
            .default_open(self.phase_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.gate_button(ui, GateType::S, "S");
                    self.gate_button(ui, GateType::T, "T");
                });
            });
    }

    /// Rotation gates: Rx, Ry, Rz (with default π/2 angle)
    fn rotation_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Rotation Gates")
            .default_open(self.rotation_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.gate_button(ui, GateType::Rx(std::f64::consts::FRAC_PI_2), "Rx");
                    self.gate_button(ui, GateType::Ry(std::f64::consts::FRAC_PI_2), "Ry");
                    self.gate_button(ui, GateType::Rz(std::f64::consts::FRAC_PI_2), "Rz");
                });
            });
    }

    /// Multi-qubit gates: CNOT, CZ, SWAP, Toffoli
    fn multi_qubit_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Multi-Qubit Gates")
            .default_open(self.multi_qubit_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.gate_button(ui, GateType::Cnot, "C+");
                    self.gate_button(ui, GateType::Cz, "CZ");
                    self.gate_button(ui, GateType::Swap, "SW");
                    self.gate_button(ui, GateType::Toffoli, "CC+");
                });
            });
    }

    /// Meta gates: Barrier, Identity
    fn meta_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Meta Gates")
            .default_open(self.meta_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.gate_button(ui, GateType::Barrier, "|");
                    self.gate_button(ui, GateType::Identity, "I");
                });
            });
    }

    /// Measurement operations.
    fn measurement_section(&mut self, ui: &mut egui::Ui) {
        egui::CollapsingHeader::new("Operations")
            .default_open(self.measurement_open)
            .show(ui, |ui| {
                ui.horizontal_wrapped(|ui| {
                    self.measurement_button(ui);
                });
            });
    }

    /// Renders a measurement button.
    fn measurement_button(&mut self, ui: &mut egui::Ui) {
        let is_selected = matches!(self.selected, Some(PaletteSelection::Measurement));
        let button_size = egui::Vec2::new(56.0, 40.0);

        let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click_and_drag());

        let visuals = ui.style().interact(&response);

        // Determine background color based on state
        let bg_color = if is_selected {
            egui::Color32::from_rgb(100, 150, 255)
        } else if response.hovered() {
            visuals.bg_fill.gamma_multiply(1.3)
        } else {
            visuals.bg_fill
        };

        // Draw button background
        ui.painter().rect_filled(rect, visuals.corner_radius, bg_color);

        // Draw selection border if selected
        if is_selected {
            ui.painter().rect_stroke(
                rect,
                visuals.corner_radius,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 230)),
                egui::StrokeKind::Middle,
            );
        } else {
            ui.painter().rect_stroke(
                rect,
                visuals.corner_radius,
                visuals.bg_stroke,
                egui::StrokeKind::Middle,
            );
        }

        // Draw measurement icon (simplified meter symbol)
        let text_color = if is_selected {
            egui::Color32::WHITE
        } else {
            visuals.text_color()
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            "M",
            egui::FontId::proportional(16.0),
            text_color,
        );

        // Handle interactions: click toggles selection, drag always selects
        if response.clicked() {
            if is_selected {
                self.selected = None;
            } else {
                self.selected = Some(PaletteSelection::Measurement);
            }
        }

        if response.drag_started() {
            self.selected = Some(PaletteSelection::Measurement);
        }
    }

    /// Renders a draggable gate button.
    fn gate_button(&mut self, ui: &mut egui::Ui, gate_type: GateType, label: &str) {
        let is_selected = matches!(
            self.selected,
            Some(PaletteSelection::Gate(ref g)) if *g == gate_type
        );

        let button_size = if label.len() > 2 {
            egui::Vec2::new(56.0, 40.0)
        } else {
            egui::Vec2::splat(40.0)
        };

        let (rect, response) = ui.allocate_exact_size(button_size, egui::Sense::click_and_drag());

        let visuals = ui.style().interact(&response);

        // Determine background color based on state
        let bg_color = if is_selected {
            // Highlight color for selected gate
            egui::Color32::from_rgb(100, 150, 255)
        } else if response.hovered() {
            visuals.bg_fill.gamma_multiply(1.3)
        } else {
            visuals.bg_fill
        };

        // Draw button background
        ui.painter().rect_filled(rect, visuals.corner_radius, bg_color);

        // Draw selection border if selected
        if is_selected {
            ui.painter().rect_stroke(
                rect,
                visuals.corner_radius,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(70, 130, 230)),
                egui::StrokeKind::Middle,
            );
        } else {
            ui.painter().rect_stroke(
                rect,
                visuals.corner_radius,
                visuals.bg_stroke,
                egui::StrokeKind::Middle,
            );
        }

        // Draw gate label
        let text_color = if is_selected {
            egui::Color32::WHITE
        } else {
            visuals.text_color()
        };

        ui.painter().text(
            rect.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(if label.len() > 2 { 14.0 } else { 16.0 }),
            text_color,
        );

        // Handle interactions: click toggles selection, drag always selects
        if response.clicked() {
            if is_selected {
                self.selected = None;
            } else {
                self.selected = Some(PaletteSelection::Gate(gate_type.clone()));
            }
        }

        if response.drag_started() {
            self.selected = Some(PaletteSelection::Gate(gate_type));
        }
    }
}
