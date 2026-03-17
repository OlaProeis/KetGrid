//! Circuit editor view with qubit wires, gate rendering, and drop-zone interaction.

use ketgrid_core::gate::GateType;
use ketgrid_core::{Circuit, PlacedGate};

use crate::editor::{DropTarget, EditorState};

/// Result of a gate hit-test: which gate and what type of interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateHitResult {
    /// A gate was hit at this index.
    Gate(usize),
    /// Empty space was clicked.
    Empty,
}

/// Interaction type for gate clicks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateInteraction {
    /// Left click (for selection).
    LeftClick,
    /// Right click (for context menu).
    RightClick,
    /// Ctrl+Click (for multi-select).
    CtrlClick,
}

/// Outcome of a gate interaction.
#[derive(Debug, Clone)]
pub struct GateClickOutcome {
    pub hit: GateHitResult,
    pub interaction: GateInteraction,
    pub position: egui::Pos2,
}

const WIRE_SPACING: f32 = 60.0;
const WIRE_MARGIN_TOP: f32 = 40.0;
const LABEL_WIDTH: f32 = 55.0;
const COLUMN_WIDTH: f32 = 64.0;
const GATE_BOX_SIZE: f32 = 36.0;
const CONTROL_DOT_RADIUS: f32 = 5.0;
const TARGET_CIRCLE_RADIUS: f32 = 14.0;
const SWAP_CROSS_SIZE: f32 = 8.0;
const MEASUREMENT_BOX_SIZE: f32 = 32.0;

const DROP_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgba_premultiplied(100, 150, 255, 80);
const DROP_BORDER: egui::Color32 = egui::Color32::from_rgb(100, 150, 255);
const PENDING_HIGHLIGHT: egui::Color32 =
    egui::Color32::from_rgba_premultiplied(100, 200, 100, 100);
const PENDING_BORDER: egui::Color32 = egui::Color32::from_rgb(80, 180, 80);
const WIRE_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgba_premultiplied(100, 150, 255, 25);
const CONNECTING_LINE: egui::Color32 = egui::Color32::from_rgba_premultiplied(100, 150, 255, 120);
const SELECTION_HIGHLIGHT: egui::Color32 = egui::Color32::from_rgb(255, 200, 100);
const GATE_HIT_PADDING: f32 = 8.0; // Extra padding for gate hit-testing
const STEP_CURSOR_COLOR: egui::Color32 = egui::Color32::from_rgb(0, 200, 255);
const STEP_DIM_OVERLAY: egui::Color32 = egui::Color32::from_rgba_premultiplied(0, 0, 0, 50);

/// Circuit visualization panel with drag-and-drop support.
#[derive(Debug, Default)]
pub struct CircuitView {
    wire_y_start: f32,
    column_start_x: f32,
    last_rect: Option<egui::Rect>,
    /// Gate bounding boxes for this frame (index -> rect).
    gate_rects: Vec<(usize, egui::Rect)>,
    /// Measurement bounding boxes for this frame (index -> rect).
    measurement_rects: Vec<(usize, egui::Rect)>,
}

impl CircuitView {
    /// Renders the circuit editor with optional drop indicators.
    ///
    /// `step_cursor_col`: if `Some(n)`, draws a vertical step cursor at column `n`.
    /// Gates at columns `< n` render normally; gates at columns `>= n` are dimmed.
    ///
    /// `wire_colors`: per-qubit entanglement colors. `None` entries use the default
    /// wire stroke; `Some(color)` draws that wire (and label) in the cluster color.
    ///
    /// Returns:
    /// - The clicked drop target when the user clicks on a valid wire position while a gate is active
    /// - Gate click outcomes (for selection, context menu, etc.)
    /// - Measurement index if a measurement was right-clicked
    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        circuit: &Circuit,
        active_gate: Option<&GateType>,
        is_measurement_mode: bool,
        editor_state: &EditorState,
        step_cursor_col: Option<usize>,
        wire_colors: &[Option<egui::Color32>],
    ) -> (Option<DropTarget>, Option<GateClickOutcome>, Option<usize>) {
        let available_size = ui.available_size();
        let (rect, response) = ui.allocate_exact_size(available_size, egui::Sense::click());

        self.last_rect = Some(rect);
        self.wire_y_start = rect.top() + WIRE_MARGIN_TOP;
        self.column_start_x = rect.left() + LABEL_WIDTH + 20.0;
        self.gate_rects.clear();
        self.measurement_rects.clear();

        let painter = ui.painter_at(rect);
        painter.rect_filled(rect, 0.0, ui.visuals().extreme_bg_color);

        if circuit.num_qubits() == 0 {
            painter.text(
                rect.center(),
                egui::Align2::CENTER_CENTER,
                "Empty circuit - add qubits to begin",
                egui::FontId::proportional(16.0),
                ui.visuals().text_color(),
            );
            return (None, None, None);
        }

        let wire_y_start = self.wire_y_start;
        let column_start_x = self.column_start_x;
        let wire_stroke = ui.visuals().widgets.noninteractive.bg_stroke;

        for qubit_idx in 0..circuit.num_qubits() {
            let y = wire_y(qubit_idx, wire_y_start);

            let ent_color = wire_colors.get(qubit_idx).and_then(|c| *c);

            let label_color = ent_color.unwrap_or_else(|| ui.visuals().text_color());
            painter.text(
                egui::pos2(rect.left() + 8.0, y),
                egui::Align2::LEFT_CENTER,
                &circuit.qubits[qubit_idx].label,
                egui::FontId::proportional(14.0),
                label_color,
            );

            // Colored dot for entangled qubits
            if let Some(color) = ent_color {
                painter.circle_filled(
                    egui::pos2(column_start_x - 16.0, y),
                    4.0,
                    color,
                );
            }

            let qubit_stroke = if let Some(color) = ent_color {
                egui::Stroke::new(2.5, color)
            } else {
                wire_stroke
            };

            painter.hline(
                column_start_x - 10.0..=rect.right() - 10.0,
                y,
                qubit_stroke,
            );
        }

        // Draw gates and store their bounding boxes
        for (idx, gate) in circuit.gates.iter().enumerate() {
            let cx = column_x(gate.column, column_start_x);
            let gate_rect = draw_gate_with_rect(&painter, ui, gate, cx, wire_y_start, editor_state.is_gate_selected(idx));
            self.gate_rects.push((idx, gate_rect));
        }

        // Draw measurements and store their bounding boxes
        for (idx, meas) in circuit.measurements.iter().enumerate() {
            if meas.qubit_id < circuit.num_qubits() {
                let cx = column_x(meas.column, column_start_x);
                let cy = wire_y(meas.qubit_id, wire_y_start);
                let meas_rect = draw_measurement_with_rect(&painter, ui, egui::pos2(cx, cy));
                self.measurement_rects.push((idx, meas_rect));
            }
        }

        // === Step cursor and dim overlay ===
        if let Some(cursor_col) = step_cursor_col {
            let num_q = circuit.num_qubits();
            let y_top = wire_y(0, wire_y_start) - WIRE_SPACING / 2.0;
            let y_bot = wire_y(num_q - 1, wire_y_start) + WIRE_SPACING / 2.0;
            let cursor_x = column_start_x + cursor_col as f32 * COLUMN_WIDTH;

            // Dim the "future" region (gates not yet applied)
            if cursor_x < rect.right() {
                let dim_rect = egui::Rect::from_min_max(
                    egui::pos2(cursor_x, rect.top()),
                    rect.right_bottom(),
                );
                painter.rect_filled(dim_rect, 0.0, STEP_DIM_OVERLAY);
            }

            // Draw the cursor line
            painter.line_segment(
                [egui::pos2(cursor_x, y_top), egui::pos2(cursor_x, y_bot)],
                egui::Stroke::new(2.5, STEP_CURSOR_COLOR),
            );

            // Small triangle indicator at the top
            let tri_size = 6.0;
            let tri_points = vec![
                egui::pos2(cursor_x, y_top - tri_size * 2.0),
                egui::pos2(cursor_x - tri_size, y_top - tri_size * 2.0 - tri_size),
                egui::pos2(cursor_x + tri_size, y_top - tri_size * 2.0 - tri_size),
            ];
            painter.add(egui::Shape::convex_polygon(
                tri_points,
                STEP_CURSOR_COLOR,
                egui::Stroke::NONE,
            ));
        }

        // === Gate interaction (selection, context menu) ===
        let mut gate_outcome: Option<GateClickOutcome> = None;
        let mut measurement_outcome: Option<usize> = None;

        // Check for gate clicks (only when not placing a gate or measurement)
        let is_placing = active_gate.is_some()
            || editor_state.is_awaiting_more_qubits()
            || is_measurement_mode;

        if !is_placing {
            // Handle right-click (context menu for gates or measurements)
            if response.secondary_clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    // Check gates first
                    if let Some(gate_idx) = self.hit_test_gate(pos) {
                        gate_outcome = Some(GateClickOutcome {
                            hit: GateHitResult::Gate(gate_idx),
                            interaction: GateInteraction::RightClick,
                            position: pos,
                        });
                    } else if let Some(meas_idx) = self.hit_test_measurement(pos) {
                        // Right-clicked on a measurement
                        measurement_outcome = Some(meas_idx);
                    }
                }
            }

            // Handle left-click on gates (selection)
            if response.clicked() {
                if let Some(pos) = response.interact_pointer_pos() {
                    if let Some(gate_idx) = self.hit_test_gate(pos) {
                        let ctrl_pressed = ui.ctx().input(|i| i.modifiers.ctrl);
                        let interaction = if ctrl_pressed {
                            GateInteraction::CtrlClick
                        } else {
                            GateInteraction::LeftClick
                        };
                        gate_outcome = Some(GateClickOutcome {
                            hit: GateHitResult::Gate(gate_idx),
                            interaction,
                            position: pos,
                        });
                    } else {
                        // Clicked on empty space - clear selection
                        gate_outcome = Some(GateClickOutcome {
                            hit: GateHitResult::Empty,
                            interaction: GateInteraction::LeftClick,
                            position: pos,
                        });
                    }
                }
            }
        }

        // === Drop zone interaction ===
        let mut hover_target: Option<DropTarget> = None;

        if is_placing {
            if let Some(pos) = response.hover_pos() {
                hover_target =
                    hit_test_at(pos, wire_y_start, column_start_x, circuit.num_qubits());

                // Snap column to pending column for multi-qubit gates
                if let Some(ref pending) = editor_state.multi_qubit_pending {
                    if let Some(ref mut target) = hover_target {
                        target.column = pending.column;
                    }
                }
            }

            if let Some(ref target) = hover_target {
                // Subtle highlight on the entire target wire
                let y = wire_y(target.qubit_idx, wire_y_start);
                let wire_rect = egui::Rect::from_min_max(
                    egui::pos2(column_start_x - 10.0, y - WIRE_SPACING / 4.0),
                    egui::pos2(rect.right() - 10.0, y + WIRE_SPACING / 4.0),
                );
                painter.rect_filled(wire_rect, 2.0, WIRE_HIGHLIGHT);

                // Ghost gate box or measurement at the drop position
                if is_measurement_mode {
                    draw_measurement_indicator(&painter, target, wire_y_start, column_start_x);
                } else {
                    let label = active_gate.map(gate_display_label);
                    draw_drop_indicator(
                        &painter,
                        target,
                        wire_y_start,
                        column_start_x,
                        label.as_deref(),
                    );
                }

                // Vertical connecting line from pending qubits to hover target
                if let Some(ref pending) = editor_state.multi_qubit_pending {
                    let cx = column_x(pending.column, column_start_x);
                    let target_y = wire_y(target.qubit_idx, wire_y_start);
                    for &q in &pending.selected_qubits {
                        let pending_y = wire_y(q, wire_y_start);
                        painter.line_segment(
                            [egui::pos2(cx, pending_y), egui::pos2(cx, target_y)],
                            egui::Stroke::new(1.5, CONNECTING_LINE),
                        );
                    }
                }

                ui.ctx().set_cursor_icon(egui::CursorIcon::Crosshair);
            }

            // Only return drop target if we're placing and clicked
            if response.clicked() && hover_target.is_some() {
                return (hover_target, None, None);
            }
        }

        // Highlight already-selected qubits for pending multi-qubit gate
        if let Some(ref pending) = editor_state.multi_qubit_pending {
            for &q in &pending.selected_qubits {
                draw_pending_qubit_indicator(
                    &painter,
                    q,
                    pending.column,
                    wire_y_start,
                    column_start_x,
                );
            }
        }

        // Hover border when not in placement mode
        if !is_placing && response.hovered() {
            painter.rect_stroke(
                rect.shrink(2.0),
                4.0,
                egui::Stroke::new(2.0, ui.visuals().selection.stroke.color),
                egui::StrokeKind::Middle,
            );
        }

        (None, gate_outcome, measurement_outcome)
    }

    /// Hit-test a position against gate bounding boxes.
    /// Returns the index of the gate if hit.
    pub fn hit_test_gate(&self, pos: egui::Pos2) -> Option<usize> {
        for (idx, rect) in &self.gate_rects {
            if rect.expand(GATE_HIT_PADDING).contains(pos) {
                return Some(*idx);
            }
        }
        None
    }

    /// Hit-test a position against measurement bounding boxes.
    /// Returns the index of the measurement if hit.
    pub fn hit_test_measurement(&self, pos: egui::Pos2) -> Option<usize> {
        for (idx, rect) in &self.measurement_rects {
            if rect.expand(GATE_HIT_PADDING).contains(pos) {
                return Some(*idx);
            }
        }
        None
    }

    /// Hit-test a global position against the circuit grid.
    /// Used for cross-panel drag-drop detection.
    pub fn hit_test(&self, pos: egui::Pos2, num_qubits: usize) -> Option<DropTarget> {
        let rect = self.last_rect?;
        if !rect.contains(pos) {
            return None;
        }
        hit_test_at(pos, self.wire_y_start, self.column_start_x, num_qubits)
    }
}

/// Maps a screen position to the nearest valid wire/column on the circuit grid.
fn hit_test_at(
    pos: egui::Pos2,
    wire_y_start: f32,
    column_start_x: f32,
    num_qubits: usize,
) -> Option<DropTarget> {
    if num_qubits == 0 {
        return None;
    }

    let y_min = wire_y_start - WIRE_SPACING / 2.0;
    let y_max = wire_y_start + (num_qubits - 1) as f32 * WIRE_SPACING + WIRE_SPACING / 2.0;
    if pos.y < y_min || pos.y > y_max || pos.x < column_start_x - COLUMN_WIDTH / 2.0 {
        return None;
    }

    let relative_y = pos.y - wire_y_start;
    let qubit_f = relative_y / WIRE_SPACING;
    let qubit_idx = qubit_f.round().clamp(0.0, (num_qubits - 1) as f32) as usize;

    let relative_x = pos.x - column_start_x;
    let column = if relative_x < 0.0 {
        0
    } else {
        (relative_x / COLUMN_WIDTH) as usize
    };

    Some(DropTarget { qubit_idx, column })
}

/// Draws a ghost gate box at the prospective drop position.
fn draw_drop_indicator(
    painter: &egui::Painter,
    target: &DropTarget,
    wire_y_start: f32,
    column_start_x: f32,
    label: Option<&str>,
) {
    let cx = column_x(target.column, column_start_x);
    let cy = wire_y(target.qubit_idx, wire_y_start);
    let center = egui::pos2(cx, cy);

    let width = match label {
        Some(l) if l.len() > 3 => GATE_BOX_SIZE + 16.0,
        _ => GATE_BOX_SIZE,
    };
    let ghost_rect = egui::Rect::from_center_size(center, egui::vec2(width, GATE_BOX_SIZE));

    painter.rect_filled(ghost_rect, 4.0, DROP_HIGHLIGHT);
    painter.rect_stroke(
        ghost_rect,
        4.0,
        egui::Stroke::new(2.0, DROP_BORDER),
        egui::StrokeKind::Middle,
    );

    if let Some(label) = label {
        painter.text(
            center,
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(13.0),
            DROP_BORDER,
        );
    }
}

/// Draws a green indicator on an already-selected qubit for multi-qubit placement.
fn draw_pending_qubit_indicator(
    painter: &egui::Painter,
    qubit_idx: usize,
    column: usize,
    wire_y_start: f32,
    column_start_x: f32,
) {
    let cx = column_x(column, column_start_x);
    let cy = wire_y(qubit_idx, wire_y_start);
    let center = egui::pos2(cx, cy);

    let indicator_rect =
        egui::Rect::from_center_size(center, egui::vec2(GATE_BOX_SIZE, GATE_BOX_SIZE));
    painter.rect_filled(indicator_rect, 4.0, PENDING_HIGHLIGHT);
    painter.rect_stroke(
        indicator_rect,
        4.0,
        egui::Stroke::new(2.0, PENDING_BORDER),
        egui::StrokeKind::Middle,
    );
    painter.circle_filled(center, CONTROL_DOT_RADIUS, PENDING_BORDER);
}

// =========================================================================
// Layout helpers
// =========================================================================

fn wire_y(qubit_idx: usize, wire_y_start: f32) -> f32 {
    wire_y_start + qubit_idx as f32 * WIRE_SPACING
}

fn column_x(column: usize, column_start_x: f32) -> f32 {
    column_start_x + column as f32 * COLUMN_WIDTH + COLUMN_WIDTH / 2.0
}

// =========================================================================
// Gate rendering (unchanged from original)
// =========================================================================

/// Draw a gate and return its bounding rect.
/// If `selected` is true, draws a selection highlight around the gate.
fn draw_gate_with_rect(
    painter: &egui::Painter,
    ui: &egui::Ui,
    gate: &PlacedGate,
    cx: f32,
    wire_y_start: f32,
    selected: bool,
) -> egui::Rect {
    let gate_rect = match &gate.gate {
        GateType::Cnot | GateType::Toffoli => {
            draw_controlled_not(painter, ui, gate, cx, wire_y_start, selected)
        }
        GateType::Cz => {
            draw_cz(painter, ui, gate, cx, wire_y_start, selected)
        }
        GateType::Swap => {
            draw_swap(painter, ui, gate, cx, wire_y_start, selected)
        }
        GateType::Barrier => {
            let mut min_y = f32::MAX;
            let mut max_y = f32::MIN;
            for &q in &gate.target_qubits {
                let y = wire_y(q, wire_y_start);
                min_y = min_y.min(y);
                max_y = max_y.max(y);
                let half = WIRE_SPACING / 2.0;
                let dash_len = 4.0;
                let mut py = y - half;
                let end_y = y + half;
                let color = ui.visuals().text_color().gamma_multiply(0.3);
                while py < end_y {
                    let seg_end = (py + dash_len).min(end_y);
                    painter.line_segment(
                        [egui::pos2(cx, py), egui::pos2(cx, seg_end)],
                        egui::Stroke::new(1.5, color),
                    );
                    py += dash_len * 2.0;
                }
            }
            // Return approximate rect for barrier
            if min_y < f32::MAX {
                egui::Rect::from_min_max(
                    egui::pos2(cx - GATE_BOX_SIZE / 2.0, min_y - GATE_BOX_SIZE / 2.0),
                    egui::pos2(cx + GATE_BOX_SIZE / 2.0, max_y + GATE_BOX_SIZE / 2.0),
                )
            } else {
                egui::Rect::from_center_size(egui::pos2(cx, wire_y_start), egui::vec2(GATE_BOX_SIZE, GATE_BOX_SIZE))
            }
        }
        _ => {
            if let Some(&target_q) = gate.target_qubits.first() {
                let cy = wire_y(target_q, wire_y_start);
                let label = gate_display_label(&gate.gate);
                draw_gate_box(painter, ui, egui::pos2(cx, cy), &label, selected)
            } else {
                egui::Rect::from_center_size(egui::pos2(cx, wire_y_start), egui::vec2(GATE_BOX_SIZE, GATE_BOX_SIZE))
            }
        }
    };

    gate_rect
}

fn gate_display_label(gate: &GateType) -> String {
    match gate {
        GateType::Rx(theta) => format!("Rx({:.1})", theta),
        GateType::Ry(theta) => format!("Ry({:.1})", theta),
        GateType::Rz(theta) => format!("Rz({:.1})", theta),
        GateType::Cnot | GateType::Toffoli => "+".to_string(),
        _ => gate.display_name(),
    }
}

fn draw_gate_box(
    painter: &egui::Painter,
    ui: &egui::Ui,
    center: egui::Pos2,
    label: &str,
    selected: bool,
) -> egui::Rect {
    let width = if label.len() > 3 {
        GATE_BOX_SIZE + 16.0
    } else {
        GATE_BOX_SIZE
    };
    let gate_rect = egui::Rect::from_center_size(center, egui::vec2(width, GATE_BOX_SIZE));

    // Draw selection highlight if selected
    if selected {
        let selection_rect = gate_rect.expand(4.0);
        painter.rect_filled(selection_rect, 6.0, SELECTION_HIGHLIGHT.gamma_multiply(0.3));
        painter.rect_stroke(
            selection_rect,
            6.0,
            egui::Stroke::new(2.0, SELECTION_HIGHLIGHT),
            egui::StrokeKind::Middle,
        );
    }

    painter.rect_filled(gate_rect, 4.0, ui.visuals().widgets.inactive.bg_fill);
    painter.rect_stroke(
        gate_rect,
        4.0,
        ui.visuals().widgets.inactive.bg_stroke,
        egui::StrokeKind::Middle,
    );

    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(13.0),
        ui.visuals().text_color(),
    );

    gate_rect
}

/// ● on each control qubit, ⊕ on each target qubit, vertical line connecting them.
/// Returns the bounding rect of the gate.
fn draw_controlled_not(
    painter: &egui::Painter,
    ui: &egui::Ui,
    gate: &PlacedGate,
    cx: f32,
    wire_y_start: f32,
    selected: bool,
) -> egui::Rect {
    let all = gate.all_qubits();
    let (min_q, max_q) = qubit_extent(&all);

    let y_top = wire_y(min_q, wire_y_start);
    let y_bot = wire_y(max_q, wire_y_start);
    let line_color = ui.visuals().text_color();
    painter.line_segment(
        [egui::pos2(cx, y_top), egui::pos2(cx, y_bot)],
        egui::Stroke::new(2.0, line_color),
    );

    for &q in &gate.control_qubits {
        draw_control_dot(painter, ui, egui::pos2(cx, wire_y(q, wire_y_start)));
    }

    for &q in &gate.target_qubits {
        draw_oplus(painter, ui, egui::pos2(cx, wire_y(q, wire_y_start)));
    }

    // Calculate bounding rect
    let gate_rect = egui::Rect::from_min_max(
        egui::pos2(cx - TARGET_CIRCLE_RADIUS - 2.0, y_top - TARGET_CIRCLE_RADIUS),
        egui::pos2(cx + TARGET_CIRCLE_RADIUS + 2.0, y_bot + TARGET_CIRCLE_RADIUS),
    );

    // Draw selection highlight
    if selected {
        let selection_rect = gate_rect.expand(4.0);
        painter.rect_filled(selection_rect, 6.0, SELECTION_HIGHLIGHT.gamma_multiply(0.3));
        painter.rect_stroke(
            selection_rect,
            6.0,
            egui::Stroke::new(2.0, SELECTION_HIGHLIGHT),
            egui::StrokeKind::Middle,
        );
    }

    gate_rect
}

/// ● on both control and target qubits, vertical line connecting them.
/// Returns the bounding rect of the gate.
fn draw_cz(
    painter: &egui::Painter,
    ui: &egui::Ui,
    gate: &PlacedGate,
    cx: f32,
    wire_y_start: f32,
    selected: bool,
) -> egui::Rect {
    let all = gate.all_qubits();
    let (min_q, max_q) = qubit_extent(&all);

    let y_top = wire_y(min_q, wire_y_start);
    let y_bot = wire_y(max_q, wire_y_start);
    let line_color = ui.visuals().text_color();
    painter.line_segment(
        [egui::pos2(cx, y_top), egui::pos2(cx, y_bot)],
        egui::Stroke::new(2.0, line_color),
    );

    for &q in gate.control_qubits.iter().chain(gate.target_qubits.iter()) {
        draw_control_dot(painter, ui, egui::pos2(cx, wire_y(q, wire_y_start)));
    }

    // Calculate bounding rect
    let gate_rect = egui::Rect::from_min_max(
        egui::pos2(cx - CONTROL_DOT_RADIUS - 4.0, y_top - CONTROL_DOT_RADIUS),
        egui::pos2(cx + CONTROL_DOT_RADIUS + 4.0, y_bot + CONTROL_DOT_RADIUS),
    );

    // Draw selection highlight
    if selected {
        let selection_rect = gate_rect.expand(4.0);
        painter.rect_filled(selection_rect, 6.0, SELECTION_HIGHLIGHT.gamma_multiply(0.3));
        painter.rect_stroke(
            selection_rect,
            6.0,
            egui::Stroke::new(2.0, SELECTION_HIGHLIGHT),
            egui::StrokeKind::Middle,
        );
    }

    gate_rect
}

/// ✕ on each target qubit, vertical line connecting them.
/// Returns the bounding rect of the gate.
fn draw_swap(
    painter: &egui::Painter,
    ui: &egui::Ui,
    gate: &PlacedGate,
    cx: f32,
    wire_y_start: f32,
    selected: bool,
) -> egui::Rect {
    let (min_q, max_q) = qubit_extent(&gate.target_qubits);

    let y_top = wire_y(min_q, wire_y_start);
    let y_bot = wire_y(max_q, wire_y_start);
    let line_color = ui.visuals().text_color();
    painter.line_segment(
        [egui::pos2(cx, y_top), egui::pos2(cx, y_bot)],
        egui::Stroke::new(2.0, line_color),
    );

    for &q in &gate.target_qubits {
        draw_swap_cross(painter, ui, egui::pos2(cx, wire_y(q, wire_y_start)));
    }

    // Calculate bounding rect
    let gate_rect = egui::Rect::from_min_max(
        egui::pos2(cx - SWAP_CROSS_SIZE - 4.0, y_top - SWAP_CROSS_SIZE),
        egui::pos2(cx + SWAP_CROSS_SIZE + 4.0, y_bot + SWAP_CROSS_SIZE),
    );

    // Draw selection highlight
    if selected {
        let selection_rect = gate_rect.expand(4.0);
        painter.rect_filled(selection_rect, 6.0, SELECTION_HIGHLIGHT.gamma_multiply(0.3));
        painter.rect_stroke(
            selection_rect,
            6.0,
            egui::Stroke::new(2.0, SELECTION_HIGHLIGHT),
            egui::StrokeKind::Middle,
        );
    }

    gate_rect
}

fn draw_control_dot(painter: &egui::Painter, ui: &egui::Ui, center: egui::Pos2) {
    painter.circle_filled(center, CONTROL_DOT_RADIUS, ui.visuals().text_color());
}

/// Circle with a + inscribed (the XOR / NOT target symbol).
fn draw_oplus(painter: &egui::Painter, ui: &egui::Ui, center: egui::Pos2) {
    let color = ui.visuals().text_color();
    let stroke = egui::Stroke::new(2.0, color);
    let r = TARGET_CIRCLE_RADIUS;

    painter.circle_stroke(center, r, stroke);
    painter.line_segment(
        [
            egui::pos2(center.x - r, center.y),
            egui::pos2(center.x + r, center.y),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x, center.y - r),
            egui::pos2(center.x, center.y + r),
        ],
        stroke,
    );
}

fn draw_swap_cross(painter: &egui::Painter, ui: &egui::Ui, center: egui::Pos2) {
    let stroke = egui::Stroke::new(2.0, ui.visuals().text_color());
    let s = SWAP_CROSS_SIZE;
    painter.line_segment(
        [
            egui::pos2(center.x - s, center.y - s),
            egui::pos2(center.x + s, center.y + s),
        ],
        stroke,
    );
    painter.line_segment(
        [
            egui::pos2(center.x + s, center.y - s),
            egui::pos2(center.x - s, center.y + s),
        ],
        stroke,
    );
}

/// Meter icon: box with a semicircular arc and an arrow.
/// Returns the bounding rect of the measurement.
fn draw_measurement_with_rect(
    painter: &egui::Painter,
    ui: &egui::Ui,
    center: egui::Pos2,
) -> egui::Rect {
    let s = MEASUREMENT_BOX_SIZE / 2.0;
    let meas_rect =
        egui::Rect::from_center_size(center, egui::vec2(MEASUREMENT_BOX_SIZE, MEASUREMENT_BOX_SIZE));
    let color = ui.visuals().text_color();
    let stroke = egui::Stroke::new(1.5, color);

    painter.rect_filled(meas_rect, 3.0, ui.visuals().widgets.inactive.bg_fill);
    painter.rect_stroke(meas_rect, 3.0, stroke, egui::StrokeKind::Middle);

    let arc_center = egui::pos2(center.x, center.y + s * 0.15);
    let arc_radius = s * 0.55;
    let segments = 16;
    for i in 0..segments {
        let a0 = std::f32::consts::PI + (i as f32 / segments as f32) * std::f32::consts::PI;
        let a1 = std::f32::consts::PI + ((i + 1) as f32 / segments as f32) * std::f32::consts::PI;
        painter.line_segment(
            [
                egui::pos2(
                    arc_center.x + arc_radius * a0.cos(),
                    arc_center.y + arc_radius * a0.sin(),
                ),
                egui::pos2(
                    arc_center.x + arc_radius * a1.cos(),
                    arc_center.y + arc_radius * a1.sin(),
                ),
            ],
            stroke,
        );
    }

    let arrow_end = egui::pos2(center.x + s * 0.45, center.y - s * 0.45);
    painter.line_segment([arc_center, arrow_end], stroke);

    meas_rect
}

/// Draws a ghost measurement indicator at the prospective drop position.
fn draw_measurement_indicator(
    painter: &egui::Painter,
    target: &DropTarget,
    wire_y_start: f32,
    column_start_x: f32,
) {
    let cx = column_x(target.column, column_start_x);
    let cy = wire_y(target.qubit_idx, wire_y_start);
    let center = egui::pos2(cx, cy);

    let meas_rect = egui::Rect::from_center_size(
        center,
        egui::vec2(MEASUREMENT_BOX_SIZE, MEASUREMENT_BOX_SIZE),
    );

    painter.rect_filled(meas_rect, 3.0, DROP_HIGHLIGHT);
    painter.rect_stroke(
        meas_rect,
        3.0,
        egui::Stroke::new(2.0, DROP_BORDER),
        egui::StrokeKind::Middle,
    );

    // Draw simple "M" label
    painter.text(
        center,
        egui::Align2::CENTER_CENTER,
        "M",
        egui::FontId::proportional(14.0),
        DROP_BORDER,
    );
}

fn qubit_extent(qubits: &[usize]) -> (usize, usize) {
    let min = qubits.iter().copied().min().unwrap_or(0);
    let max = qubits.iter().copied().max().unwrap_or(0);
    (min, max)
}
