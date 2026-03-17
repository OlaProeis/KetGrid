//! Example library browser for browsing and loading quantum circuit examples.
//!
//! Provides a categorized, searchable interface for 15+ example circuits
//! organized into Fundamentals, Algorithms, and Error-Correction categories.

use ketgrid_core::Circuit;
use std::path::PathBuf;

/// Categories for organizing example circuits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExampleCategory {
    /// Basic quantum gates and states.
    Fundamentals,
    /// Quantum algorithms and protocols.
    Algorithms,
    /// Error correction codes.
    ErrorCorrection,
}

impl ExampleCategory {
    /// Returns the display name for the category.
    pub fn display_name(&self) -> &'static str {
        match self {
            ExampleCategory::Fundamentals => "Fundamentals",
            ExampleCategory::Algorithms => "Algorithms",
            ExampleCategory::ErrorCorrection => "Error-Correction",
        }
    }

    /// Returns all categories in order.
    pub fn all() -> &'static [ExampleCategory] {
        &[
            ExampleCategory::Fundamentals,
            ExampleCategory::Algorithms,
            ExampleCategory::ErrorCorrection,
        ]
    }
}

/// Metadata for a single example circuit.
#[derive(Debug, Clone)]
pub struct Example {
    /// The file name (without extension).
    pub file_name: String,
    /// Display name for the example.
    pub name: String,
    /// Description of what the example demonstrates.
    pub description: String,
    /// Number of qubits in the circuit.
    pub qubit_count: usize,
    /// Gate count in the circuit.
    pub gate_count: usize,
    /// Which category this example belongs to.
    pub category: ExampleCategory,
}

impl Example {
    /// Returns a short preview text for the example.
    pub fn preview_text(&self) -> String {
        format!("{} qubits, {} gates", self.qubit_count, self.gate_count)
    }
}

/// The example library browser state and UI.
pub struct ExampleLibrary {
    /// Currently selected category filter.
    selected_category: Option<ExampleCategory>,
    /// Search query text.
    search_query: String,
    /// List of all available examples.
    examples: Vec<Example>,
    /// Currently selected example index (if any).
    selected_example: Option<usize>,
    /// Whether the browser window is open.
    is_open: bool,
    /// Path to the examples directory.
    examples_dir: PathBuf,
}

impl Default for ExampleLibrary {
    fn default() -> Self {
        Self {
            selected_category: None,
            search_query: String::new(),
            examples: Self::build_example_list(),
            selected_example: None,
            is_open: false,
            examples_dir: PathBuf::from("examples"),
        }
    }
}

impl ExampleLibrary {
    /// Creates a new example library with the given examples directory.
    pub fn with_examples_dir(dir: PathBuf) -> Self {
        Self {
            examples_dir: dir,
            ..Default::default()
        }
    }

    /// Builds the complete list of example circuits with metadata.
    fn build_example_list() -> Vec<Example> {
        vec![
            // Fundamentals
            Example {
                file_name: "hadamard".to_string(),
                name: "Hadamard Gate".to_string(),
                description: "Creates equal superposition of |0⟩ and |1⟩. Fundamental for quantum parallelism.".to_string(),
                qubit_count: 1,
                gate_count: 1,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "pauli-x".to_string(),
                name: "Pauli-X Gate".to_string(),
                description: "Quantum NOT gate. Flips |0⟩ to |1⟩ and vice versa.".to_string(),
                qubit_count: 1,
                gate_count: 1,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "pauli-y".to_string(),
                name: "Pauli-Y Gate".to_string(),
                description: "Rotates Bloch vector by π around Y-axis. Combines bit and phase flips.".to_string(),
                qubit_count: 1,
                gate_count: 1,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "pauli-z".to_string(),
                name: "Pauli-Z Gate".to_string(),
                description: "Phase flip gate. Leaves |0⟩ unchanged, flips sign of |1⟩.".to_string(),
                qubit_count: 1,
                gate_count: 3,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "phase-gate".to_string(),
                name: "Phase Gate (S)".to_string(),
                description: "Applies π/2 phase shift to |1⟩. Universal gate set component.".to_string(),
                qubit_count: 1,
                gate_count: 2,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "t-gate".to_string(),
                name: "T Gate (π/8)".to_string(),
                description: "Applies π/4 phase shift. Essential for quantum error correction.".to_string(),
                qubit_count: 1,
                gate_count: 2,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "rotation-gates".to_string(),
                name: "Rotation Gates".to_string(),
                description: "Demonstrates Rx, Ry, Rz parameterized rotation gates.".to_string(),
                qubit_count: 1,
                gate_count: 3,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "swap".to_string(),
                name: "SWAP Gate".to_string(),
                description: "Exchanges quantum states between two qubits.".to_string(),
                qubit_count: 2,
                gate_count: 2,
                category: ExampleCategory::Fundamentals,
            },
            Example {
                file_name: "toffoli".to_string(),
                name: "Toffoli (CCNOT)".to_string(),
                description: "Universal reversible logic gate with two controls.".to_string(),
                qubit_count: 3,
                gate_count: 3,
                category: ExampleCategory::Fundamentals,
            },
            // Algorithms
            Example {
                file_name: "bell".to_string(),
                name: "Bell State".to_string(),
                description: "Creates maximally entangled Bell pair |Φ+⟩.".to_string(),
                qubit_count: 2,
                gate_count: 2,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "ghz".to_string(),
                name: "GHZ State".to_string(),
                description: "3-qubit Greenberger-Horne-Zeilinger entangled state.".to_string(),
                qubit_count: 3,
                gate_count: 3,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "teleportation".to_string(),
                name: "Quantum Teleportation".to_string(),
                description: "Transfers quantum state using entanglement and classical bits.".to_string(),
                qubit_count: 3,
                gate_count: 5,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "superdense-coding".to_string(),
                name: "Superdense Coding".to_string(),
                description: "Transmits 2 classical bits using 1 qubit and entanglement.".to_string(),
                qubit_count: 2,
                gate_count: 8,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "deutsch-jozsa".to_string(),
                name: "Deutsch-Jozsa Algorithm".to_string(),
                description: "Determines if function is constant or balanced in 1 query.".to_string(),
                qubit_count: 2,
                gate_count: 5,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "bernstein-vazirani".to_string(),
                name: "Bernstein-Vazirani".to_string(),
                description: "Recovers hidden string in a single quantum query.".to_string(),
                qubit_count: 4,
                gate_count: 10,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "grover-2qubit".to_string(),
                name: "Grover's Algorithm".to_string(),
                description: "Searches unsorted database with quadratic speedup.".to_string(),
                qubit_count: 2,
                gate_count: 11,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "simon-algorithm".to_string(),
                name: "Simon's Algorithm".to_string(),
                description: "Finds hidden string exponentially faster than classical.".to_string(),
                qubit_count: 4,
                gate_count: 6,
                category: ExampleCategory::Algorithms,
            },
            Example {
                file_name: "qft-3qubit".to_string(),
                name: "Quantum Fourier Transform".to_string(),
                description: "Transforms to Fourier basis. Core of many quantum algorithms.".to_string(),
                qubit_count: 3,
                gate_count: 6,
                category: ExampleCategory::Algorithms,
            },
            // Error-Correction
            Example {
                file_name: "bit-flip-code".to_string(),
                name: "3-Qubit Bit-Flip Code".to_string(),
                description: "Protects against X (bit-flip) errors using 3 qubits.".to_string(),
                qubit_count: 3,
                gate_count: 11,
                category: ExampleCategory::ErrorCorrection,
            },
            Example {
                file_name: "phase-flip-code".to_string(),
                name: "3-Qubit Phase-Flip Code".to_string(),
                description: "Protects against Z (phase-flip) errors in Hadamard basis.".to_string(),
                qubit_count: 3,
                gate_count: 14,
                category: ExampleCategory::ErrorCorrection,
            },
            Example {
                file_name: "shor-code".to_string(),
                name: "Shor's 9-Qubit Code".to_string(),
                description: "First code protecting against arbitrary single-qubit errors.".to_string(),
                qubit_count: 9,
                gate_count: 13,
                category: ExampleCategory::ErrorCorrection,
            },
        ]
    }

    /// Opens the example library browser.
    pub fn open(&mut self) {
        self.is_open = true;
    }

    /// Closes the example library browser.
    pub fn close(&mut self) {
        self.is_open = false;
    }

    /// Returns whether the browser is currently open.
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Toggles the browser open/closed state.
    pub fn toggle(&mut self) {
        self.is_open = !self.is_open;
    }

    /// Returns the full path to an example file.
    fn example_path(&self, file_name: &str) -> PathBuf {
        self.examples_dir.join(format!("{}.ket.json", file_name))
    }

    /// Loads the selected example circuit from file.
    ///
    /// # Returns
    /// * `Ok(Circuit)` - The loaded circuit.
    /// * `Err(String)` - Error message if loading failed.
    pub fn load_selected(&self) -> Result<Circuit, String> {
        if let Some(idx) = self.selected_example {
            let example = &self.examples[idx];
            let path = self.example_path(&example.file_name);
            Circuit::from_json_file(&path)
                .map_err(|e| format!("Failed to load '{}': {}", example.name, e))
        } else {
            Err("No example selected".to_string())
        }
    }

    /// Loads an example by its file name.
    ///
    /// # Returns
    /// * `Ok(Circuit)` - The loaded circuit.
    /// * `Err(String)` - Error message if loading failed.
    pub fn load_by_name(&self, file_name: &str) -> Result<Circuit, String> {
        let path = self.example_path(file_name);
        Circuit::from_json_file(&path)
            .map_err(|e| format!("Failed to load '{}': {}", file_name, e))
    }

    /// Gets the filtered list of examples based on category and search.
    fn filtered_examples(&self) -> Vec<(usize, &Example)> {
        self.examples
            .iter()
            .enumerate()
            .filter(|(_, ex)| {
                // Category filter
                if let Some(cat) = self.selected_category {
                    if ex.category != cat {
                        return false;
                    }
                }
                // Search filter
                if !self.search_query.is_empty() {
                    let query = self.search_query.to_lowercase();
                    if !ex.name.to_lowercase().contains(&query)
                        && !ex.description.to_lowercase().contains(&query)
                    {
                        return false;
                    }
                }
                true
            })
            .collect()
    }

    /// Renders the example library browser UI.
    ///
    /// # Arguments
    /// * `ctx` - The egui context.
    ///
    /// # Returns
    /// * `Some(Circuit)` - If an example was selected and loaded.
    /// * `None` - If no example was loaded.
    pub fn show(&mut self, ctx: &egui::Context) -> Option<Circuit> {
        if !self.is_open {
            return None;
        }

        let mut should_load: Option<usize> = None;
        let mut should_close = false;

        let example_count = self.examples.len();
        let filtered_count = self.filtered_examples().len();

        egui::Window::new("Example Library")
            .default_size([600.0, 500.0])
            .min_size([500.0, 400.0])
            .collapsible(false)
            .resizable(true)
            .show(ctx, |ui| {
                ui.vertical(|ui| {
                    // Search bar
                    ui.horizontal(|ui| {
                        ui.label("🔍");
                        ui.text_edit_singleline(&mut self.search_query);
                        if ui.button("Clear").clicked() {
                            self.search_query.clear();
                        }
                    });
                    ui.separator();

                    // Category tabs
                    ui.horizontal(|ui| {
                        let all_selected = self.selected_category.is_none();
                        if ui
                            .selectable_label(all_selected, "All")
                            .clicked()
                        {
                            self.selected_category = None;
                        }
                        for cat in ExampleCategory::all() {
                            let selected = self.selected_category == Some(*cat);
                            if ui.selectable_label(selected, cat.display_name()).clicked() {
                                self.selected_category = Some(*cat);
                            }
                        }
                    });
                    ui.separator();

                    // Example list
                    let filtered = self.filtered_examples();
                    egui::ScrollArea::vertical()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            ui.vertical(|ui| {
                                if filtered.is_empty() {
                                    ui.label("No examples match your search.");
                                } else {
                                    for (idx, example) in filtered {
                                        if Self::render_example_card(ui, self.selected_example == Some(idx), example) {
                                            should_load = Some(idx);
                                        }
                                    }
                                }
                            });
                        });

                    ui.separator();

                    // Status and count
                    ui.horizontal(|ui| {
                        ui.label(format!("{}/{} examples", filtered_count, example_count));
                        ui.with_layout(
                            egui::Layout::right_to_left(egui::Align::Center),
                            |ui| {
                                if ui.button("Close").clicked() {
                                    should_close = true;
                                }
                            },
                        );
                    });
                });
            });

        // Handle actions outside the closure
        if should_close {
            self.close();
        }

        if let Some(idx) = should_load {
            self.selected_example = Some(idx);
            if let Ok(circuit) = self.load_selected() {
                self.close();
                return Some(circuit);
            }
        }

        None
    }

    /// Renders a single example card. Returns true if load was clicked.
    fn render_example_card(ui: &mut egui::Ui, is_selected: bool, example: &Example) -> bool {
        let card_color = if is_selected {
            ui.visuals().selection.bg_fill
        } else {
            ui.visuals().panel_fill
        };

        let mut load_clicked = false;

        egui::Frame::group(ui.style())
            .fill(card_color)
            .show(ui, |ui| {
                ui.set_min_width(ui.available_width());

                ui.horizontal(|ui| {
                    // Category icon
                    let icon = match example.category {
                        ExampleCategory::Fundamentals => "⚛",
                        ExampleCategory::Algorithms => "⚡",
                        ExampleCategory::ErrorCorrection => "🛡",
                    };
                    ui.label(egui::RichText::new(icon).size(24.0));

                    ui.vertical(|ui| {
                        // Title
                        ui.horizontal(|ui| {
                            ui.label(
                                egui::RichText::new(&example.name)
                                    .strong()
                                    .size(16.0),
                            );
                            ui.label(
                                egui::RichText::new(example.preview_text())
                                    .weak()
                                    .size(12.0),
                            );
                        });

                        // Description
                        ui.label(
                            egui::RichText::new(&example.description)
                                .size(12.0),
                        );

                        // Load button
                        ui.horizontal(|ui| {
                            if ui.button("Load Example").clicked() {
                                load_clicked = true;
                            }
                        });
                    });
                });
            });

        ui.add_space(4.0);
        load_clicked
    }

    /// Renders a compact version suitable for a menu dropdown.
    pub fn show_compact(&mut self, ui: &mut egui::Ui) -> Option<Circuit> {
        let mut loaded_circuit: Option<Circuit> = None;

        // Category headers with nested examples
        for cat in ExampleCategory::all() {
            ui.collapsing(cat.display_name(), |ui| {
                for (idx, example) in self.examples.iter().enumerate() {
                    if example.category == *cat {
                        if ui.button(&example.name).clicked() {
                            self.selected_example = Some(idx);
                            if let Ok(circuit) = self.load_selected() {
                                loaded_circuit = Some(circuit);
                            }
                        }
                    }
                }
            });
        }

        loaded_circuit
    }

    /// Returns the number of examples in the library.
    pub fn example_count(&self) -> usize {
        self.examples.len()
    }

    /// Returns the number of examples in a specific category.
    pub fn count_by_category(&self, category: ExampleCategory) -> usize {
        self.examples
            .iter()
            .filter(|ex| ex.category == category)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_library_default() {
        let lib = ExampleLibrary::default();
        assert!(!lib.is_open());
        assert_eq!(lib.example_count(), 21); // 15+ examples as required
    }

    #[test]
    fn test_category_counts() {
        let lib = ExampleLibrary::default();
        assert_eq!(lib.count_by_category(ExampleCategory::Fundamentals), 9);
        assert_eq!(lib.count_by_category(ExampleCategory::Algorithms), 9);
        assert_eq!(lib.count_by_category(ExampleCategory::ErrorCorrection), 3);
    }

    #[test]
    fn test_category_display_names() {
        assert_eq!(ExampleCategory::Fundamentals.display_name(), "Fundamentals");
        assert_eq!(ExampleCategory::Algorithms.display_name(), "Algorithms");
        assert_eq!(ExampleCategory::ErrorCorrection.display_name(), "Error-Correction");
    }

    #[test]
    fn test_example_open_close() {
        let mut lib = ExampleLibrary::default();
        assert!(!lib.is_open());
        lib.open();
        assert!(lib.is_open());
        lib.close();
        assert!(!lib.is_open());
        lib.toggle();
        assert!(lib.is_open());
    }

    #[test]
    fn test_preview_text() {
        let example = Example {
            file_name: "test".to_string(),
            name: "Test".to_string(),
            description: "Test example".to_string(),
            qubit_count: 3,
            gate_count: 5,
            category: ExampleCategory::Fundamentals,
        };
        assert_eq!(example.preview_text(), "3 qubits, 5 gates");
    }
}
