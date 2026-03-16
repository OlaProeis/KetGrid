//! Main egui application shell with three-panel layout.

use ketgrid_core::circuit::PlacedGate;
use ketgrid_core::gate::GateType;
use ketgrid_core::Circuit;
use ketgrid_sim::StateVectorSimulator;

use crate::circuit_view::{CircuitView, GateHitResult, GateInteraction};
use crate::editor::{ClipboardContent, DropTarget, EditorState};
use crate::gate_palette::{GatePalette, PaletteSelection};
use crate::state_view::StateView;

/// Memory estimation constants for state vector simulation.
const BYTES_PER_COMPLEX: usize = 16; // Complex<f64> = 2 * 8 bytes

/// Actions that can be triggered from the context menu.
enum ContextMenuAction {
    Edit(usize),
    Copy(PlacedGate),
    Paste(usize),
    DeleteGate(usize),
    DeleteMeasurement(usize),
}

/// Returns estimated memory for state vector: 2^n * 16 bytes.
fn estimate_state_vector_memory_bytes(num_qubits: usize) -> usize {
    if num_qubits >= 64 {
        // Would overflow usize, return max
        usize::MAX
    } else {
        (1usize << num_qubits) * BYTES_PER_COMPLEX
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

/// Returns system memory information (available, total).
#[cfg(target_os = "windows")]
fn get_system_memory() -> Option<(usize, usize)> {
    use std::mem;
    unsafe {
        let mut mem_status: winapi::um::sysinfoapi::MEMORYSTATUSEX = mem::zeroed();
        mem_status.dwLength = mem::size_of::<winapi::um::sysinfoapi::MEMORYSTATUSEX>() as u32;
        winapi::um::sysinfoapi::GlobalMemoryStatusEx(&mut mem_status);
        let available = mem_status.ullAvailPhys as usize;
        let total = mem_status.ullTotalPhys as usize;
        Some((available, total))
    }
}

#[cfg(not(target_os = "windows"))]
fn get_system_memory() -> Option<(usize, usize)> {
    // Fallback for non-Windows platforms
    None
}

/// Main application state.
pub struct KetGridApp {
    /// Currently open circuit.
    circuit: Circuit,
    /// Current simulator (if any).
    simulator: Option<StateVectorSimulator>,
    /// Gate palette panel state.
    gate_palette: GatePalette,
    /// Circuit view panel state.
    circuit_view: CircuitView,
    /// State visualization panel state.
    state_view: StateView,
    /// Editor state for drag-and-drop gate placement.
    editor_state: EditorState,
}

impl KetGridApp {
    /// Creates a new app instance.
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let mut circuit = Circuit::new(3);
        // Demo Bell state: H on q0 col 0, CNOT(q0→q1) col 1, measurements col 2
        let _ = circuit.add_gate(ketgrid_core::GateType::H, vec![0], vec![], 0);
        let _ = circuit.add_gate(ketgrid_core::GateType::Cnot, vec![1], vec![0], 1);
        let _ = circuit.add_measurement(0, 2);
        let _ = circuit.add_measurement(1, 2);

        // Create simulator and apply the circuit
        let mut sim = StateVectorSimulator::new(circuit.num_qubits());
        sim.apply_circuit(&circuit);

        Self {
            simulator: Some(sim),
            circuit,
            gate_palette: GatePalette::default(),
            circuit_view: CircuitView::default(),
            state_view: StateView::default(),
            editor_state: EditorState::default(),
        }
    }

    /// Re-run simulation when circuit changes.
    fn refresh_simulation(&mut self) {
        let mut sim = StateVectorSimulator::new(self.circuit.num_qubits());
        sim.apply_circuit(&self.circuit);
        self.simulator = Some(sim);
    }

    /// Render the context menu if open.
    fn render_context_menu(&mut self, ctx: &egui::Context) {
        use crate::editor::ContextMenuType;

        // Take ownership of menu state temporarily to avoid borrow issues
        let menu_action = if let Some(ref menu_state) = self.editor_state.context_menu {
            let item_idx = menu_state.item_index;
            let item_type = menu_state.item_type;
            let menu_pos = menu_state.position;
            let has_clipboard = self.editor_state.has_clipboard_content();

            let mut action: Option<ContextMenuAction> = None;

            match item_type {
                ContextMenuType::Gate => {
                    // Get gate info
                    let gate_info: Option<(bool, PlacedGate)> =
                        self.circuit.gates.get(item_idx).map(|g| {
                            let is_parameterized = g.gate.is_parameterized();
                            (is_parameterized, g.clone())
                        });

                    egui::Window::new("Gate Menu")
                        .fixed_pos(menu_pos)
                        .fixed_size([140.0, 140.0])
                        .title_bar(false)
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if let Some((is_parameterized, gate)) = gate_info {
                                // Edit option (only for parameterized gates)
                                if is_parameterized {
                                    if ui.button("Edit Parameters...").clicked() {
                                        action = Some(ContextMenuAction::Edit(item_idx));
                                    }
                                    ui.separator();
                                }

                                if ui.button("Copy").clicked() {
                                    action = Some(ContextMenuAction::Copy(gate));
                                }

                                // Paste option (only if clipboard has content)
                                if has_clipboard {
                                    if ui.button("Paste Here").clicked() {
                                        action = Some(ContextMenuAction::Paste(item_idx));
                                    }
                                }

                                ui.separator();

                                if ui.button("Delete").clicked() {
                                    action = Some(ContextMenuAction::DeleteGate(item_idx));
                                }
                            }
                        });
                }
                ContextMenuType::Measurement => {
                    egui::Window::new("Measurement Menu")
                        .fixed_pos(menu_pos)
                        .fixed_size([120.0, 60.0])
                        .title_bar(false)
                        .resizable(false)
                        .collapsible(false)
                        .show(ctx, |ui| {
                            if ui.button("Delete").clicked() {
                                action = Some(ContextMenuAction::DeleteMeasurement(item_idx));
                            }
                        });
                }
            }

            action
        } else {
            None
        };

        // Execute the action outside the closure
        match menu_action {
            Some(ContextMenuAction::Edit(idx)) => {
                self.editor_state.start_editing_gate(idx);
                self.editor_state.close_context_menu();
            }
            Some(ContextMenuAction::Copy(gate)) => {
                self.editor_state.copy_gate(&gate);
                self.editor_state.close_context_menu();
            }
            Some(ContextMenuAction::Paste(idx)) => {
                self.handle_paste_at(idx);
                self.editor_state.close_context_menu();
            }
            Some(ContextMenuAction::DeleteGate(idx)) => {
                self.handle_delete_gate(idx);
                self.editor_state.close_context_menu();
            }
            Some(ContextMenuAction::DeleteMeasurement(idx)) => {
                self.handle_delete_measurement(idx);
                self.editor_state.close_context_menu();
            }
            None => {}
        }

        // Close context menu on click outside
        if ctx.input(|i| i.pointer.any_click()) {
            if let Some(ref menu_state) = self.editor_state.context_menu {
                // Check if click was outside the menu window
                if let Some(pointer_pos) = ctx.input(|i| i.pointer.interact_pos()) {
                    let menu_size = match menu_state.item_type {
                        ContextMenuType::Gate => egui::vec2(140.0, 140.0),
                        ContextMenuType::Measurement => egui::vec2(120.0, 60.0),
                    };
                    let menu_rect = egui::Rect::from_min_size(menu_state.position, menu_size);
                    if !menu_rect.contains(pointer_pos) {
                        self.editor_state.close_context_menu();
                    }
                }
            }
        }
    }

    /// Handle paste at a specific gate's position.
    fn handle_paste_at(&mut self, target_gate_idx: usize) {
        // Clone the clipboard content to avoid borrow issues
        let clipboard_content = self.editor_state.clipboard.clone();

        if let Some(clipboard) = clipboard_content {
            match clipboard {
                ClipboardContent::Single {
                    gate,
                    original_column: _,
                    original_qubits: _,
                } => {
                    if let Some(target_gate) = self.circuit.gates.get(target_gate_idx) {
                        let target_column = target_gate.column;
                        self.paste_gate_at_column(&gate, target_column);
                    }
                }
                ClipboardContent::Multiple(_) => {
                    // Multi-gate paste not yet implemented
                }
            }
        }
    }

    /// Paste a gate at a specific column, adjusting qubit positions relative to original.
    fn paste_gate_at_column(
        &mut self,
        gate: &PlacedGate,
        target_column: usize,
    ) {
        // For simplicity, paste at the same qubits as the original
        let target_qubits = gate.target_qubits.clone();
        let control_qubits = gate.control_qubits.clone();

        // Ensure qubits are within bounds
        let num_qubits = self.circuit.num_qubits();
        if target_qubits.iter().all(|&q| q < num_qubits)
            && control_qubits.iter().all(|&q| q < num_qubits)
        {
            let _ = self.circuit.add_gate(
                gate.gate.clone(),
                target_qubits,
                control_qubits,
                target_column,
            );
            self.refresh_simulation();
        }
    }

    /// Render parameter editor for parameterized gates.
    fn render_parameter_editor(&mut self, ctx: &egui::Context) {
        if let Some(gate_idx) = self.editor_state.editing_gate {
            let mut should_close = false;
            let mut should_apply = false;
            let mut new_theta: Option<f64> = None;

            egui::Window::new("Edit Gate Parameters")
                .fixed_size([250.0, 120.0])
                .resizable(false)
                .collapsible(false)
                .show(ctx, |ui| {
                    if let Some(gate) = self.circuit.gates.get(gate_idx) {
                        ui.label(format!("Editing: {}", gate.gate.display_name()));
                        ui.separator();

                        // Extract current theta value
                        let current_theta = match gate.gate {
                            GateType::Rx(theta) | GateType::Ry(theta) | GateType::Rz(theta) => theta,
                            _ => 0.0,
                        };

                        let mut theta_degrees = current_theta.to_degrees();
                        ui.horizontal(|ui| {
                            ui.label("Angle (°):");
                            ui.add(egui::Slider::new(&mut theta_degrees, -360.0..=360.0));
                        });

                        new_theta = Some(theta_degrees.to_radians());

                        ui.separator();

                        ui.horizontal(|ui| {
                            if ui.button("Apply").clicked() {
                                should_apply = true;
                                should_close = true;
                            }
                            if ui.button("Cancel").clicked() {
                                should_close = true;
                            }
                        });
                    } else {
                        // Gate no longer exists
                        should_close = true;
                    }
                });

            if should_apply {
                if let Some(theta) = new_theta {
                    if let Some(gate) = self.circuit.gates.get(gate_idx) {
                        let new_gate_type = match gate.gate {
                            GateType::Rx(_) => GateType::Rx(theta),
                            GateType::Ry(_) => GateType::Ry(theta),
                            GateType::Rz(_) => GateType::Rz(theta),
                            _ => gate.gate.clone(),
                        };
                        let _ = self.circuit.update_gate_parameters(gate_idx, new_gate_type);
                        self.refresh_simulation();
                    }
                }
            }

            if should_close {
                self.editor_state.stop_editing_gate();
            }
        }
    }

    /// Handle deleting a gate.
    fn handle_delete_gate(&mut self, gate_idx: usize) {
        if self.circuit.remove_gate(gate_idx).is_some() {
            // Clear selection if the deleted gate was selected
            self.editor_state.clear_selection();
            self.refresh_simulation();
        }
    }

    /// Handle deleting a measurement.
    fn handle_delete_measurement(&mut self, measurement_idx: usize) {
        if self.circuit.remove_measurement(measurement_idx).is_some() {
            self.refresh_simulation();
        }
    }

    /// Handle paste operation (uses clipboard content at current mouse position).
    fn handle_paste(&mut self) {
        // Clone the clipboard content to avoid borrow issues
        let clipboard_content = self.editor_state.clipboard.clone();

        if let Some(clipboard) = clipboard_content {
            match clipboard {
                ClipboardContent::Single { gate, .. } => {
                    // Find the gate at the selected position or use last selected gate position
                    let target_column = if let Some(selected) = self.editor_state.last_selected_gate
                    {
                        self.circuit.gates.get(selected.index).map(|g| g.column)
                    } else {
                        self.circuit.gates.last().map(|g| g.column + 1)
                    };

                    if let Some(column) = target_column {
                        self.paste_gate_at_column(&gate, column);
                    }
                }
                ClipboardContent::Multiple(_) => {
                    // Multi-gate paste not yet implemented
                }
            }
        }
    }
}

impl eframe::App for KetGridApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Circuit").clicked() {
                        self.circuit = Circuit::new(3);
                        self.editor_state.cancel_pending();
                        self.gate_palette.clear_selection();
                        self.refresh_simulation();
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        // TODO: File dialog
                        ui.close_menu();
                    }
                    if ui.button("Save").clicked() {
                        // TODO: Save circuit
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Add Qubit").clicked() {
                        self.circuit.add_qubit();
                        self.refresh_simulation();
                        ui.close_menu();
                    }
                });
            });
        });

        // Left panel: Gate palette
        egui::SidePanel::left("gate_palette")
            .resizable(true)
            .default_width(180.0)
            .show(ctx, |ui| {
                ui.heading("Gates");
                ui.separator();
                self.gate_palette.show(ui);
            });

        // Cancel placement on Escape
        if ctx.input(|i| i.key_pressed(egui::Key::Escape)) {
            self.editor_state.cancel_pending();
            self.gate_palette.clear_selection();
        }

        // Cancel pending if palette selection changed to a different gate
        if let Some(ref pending) = self.editor_state.multi_qubit_pending {
            let mismatch = self
                .gate_palette
                .selected_gate()
                .map_or(true, |g| *g != pending.gate);
            if mismatch {
                self.editor_state.cancel_pending();
            }
        }

        // Determine the active item for placement (pending takes priority)
        let (active_gate, is_measurement_mode) = if self.editor_state.is_awaiting_more_qubits() {
            (
                self.editor_state
                    .multi_qubit_pending
                    .as_ref()
                    .map(|p| p.gate.clone()),
                false,
            )
        } else {
            match self.gate_palette.selected() {
                Some(PaletteSelection::Gate(gate)) => (Some(gate.clone()), false),
                Some(PaletteSelection::Measurement) => (None, true),
                None => (None, false),
            }
        };

        let status_text = if is_measurement_mode {
            "Click wire to place measurement".to_string()
        } else {
            self.editor_state.status_text(active_gate.as_ref())
        };

        // Right panel: State view
        egui::SidePanel::right("state_view")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.heading("State");
                ui.separator();
                if let Some(ref sim) = self.simulator {
                    self.state_view.show(ui, sim.state());
                }
            });

        // Central panel: Circuit editor with drop zones
        let mut clicked_target: Option<DropTarget> = None;
        let mut gate_outcome = None;
        let mut measurement_outcome = None;
        egui::CentralPanel::default().show(ctx, |ui| {
            let (target, outcome, meas_outcome) = self.circuit_view.show(
                ui,
                &self.circuit,
                active_gate.as_ref(),
                is_measurement_mode,
                &self.editor_state,
            );
            clicked_target = target;
            gate_outcome = outcome;
            measurement_outcome = meas_outcome;
        });

        // Handle gate clicks (selection, context menu)
        if let Some(outcome) = gate_outcome {
            match (outcome.hit, outcome.interaction) {
                (GateHitResult::Gate(index), GateInteraction::RightClick) => {
                    // Open context menu for gate
                    self.editor_state.open_context_menu(outcome.position, index, false);
                }
                (GateHitResult::Gate(index), GateInteraction::LeftClick) => {
                    // Select the gate
                    self.editor_state.toggle_gate_selection(index, false);
                }
                (GateHitResult::Gate(index), GateInteraction::CtrlClick) => {
                    // Toggle selection
                    self.editor_state.toggle_gate_selection(index, true);
                }
                (GateHitResult::Empty, GateInteraction::LeftClick) => {
                    // Clear selection when clicking empty space
                    self.editor_state.clear_selection();
                }
                _ => {}
            }
        }

        // Handle measurement clicks (context menu only)
        if let Some(meas_idx) = measurement_outcome {
            // Right-click on measurement - open context menu
            if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                self.editor_state.open_context_menu(pos, meas_idx, true);
            }
        }

        // Render context menu if open
        self.render_context_menu(ctx);

        // Render parameter editor if editing
        self.render_parameter_editor(ctx);

        // Handle measurement placement
        if is_measurement_mode {
            if let Some(target) = clicked_target {
                let _ = self
                    .circuit
                    .add_measurement(target.qubit_idx, target.column);
                self.gate_palette.clear_selection();
                self.refresh_simulation();
            }

            // Handle drag-drop for measurement
            if clicked_target.is_none() {
                let released = ctx.input(|i| i.pointer.any_released());
                if released {
                    if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                        if let Some(target) =
                            self.circuit_view.hit_test(pos, self.circuit.num_qubits())
                        {
                            let _ = self
                                .circuit
                                .add_measurement(target.qubit_idx, target.column);
                            self.gate_palette.clear_selection();
                            self.refresh_simulation();
                        }
                    }
                }
            }
        } else if let (Some(target), Some(gate)) = (clicked_target, &active_gate) {
            // Handle gate placement from circuit view click
            if let Some(placement) = self.editor_state.try_place(gate, target) {
                let _ = self.circuit.add_gate(
                    placement.gate,
                    placement.target_qubits,
                    placement.control_qubits,
                    placement.column,
                );
                if !self.editor_state.is_awaiting_more_qubits() {
                    self.gate_palette.clear_selection();
                }
                self.refresh_simulation();
            }
        }

        // Handle cross-panel drag-drop for gates (pointer released after dragging from palette)
        if clicked_target.is_none() && !is_measurement_mode {
            if let Some(ref gate) = active_gate {
                let released = ctx.input(|i| i.pointer.any_released());
                if released {
                    if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                        if let Some(target) =
                            self.circuit_view.hit_test(pos, self.circuit.num_qubits())
                        {
                            if let Some(placement) = self.editor_state.try_place(gate, target) {
                                let _ = self.circuit.add_gate(
                                    placement.gate,
                                    placement.target_qubits,
                                    placement.control_qubits,
                                    placement.column,
                                );
                                if !self.editor_state.is_awaiting_more_qubits() {
                                    self.gate_palette.clear_selection();
                                }
                                self.refresh_simulation();
                            }
                        }
                    }
                }
            }
        }

        // Handle paste on Ctrl+V
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
            self.handle_paste();
        }

        // Status bar with metrics
        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                let num_qubits = self.circuit.num_qubits();
                let num_gates = self.circuit.gates.len();
                let memory_bytes = estimate_state_vector_memory_bytes(num_qubits);

                ui.label(format!("{} qubits", num_qubits));
                ui.separator();
                ui.label(format!("{} gates", num_gates));
                ui.separator();

                let memory_text = format!("~{} state vector", format_memory(memory_bytes));
                let memory_color = if let Some((available, _)) = get_system_memory() {
                    if memory_bytes > 0 && available > 0 && memory_bytes > (available * 9 / 10) {
                        ui.ctx().request_repaint();
                        Some(egui::Color32::from_rgb(255, 100, 100))
                    } else {
                        None
                    }
                } else {
                    if num_qubits > 30 {
                        Some(egui::Color32::from_rgb(255, 200, 100))
                    } else {
                        None
                    }
                };

                if let Some(color) = memory_color {
                    ui.colored_label(color, format!("! {}", memory_text));
                } else {
                    ui.label(memory_text);
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(&status_text);
                });
            });
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_estimate_small_circuits() {
        // 1 qubit: 2^1 * 16 = 32 bytes
        assert_eq!(estimate_state_vector_memory_bytes(1), 32);
        // 2 qubits: 4 * 16 = 64 bytes
        assert_eq!(estimate_state_vector_memory_bytes(2), 64);
        // 3 qubits: 8 * 16 = 128 bytes
        assert_eq!(estimate_state_vector_memory_bytes(3), 128);
    }

    #[test]
    fn test_memory_estimate_12_qubits() {
        // 12 qubits: 2^12 * 16 = 4096 * 16 = 65536 bytes = ~64KB
        let bytes = estimate_state_vector_memory_bytes(12);
        assert_eq!(bytes, 65536);
        assert_eq!(format_memory(bytes), "64 KB");
    }

    #[test]
    fn test_format_memory_units() {
        // Bytes
        assert_eq!(format_memory(100), "100 B");
        // Kilobytes
        assert_eq!(format_memory(1024), "1 KB");
        assert_eq!(format_memory(1536), "2 KB");
        // Megabytes
        assert_eq!(format_memory(1024 * 1024), "1.0 MB");
        assert_eq!(format_memory(5 * 1024 * 1024), "5.0 MB");
        // Gigabytes
        assert_eq!(format_memory(1024 * 1024 * 1024), "1.0 GB");
        assert_eq!(format_memory(16 * 1024 * 1024 * 1024), "16.0 GB");
    }

    #[test]
    fn test_memory_estimate_overflow_protection() {
        // Very large qubit counts should not panic
        let bytes = estimate_state_vector_memory_bytes(64);
        assert_eq!(bytes, usize::MAX);
    }
}
