//! Main egui application shell with three-panel layout.

use std::sync::mpsc;
use std::time::{Duration, Instant};

use ketgrid_core::circuit::PlacedGate;
use ketgrid_core::gate::GateType;
use ketgrid_core::Circuit;
use ketgrid_sim::{EntanglementInfo, StateVectorSimulator, compute_entanglement_info};

use crate::circuit_view::{CircuitView, GateHitResult, GateInteraction};
use crate::editor::{ClipboardContent, DropTarget, EditorState};
use crate::examples::ExampleLibrary;
use crate::gate_palette::{GatePalette, PaletteSelection};
use crate::history::{EditHistory, EditOperation};
use crate::state_view::StateView;
use crate::stats_panel::StatsPanel;

/// Memory estimation constants for state vector simulation.
const BYTES_PER_COMPLEX: usize = 16; // Complex<f64> = 2 * 8 bytes

/// Debounce duration for auto-simulation after edits.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(100);

/// Maximum qubits for automatic simulation. Above this, require manual trigger.
const AUTO_SIM_MAX_QUBITS: usize = 15;

/// Distinct colors for entanglement clusters (up to 8 before cycling).
const ENTANGLEMENT_PALETTE: [egui::Color32; 8] = [
    egui::Color32::from_rgb(255, 100, 100), // Red
    egui::Color32::from_rgb(100, 160, 255), // Blue
    egui::Color32::from_rgb(100, 220, 100), // Green
    egui::Color32::from_rgb(200, 130, 255), // Purple
    egui::Color32::from_rgb(255, 180, 60),  // Orange
    egui::Color32::from_rgb(255, 120, 200), // Pink
    egui::Color32::from_rgb(60, 220, 200),  // Teal
    egui::Color32::from_rgb(220, 200, 80),  // Yellow
];

/// Maps entanglement clusters to per-qubit wire colors.
///
/// Entangled qubits get a cluster color from the palette; unentangled qubits get `None`.
fn entanglement_wire_colors(info: &EntanglementInfo) -> Vec<Option<egui::Color32>> {
    let n = info.qubit_purities.len();
    let mut wire_colors = vec![None; n];

    let mut color_idx = 0usize;
    for cluster in &info.clusters {
        if cluster.len() > 1 {
            let color = ENTANGLEMENT_PALETTE[color_idx % ENTANGLEMENT_PALETTE.len()];
            for &q in cluster {
                wire_colors[q] = Some(color);
            }
            color_idx += 1;
        }
    }

    wire_colors
}

/// Actions from the stepper toolbar (processed outside the UI closure).
enum StepperAction {
    Enter,
    Exit,
    Reset,
    Back,
    Forward,
    TogglePlay,
}

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
    /// Circuit statistics panel state.
    stats_panel: StatsPanel,
    /// Example library browser.
    example_library: ExampleLibrary,
    /// Editor state for drag-and-drop gate placement.
    editor_state: EditorState,
    /// Undo/redo history for circuit edits.
    history: EditHistory,
    /// Path to the currently open file (for Save vs Save As).
    current_file_path: Option<std::path::PathBuf>,
    /// Status message for file operations.
    file_status: Option<String>,
    /// Timestamp of the most recent circuit edit (for debounce).
    sim_dirty_since: Option<Instant>,
    /// Sender half of background simulation result channel.
    sim_result_tx: mpsc::Sender<StateVectorSimulator>,
    /// Receiver half of background simulation result channel.
    sim_result_rx: mpsc::Receiver<StateVectorSimulator>,
    /// Whether a background simulation thread is currently running.
    sim_running: bool,
    /// Whether displayed simulation results are outdated.
    sim_stale: bool,
    /// Whether step-through mode is active.
    step_mode: bool,
    /// Current step position: 0 = |0…0⟩, k = first k unique columns applied.
    step_position: usize,
    /// Sorted unique gate column indices for the current circuit.
    step_columns: Vec<usize>,
    /// Simulator state at the current step position.
    step_simulator: Option<StateVectorSimulator>,
    /// Whether auto-play is advancing through steps.
    step_playing: bool,
    /// Timestamp of the last auto-play advance.
    step_last_advance: Option<Instant>,
    /// Cached entanglement info for the current simulation state.
    entanglement_info: Option<EntanglementInfo>,
    /// Cached per-qubit wire colors derived from entanglement clusters.
    entanglement_colors: Vec<Option<egui::Color32>>,
    /// Earliest modified column since last simulation (for incremental updates).
    dirty_column: Option<usize>,
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

        // Create simulator and apply the circuit (sync for initial small demo)
        let mut sim = StateVectorSimulator::new(circuit.num_qubits());
        sim.apply_circuit(&circuit);

        let (sim_result_tx, sim_result_rx) = mpsc::channel();

        let ent_info = compute_entanglement_info(sim.state());
        let ent_colors = entanglement_wire_colors(&ent_info);

        Self {
            simulator: Some(sim),
            circuit,
            gate_palette: GatePalette::default(),
            circuit_view: CircuitView::default(),
            state_view: StateView::default(),
            stats_panel: StatsPanel::default(),
            example_library: ExampleLibrary::default(),
            editor_state: EditorState::default(),
            history: EditHistory::default(),
            current_file_path: None,
            file_status: None,
            sim_dirty_since: None,
            sim_result_tx,
            sim_result_rx,
            sim_running: false,
            sim_stale: false,
            step_mode: false,
            step_position: 0,
            step_columns: Vec::new(),
            step_simulator: None,
            step_playing: false,
            step_last_advance: None,
            entanglement_info: Some(ent_info),
            entanglement_colors: ent_colors,
            dirty_column: None,
        }
    }

    /// Show file open dialog and load the selected circuit.
    fn open_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KetGrid Circuit", &["ket.json"])
            .add_filter("All Files", &["*"])
            .set_directory(std::env::current_dir().unwrap_or_default())
            .pick_file()
        {
            match Circuit::from_json_file(&path) {
                Ok(loaded_circuit) => {
                    let old = std::mem::replace(&mut self.circuit, loaded_circuit);
                    self.history
                        .push(EditOperation::ReplaceCircuit { old_circuit: old });
                    self.current_file_path = Some(path.clone());
                    self.editor_state.cancel_pending();
                    self.gate_palette.clear_selection();
                    self.editor_state.clear_selection();
                    self.mark_sim_dirty();
                    self.file_status =
                        Some(format!("Loaded: {}", path.file_stem().unwrap_or_default().to_string_lossy()));
                }
                Err(e) => {
                    self.file_status = Some(format!("Error loading: {}", e));
                }
            }
        }
    }

    /// Save the current circuit to file.
    fn save_file_dialog(&mut self) {
        if let Some(ref path) = self.current_file_path {
            // Save to existing file
            if let Err(e) = self.circuit.to_json_file(path) {
                self.file_status = Some(format!("Error saving: {}", e));
            } else {
                self.file_status =
                    Some(format!("Saved: {}", path.file_stem().unwrap_or_default().to_string_lossy()));
            }
        } else {
            // No existing file, show save dialog
            self.save_as_file_dialog();
        }
    }

    /// Show save as dialog and save the circuit.
    fn save_as_file_dialog(&mut self) {
        if let Some(path) = rfd::FileDialog::new()
            .add_filter("KetGrid Circuit", &["ket.json"])
            .add_filter("All Files", &["*"])
            .set_directory(std::env::current_dir().unwrap_or_default())
            .set_file_name("circuit.ket.json")
            .save_file()
        {
            // Ensure the file has the correct extension
            let path_with_ext = if path.extension().is_none() {
                path.with_extension("ket.json")
            } else {
                path
            };

            match self.circuit.to_json_file(&path_with_ext) {
                Ok(()) => {
                    self.current_file_path = Some(path_with_ext.clone());
                    self.file_status = Some(format!(
                        "Saved: {}",
                        path_with_ext.file_stem().unwrap_or_default().to_string_lossy()
                    ));
                }
                Err(e) => {
                    self.file_status = Some(format!("Error saving: {}", e));
                }
            }
        }
    }

    /// Show export dialog and save circuit as Qiskit Python code.
    fn export_qiskit_dialog(&mut self) {
        use ketgrid_core::format::qiskit::circuit_to_qiskit;

        // Generate the Python code
        match circuit_to_qiskit(&self.circuit) {
            Ok(python_code) => {
                // Show save dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Python Files", &["py"])
                    .add_filter("All Files", &["*"])
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .set_file_name("circuit.py")
                    .save_file()
                {
                    // Ensure the file has .py extension
                    let path_with_ext = if path.extension().is_none() {
                        path.with_extension("py")
                    } else {
                        path
                    };

                    match std::fs::write(&path_with_ext, python_code) {
                        Ok(()) => {
                            self.file_status = Some(format!(
                                "Exported to Qiskit: {}",
                                path_with_ext.file_stem().unwrap_or_default().to_string_lossy()
                            ));
                        }
                        Err(e) => {
                            self.file_status = Some(format!("Error exporting: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                self.file_status = Some(format!("Export failed: {}", e));
            }
        }
    }

    /// Show import dialog and load an OpenQASM 2.0 file as a circuit.
    fn import_qasm_dialog(&mut self) {
        use ketgrid_core::format::qasm::circuit_from_qasm;

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("OpenQASM Files", &["qasm"])
            .add_filter("All Files", &["*"])
            .set_directory(std::env::current_dir().unwrap_or_default())
            .pick_file()
        {
            match std::fs::read_to_string(&path) {
                Ok(contents) => match circuit_from_qasm(&contents) {
                    Ok(result) => {
                        let old = std::mem::replace(&mut self.circuit, result.circuit);
                        self.history
                            .push(EditOperation::ReplaceCircuit { old_circuit: old });
                        self.current_file_path = None;
                        self.editor_state.cancel_pending();
                        self.gate_palette.clear_selection();
                        self.editor_state.clear_selection();
                        self.mark_sim_dirty();

                        if result.warnings.is_empty() {
                            self.file_status = Some(format!(
                                "Imported: {}",
                                path.file_stem().unwrap_or_default().to_string_lossy()
                            ));
                        } else {
                            self.file_status = Some(format!(
                                "Imported with {} warning(s): {}",
                                result.warnings.len(),
                                path.file_stem().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                    Err(e) => {
                        self.file_status = Some(format!("Import failed: {}", e));
                    }
                },
                Err(e) => {
                    self.file_status = Some(format!("Error reading file: {}", e));
                }
            }
        }
    }

    /// Show export dialog and save circuit as OpenQASM 2.0 code.
    fn export_qasm_dialog(&mut self) {
        use ketgrid_core::format::qasm::circuit_to_qasm;

        // Generate the OpenQASM code
        match circuit_to_qasm(&self.circuit) {
            Ok(qasm_code) => {
                // Show save dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("OpenQASM Files", &["qasm"])
                    .add_filter("All Files", &["*"])
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .set_file_name("circuit.qasm")
                    .save_file()
                {
                    // Ensure the file has .qasm extension
                    let path_with_ext = if path.extension().is_none() {
                        path.with_extension("qasm")
                    } else {
                        path
                    };

                    match std::fs::write(&path_with_ext, qasm_code) {
                        Ok(()) => {
                            self.file_status = Some(format!(
                                "Exported to OpenQASM: {}",
                                path_with_ext.file_stem().unwrap_or_default().to_string_lossy()
                            ));
                        }
                        Err(e) => {
                            self.file_status = Some(format!("Error exporting: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                self.file_status = Some(format!("Export failed: {}", e));
            }
        }
    }

    /// Show export dialog and save circuit as SVG vector graphic.
    fn export_svg_dialog(&mut self) {
        use ketgrid_core::format::svg::circuit_to_svg;

        // Generate the SVG
        match circuit_to_svg(&self.circuit) {
            Ok(svg_code) => {
                // Show save dialog
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("SVG Files", &["svg"])
                    .add_filter("All Files", &["*"])
                    .set_directory(std::env::current_dir().unwrap_or_default())
                    .set_file_name("circuit.svg")
                    .save_file()
                {
                    // Ensure the file has .svg extension
                    let path_with_ext = if path.extension().is_none() {
                        path.with_extension("svg")
                    } else {
                        path
                    };

                    match std::fs::write(&path_with_ext, svg_code) {
                        Ok(()) => {
                            self.file_status = Some(format!(
                                "Exported to SVG: {}",
                                path_with_ext.file_stem().unwrap_or_default().to_string_lossy()
                            ));
                        }
                        Err(e) => {
                            self.file_status = Some(format!("Error exporting: {}", e));
                        }
                    }
                }
            }
            Err(e) => {
                self.file_status = Some(format!("Export failed: {}", e));
            }
        }
    }

    /// Mark the simulation as needing a refresh after the debounce period.
    ///
    /// For circuits with ≤ [`AUTO_SIM_MAX_QUBITS`] qubits, auto-simulation is
    /// scheduled after [`DEBOUNCE_DURATION`]. Larger circuits require a manual
    /// trigger via [`force_simulate`].
    fn mark_sim_dirty(&mut self) {
        self.mark_sim_dirty_at(None);
    }

    /// Mark the simulation as dirty, optionally recording the modified column
    /// for incremental re-simulation.
    fn mark_sim_dirty_at(&mut self, column: Option<usize>) {
        self.sim_stale = true;
        if let Some(col) = column {
            self.dirty_column = Some(match self.dirty_column {
                Some(existing) => existing.min(col),
                None => col,
            });
        } else {
            // Full re-simulation needed (circuit structure changed).
            self.dirty_column = None;
            self.simulator = None;
        }
        if let Some(ref sim) = self.simulator {
            if sim.num_qubits() != self.circuit.num_qubits() {
                self.simulator = None;
                self.dirty_column = None;
            }
        }
        if self.circuit.num_qubits() <= AUTO_SIM_MAX_QUBITS {
            self.sim_dirty_since = Some(Instant::now());
        }
        if self.step_mode {
            self.step_columns = Self::compute_step_columns(&self.circuit);
            self.step_position = self.step_position.min(self.step_columns.len());
            self.step_playing = false;
            self.rebuild_step_simulator();
        }
    }

    /// Spawn a background thread to run the simulation on a clone of the circuit.
    ///
    /// Uses incremental simulation when a previous simulator with column
    /// checkpoints is available and only a subset of columns changed.
    /// Falls back to optimized full simulation (with gate fusion) otherwise.
    fn start_background_sim(&mut self) {
        let circuit = self.circuit.clone();
        let tx = self.sim_result_tx.clone();
        let existing_sim = self.simulator.take();
        let dirty_col = self.dirty_column.take();
        self.sim_running = true;
        std::thread::spawn(move || {
            let sim = match (existing_sim, dirty_col) {
                (Some(mut sim), Some(col)) if sim.num_qubits() == circuit.num_qubits() => {
                    sim.apply_circuit_from_column(&circuit, col);
                    sim
                }
                _ => {
                    let mut sim = StateVectorSimulator::new(circuit.num_qubits());
                    sim.apply_circuit_optimized(&circuit);
                    sim
                }
            };
            let _ = tx.send(sim);
        });
    }

    /// Recompute cached entanglement info from the given simulator state.
    fn update_entanglement(&mut self, state: &ketgrid_sim::state_vector::StateVector) {
        let info = compute_entanglement_info(state);
        self.entanglement_colors = entanglement_wire_colors(&info);
        self.entanglement_info = Some(info);
    }

    /// Poll for completed background simulations and fire the debounce timer.
    fn poll_simulation_results(&mut self, ctx: &egui::Context) {
        if let Ok(sim) = self.sim_result_rx.try_recv() {
            self.update_entanglement(sim.state());
            self.simulator = Some(sim);
            self.sim_running = false;
            if self.sim_dirty_since.is_none() {
                self.sim_stale = false;
            }
        }

        if let Some(dirty_since) = self.sim_dirty_since {
            let elapsed = dirty_since.elapsed();
            if elapsed >= DEBOUNCE_DURATION && !self.sim_running {
                self.sim_dirty_since = None;
                self.start_background_sim();
            } else if elapsed < DEBOUNCE_DURATION {
                ctx.request_repaint_after(DEBOUNCE_DURATION - elapsed);
            }
        }

        if self.sim_running {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
    }

    /// Immediately start a background simulation (for manual trigger).
    fn force_simulate(&mut self) {
        if !self.sim_running && self.circuit.num_qubits() > 0 {
            self.sim_dirty_since = None;
            self.start_background_sim();
        }
    }

    /// Compute sorted unique gate column indices from the circuit.
    fn compute_step_columns(circuit: &Circuit) -> Vec<usize> {
        let mut cols: Vec<usize> = circuit.gates.iter().map(|g| g.column).collect();
        cols.sort_unstable();
        cols.dedup();
        cols
    }

    /// Enter step-through mode: reset to |0…0⟩ at position 0.
    fn enter_step_mode(&mut self) {
        self.step_mode = true;
        self.step_playing = false;
        self.step_last_advance = None;
        self.step_columns = Self::compute_step_columns(&self.circuit);
        self.step_position = 0;
        self.rebuild_step_simulator();
    }

    /// Exit step-through mode and return to normal simulation.
    fn exit_step_mode(&mut self) {
        self.step_mode = false;
        self.step_playing = false;
        self.step_last_advance = None;
        self.step_simulator = None;
        self.step_columns.clear();
        self.step_position = 0;
    }

    /// Advance one step forward. Returns true if a step was taken.
    fn step_forward(&mut self) -> bool {
        if self.step_position >= self.step_columns.len() {
            return false;
        }
        let col = self.step_columns[self.step_position];
        self.step_position += 1;

        let has_sim = self.step_simulator.is_some();
        if has_sim {
            let mut sim = self.step_simulator.take().unwrap();
            sim.apply_column(&self.circuit, col);
            self.update_entanglement(sim.state());
            self.step_simulator = Some(sim);
        } else {
            self.rebuild_step_simulator();
        }
        true
    }

    /// Go back one step (re-simulates from |0…0⟩).
    fn step_back(&mut self) {
        if self.step_position == 0 {
            return;
        }
        self.step_position -= 1;
        self.rebuild_step_simulator();
    }

    /// Reset to step 0 (initial |0…0⟩ state).
    fn step_reset(&mut self) {
        self.step_position = 0;
        self.step_playing = false;
        self.rebuild_step_simulator();
    }

    /// Rebuild the step simulator from scratch for the current step_position.
    fn rebuild_step_simulator(&mut self) {
        let num_qubits = self.circuit.num_qubits();
        if num_qubits == 0 {
            self.step_simulator = None;
            return;
        }
        let mut sim = StateVectorSimulator::new(num_qubits);
        if self.step_position > 0 {
            let max_col = self.step_columns[self.step_position - 1];
            sim.apply_columns_up_to(&self.circuit, max_col);
        }
        self.update_entanglement(sim.state());
        self.step_simulator = Some(sim);
    }

    /// Compute the cursor column for the circuit view.
    /// Returns the column index at which to draw the step cursor line.
    /// Gates at columns < this value are "applied"; gates >= are dimmed.
    fn step_cursor_col(&self) -> Option<usize> {
        if !self.step_mode {
            return None;
        }
        if self.step_position == 0 {
            Some(0)
        } else if self.step_position >= self.step_columns.len() {
            Some(self.circuit.max_column() + 1)
        } else {
            Some(self.step_columns[self.step_position])
        }
    }

    /// Auto-advance playback if playing.
    fn poll_step_playback(&mut self, ctx: &egui::Context) {
        if !self.step_playing {
            return;
        }

        const STEP_DELAY: Duration = Duration::from_millis(500);

        if let Some(last) = self.step_last_advance {
            if last.elapsed() >= STEP_DELAY {
                if !self.step_forward() {
                    self.step_playing = false;
                } else {
                    self.step_last_advance = Some(Instant::now());
                }
            }
        } else {
            self.step_last_advance = Some(Instant::now());
        }

        if self.step_playing {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
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
            if self
                .circuit
                .add_gate(gate.gate.clone(), target_qubits, control_qubits, target_column)
                .is_ok()
            {
                let idx = self.circuit.gates.len() - 1;
                self.history.push(EditOperation::AddGate {
                    index: idx,
                    gate: self.circuit.gates[idx].clone(),
                });
                self.mark_sim_dirty();
            }
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
                        let old_gate_type = gate.gate.clone();
                        let new_gate_type = match gate.gate {
                            GateType::Rx(_) => GateType::Rx(theta),
                            GateType::Ry(_) => GateType::Ry(theta),
                            GateType::Rz(_) => GateType::Rz(theta),
                            _ => gate.gate.clone(),
                        };
                        if self
                            .circuit
                            .update_gate_parameters(gate_idx, new_gate_type.clone())
                            .is_ok()
                        {
                            self.history.push(EditOperation::EditParam {
                                gate_index: gate_idx,
                                old_gate_type,
                                new_gate_type,
                            });
                            self.mark_sim_dirty();
                        }
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
        if let Some(removed) = self.circuit.remove_gate(gate_idx) {
            self.history.push(EditOperation::RemoveGate {
                index: gate_idx,
                gate: removed,
            });
            self.editor_state.clear_selection();
            self.mark_sim_dirty();
        }
    }

    /// Handle deleting a measurement.
    fn handle_delete_measurement(&mut self, measurement_idx: usize) {
        if let Some(removed) = self.circuit.remove_measurement(measurement_idx) {
            self.history.push(EditOperation::RemoveMeasurement {
                index: measurement_idx,
                measurement: removed,
            });
            self.mark_sim_dirty();
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

    /// Handle copy operation (copies the first selected gate).
    fn handle_copy(&mut self) {
        // Find the first selected gate
        if let Some(gate_id) = self.editor_state.selected_gates.iter().next() {
            if let Some(gate) = self.circuit.gates.get(gate_id.index) {
                self.editor_state.copy_gate(gate);
            }
        }
    }

    /// Handle delete key - removes all selected gates and measurements.
    fn handle_delete_selected(&mut self) {
        // Collect selected gate indices (sort in reverse order to remove from end first)
        let mut gate_indices: Vec<usize> = self
            .editor_state
            .selected_gates
            .iter()
            .map(|g| g.index)
            .collect();
        gate_indices.sort_unstable_by(|a, b| b.cmp(a)); // Reverse order

        // Remove gates (from highest index to lowest to avoid index shifting issues)
        for idx in gate_indices {
            if let Some(removed) = self.circuit.remove_gate(idx) {
                self.history.push(EditOperation::RemoveGate {
                    index: idx,
                    gate: removed,
                });
            }
        }

        // Clear selection after deletion
        if !self.editor_state.selected_gates.is_empty() {
            self.editor_state.clear_selection();
            self.mark_sim_dirty();
        }
    }

    /// Handle add qubit (+ key).
    fn handle_add_qubit(&mut self) {
        self.circuit.add_qubit();
        self.history.push(EditOperation::AddQubit);
        self.mark_sim_dirty();
    }

    /// Handle remove qubit (- key) - removes the last qubit if it has no gates.
    fn handle_remove_qubit(&mut self) {
        let num_qubits = self.circuit.num_qubits();
        if num_qubits == 0 {
            return;
        }

        // Try to remove the last qubit
        let last_qubit_id = num_qubits - 1;

        // Check if we can get the wire info before removing (for undo)
        if let Some(wire) = self.circuit.qubits.get(last_qubit_id).cloned() {
            match self.circuit.remove_qubit(last_qubit_id) {
                Ok(()) => {
                    self.history.push(EditOperation::RemoveQubit {
                        qubit_id: last_qubit_id,
                        wire,
                    });
                    self.editor_state.clear_selection();
                    self.mark_sim_dirty();
                }
                Err(_) => {
                    // Qubit in use, can't remove - could add visual feedback here
                }
            }
        }
    }
}

impl eframe::App for KetGridApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle example library
        if let Some(circuit) = self.example_library.show(ctx) {
            let old = std::mem::replace(&mut self.circuit, circuit);
            self.history
                .push(EditOperation::ReplaceCircuit { old_circuit: old });
            self.current_file_path = None;
            self.editor_state.cancel_pending();
            self.gate_palette.clear_selection();
            self.editor_state.clear_selection();
            self.mark_sim_dirty();
            self.file_status = Some("Example loaded".to_string());
        }

        self.poll_simulation_results(ctx);
        self.poll_step_playback(ctx);

        // Top menu bar
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New Circuit").clicked() {
                        let old = std::mem::replace(&mut self.circuit, Circuit::new(3));
                        self.history
                            .push(EditOperation::ReplaceCircuit { old_circuit: old });
                        self.current_file_path = None;
                        self.editor_state.cancel_pending();
                        self.gate_palette.clear_selection();
                        self.editor_state.clear_selection();
                        self.mark_sim_dirty();
                        self.file_status = Some("New circuit created".to_string());
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        self.open_file_dialog();
                        ui.close_menu();
                    }
                    ui.menu_button("Examples", |ui| {
                        if ui.button("Browse Library…").clicked() {
                            self.example_library.open();
                            ui.close_menu();
                        }
                        ui.separator();
                        if let Some(circuit) = self.example_library.show_compact(ui) {
                            let old = std::mem::replace(&mut self.circuit, circuit);
                            self.history
                                .push(EditOperation::ReplaceCircuit { old_circuit: old });
                            self.current_file_path = None;
                            self.editor_state.cancel_pending();
                            self.gate_palette.clear_selection();
                            self.editor_state.clear_selection();
                            self.mark_sim_dirty();
                            self.file_status = Some("Example loaded".to_string());
                        }
                    });
                    if ui.button("Save").clicked() {
                        self.save_file_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Save As…").clicked() {
                        self.save_as_file_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Export to Qiskit…").clicked() {
                        self.export_qiskit_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Export to OpenQASM…").clicked() {
                        self.export_qasm_dialog();
                        ui.close_menu();
                    }
                    if ui.button("Export to SVG…").clicked() {
                        self.export_svg_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Import from OpenQASM…").clicked() {
                        self.import_qasm_dialog();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.menu_button("Edit", |ui| {
                    let undo_label = if self.history.can_undo() {
                        "Undo  Ctrl+Z"
                    } else {
                        "Undo"
                    };
                    if ui
                        .add_enabled(self.history.can_undo(), egui::Button::new(undo_label))
                        .clicked()
                    {
                        if self.history.undo(&mut self.circuit) {
                            self.editor_state.clear_selection();
                            self.mark_sim_dirty();
                        }
                        ui.close_menu();
                    }
                    let redo_label = if self.history.can_redo() {
                        "Redo  Ctrl+Y"
                    } else {
                        "Redo"
                    };
                    if ui
                        .add_enabled(self.history.can_redo(), egui::Button::new(redo_label))
                        .clicked()
                    {
                        if self.history.redo(&mut self.circuit) {
                            self.editor_state.clear_selection();
                            self.mark_sim_dirty();
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Add Qubit").clicked() {
                        self.circuit.add_qubit();
                        self.history.push(EditOperation::AddQubit);
                        self.mark_sim_dirty();
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
        let sim_stale = self.sim_stale;
        let sim_running = self.sim_running;
        let circuit_qubits = self.circuit.num_qubits();
        let in_step_mode = self.step_mode;
        let step_pos = self.step_position;
        let step_total = self.step_columns.len();
        let mut should_force_sim = false;

        egui::SidePanel::right("state_view")
            .resizable(true)
            .default_width(280.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("State");
                    if in_step_mode {
                        ui.colored_label(
                            egui::Color32::from_rgb(0, 200, 255),
                            format!("Step {}/{}", step_pos, step_total),
                        );
                    } else if circuit_qubits > AUTO_SIM_MAX_QUBITS && !sim_running {
                        if ui.button("▶ Simulate").clicked() {
                            should_force_sim = true;
                        }
                    }
                });

                if !in_step_mode && sim_stale {
                    if sim_running {
                        ui.horizontal(|ui| {
                            ui.spinner();
                            ui.colored_label(
                                egui::Color32::from_rgb(200, 200, 100),
                                "Simulating…",
                            );
                        });
                    } else if circuit_qubits > AUTO_SIM_MAX_QUBITS {
                        ui.colored_label(
                            egui::Color32::from_rgb(200, 200, 100),
                            "Results outdated — click Simulate",
                        );
                    }
                }

                ui.separator();

                // Circuit statistics panel
                self.stats_panel.show(ui, &self.circuit);

                ui.add_space(8.0);
                ui.separator();

                let active_sim = if in_step_mode {
                    self.step_simulator.as_ref()
                } else {
                    self.simulator.as_ref()
                };
                if let Some(sim) = active_sim {
                    let ent_data = self.entanglement_info.as_ref()
                        .map(|info| (info, self.entanglement_colors.as_slice()));
                    self.state_view.show(ui, sim.state(), ent_data);
                }
            });

        if should_force_sim {
            self.force_simulate();
        }

        // Central panel: Circuit editor with drop zones
        let mut clicked_target: Option<DropTarget> = None;
        let mut gate_outcome = None;
        let mut measurement_outcome = None;
        let mut stepper_action: Option<StepperAction> = None;
        let step_cursor = self.step_cursor_col();

        egui::CentralPanel::default().show(ctx, |ui| {
            // Stepper toolbar
            ui.horizontal(|ui| {
                let in_step = self.step_mode;
                if ui.selectable_label(in_step, "Step Mode").clicked() {
                    stepper_action = Some(if in_step {
                        StepperAction::Exit
                    } else {
                        StepperAction::Enter
                    });
                }

                if in_step {
                    ui.separator();
                    if ui.button("⏮ Reset").clicked() {
                        stepper_action = Some(StepperAction::Reset);
                    }
                    let can_back = self.step_position > 0;
                    if ui.add_enabled(can_back, egui::Button::new("◀ Back")).clicked() {
                        stepper_action = Some(StepperAction::Back);
                    }
                    let can_fwd = self.step_position < self.step_columns.len();
                    if ui.add_enabled(can_fwd, egui::Button::new("▶ Fwd")).clicked() {
                        stepper_action = Some(StepperAction::Forward);
                    }
                    let play_label = if self.step_playing { "⏸ Pause" } else { "⏵ Play" };
                    if ui.add_enabled(can_fwd || self.step_playing, egui::Button::new(play_label)).clicked() {
                        stepper_action = Some(StepperAction::TogglePlay);
                    }
                    ui.separator();
                    ui.label(format!(
                        "Step {}/{}",
                        self.step_position, self.step_columns.len()
                    ));
                }
            });
            ui.separator();

            let (target, outcome, meas_outcome) = self.circuit_view.show(
                ui,
                &self.circuit,
                active_gate.as_ref(),
                is_measurement_mode,
                &self.editor_state,
                step_cursor,
                &self.entanglement_colors,
            );
            clicked_target = target;
            gate_outcome = outcome;
            measurement_outcome = meas_outcome;
        });

        // Process stepper actions outside the closure
        match stepper_action {
            Some(StepperAction::Enter) => self.enter_step_mode(),
            Some(StepperAction::Exit) => self.exit_step_mode(),
            Some(StepperAction::Reset) => self.step_reset(),
            Some(StepperAction::Back) => self.step_back(),
            Some(StepperAction::Forward) => { self.step_forward(); }
            Some(StepperAction::TogglePlay) => {
                self.step_playing = !self.step_playing;
                if self.step_playing {
                    self.step_last_advance = Some(Instant::now());
                }
            }
            None => {}
        }

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
                if self
                    .circuit
                    .add_measurement(target.qubit_idx, target.column)
                    .is_ok()
                {
                    let idx = self.circuit.measurements.len() - 1;
                    self.history.push(EditOperation::AddMeasurement {
                        index: idx,
                        measurement: self.circuit.measurements[idx].clone(),
                    });
                    self.gate_palette.clear_selection();
                    self.mark_sim_dirty();
                }
            }

            // Handle drag-drop for measurement
            if clicked_target.is_none() {
                let released = ctx.input(|i| i.pointer.any_released());
                if released {
                    if let Some(pos) = ctx.input(|i| i.pointer.latest_pos()) {
                        if let Some(target) =
                            self.circuit_view.hit_test(pos, self.circuit.num_qubits())
                        {
                            if self
                                .circuit
                                .add_measurement(target.qubit_idx, target.column)
                                .is_ok()
                            {
                                let idx = self.circuit.measurements.len() - 1;
                                self.history.push(EditOperation::AddMeasurement {
                                    index: idx,
                                    measurement: self.circuit.measurements[idx].clone(),
                                });
                                self.gate_palette.clear_selection();
                                self.mark_sim_dirty();
                            }
                        }
                    }
                }
            }
        } else if let (Some(target), Some(gate)) = (clicked_target, &active_gate) {
            // Handle gate placement from circuit view click
            if let Some(placement) = self.editor_state.try_place(gate, target) {
                if self
                    .circuit
                    .add_gate(
                        placement.gate,
                        placement.target_qubits,
                        placement.control_qubits,
                        placement.column,
                    )
                    .is_ok()
                {
                    let idx = self.circuit.gates.len() - 1;
                    self.history.push(EditOperation::AddGate {
                        index: idx,
                        gate: self.circuit.gates[idx].clone(),
                    });
                    if !self.editor_state.is_awaiting_more_qubits() {
                        self.gate_palette.clear_selection();
                    }
                    self.mark_sim_dirty();
                }
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
                                if self
                                    .circuit
                                    .add_gate(
                                        placement.gate,
                                        placement.target_qubits,
                                        placement.control_qubits,
                                        placement.column,
                                    )
                                    .is_ok()
                                {
                                    let idx = self.circuit.gates.len() - 1;
                                    self.history.push(EditOperation::AddGate {
                                        index: idx,
                                        gate: self.circuit.gates[idx].clone(),
                                    });
                                    if !self.editor_state.is_awaiting_more_qubits() {
                                        self.gate_palette.clear_selection();
                                    }
                                    self.mark_sim_dirty();
                                }
                            }
                        }
                    }
                }
            }
        }

        // Undo on Ctrl+Z (without Shift)
        if ctx.input(|i| {
            i.modifiers.ctrl && i.key_pressed(egui::Key::Z) && !i.modifiers.shift
        }) {
            if self.history.undo(&mut self.circuit) {
                self.editor_state.clear_selection();
                self.editor_state.cancel_pending();
                self.gate_palette.clear_selection();
                self.mark_sim_dirty();
            }
        }

        // Redo on Ctrl+Y or Ctrl+Shift+Z
        if ctx.input(|i| {
            i.modifiers.ctrl
                && (i.key_pressed(egui::Key::Y)
                    || (i.key_pressed(egui::Key::Z) && i.modifiers.shift))
        }) {
            if self.history.redo(&mut self.circuit) {
                self.editor_state.clear_selection();
                self.editor_state.cancel_pending();
                self.gate_palette.clear_selection();
                self.mark_sim_dirty();
            }
        }

        // Handle copy on Ctrl+C
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::C)) {
            self.handle_copy();
        }

        // Handle paste on Ctrl+V
        if ctx.input(|i| i.modifiers.ctrl && i.key_pressed(egui::Key::V)) {
            self.handle_paste();
        }

        // Handle delete key for selected gates/measurements
        if ctx.input(|i| i.key_pressed(egui::Key::Delete)) {
            self.handle_delete_selected();
        }

        // Handle +/- keys for qubit management
        if ctx.input(|i| i.key_pressed(egui::Key::Plus) || i.key_pressed(egui::Key::Equals)) {
            self.handle_add_qubit();
        }
        if ctx.input(|i| i.key_pressed(egui::Key::Minus)) {
            self.handle_remove_qubit();
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

                // Display file status if present
                if let Some(ref file_status) = self.file_status {
                    ui.separator();
                    ui.colored_label(egui::Color32::from_rgb(100, 200, 100), file_status);
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
