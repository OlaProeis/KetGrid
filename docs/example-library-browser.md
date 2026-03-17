# Example Library Browser

A categorized, searchable interface for browsing and loading quantum circuit examples.

## Overview

The example library browser provides visual access to 21+ example quantum circuits organized into three categories:
- **Fundamentals** - Basic gates and quantum operations
- **Algorithms** - Famous quantum algorithms and protocols
- **Error-Correction** - Quantum error correction codes

## User Interface

### Full Browser Window

Access via **File → Examples → Browse Library...**

Features:
- **Search bar** - Filter examples by name or description
- **Category tabs** - View All, Fundamentals, Algorithms, or Error-Correction
- **Example cards** - Each card shows:
  - Category icon (⚛ Fundamentals, ⚡ Algorithms, 🛡 Error-Correction)
  - Example name and qubit/gate count
  - Description of what the example demonstrates
  - "Load Example" button

### Compact Menu View

Access via **File → Examples** submenu for quick loading without opening the full browser.

## Implementation

### Module Location

`crates/ketgrid-gui/src/examples.rs`

### Key Types

```rust
/// Example categories
enum ExampleCategory {
    Fundamentals,
    Algorithms,
    ErrorCorrection,
}

/// Example metadata
struct Example {
    file_name: String,      // e.g., "bell"
    name: String,           // e.g., "Bell State"
    description: String,    // What it demonstrates
    qubit_count: usize,
    gate_count: usize,
    category: ExampleCategory,
}

/// Browser state
struct ExampleLibrary {
    selected_category: Option<ExampleCategory>,
    search_query: String,
    examples: Vec<Example>,
    selected_example: Option<usize>,
    is_open: bool,
}
```

### Loading Examples

```rust
// Load from full browser
if let Some(circuit) = example_library.show(ctx) {
    // Replace current circuit with loaded example
}

// Load by name programmatically
let circuit = example_library.load_by_name("bell")?;
```

### Adding New Examples

1. Create `.ket.json` file in `examples/` directory
2. Add metadata entry in `ExampleLibrary::build_example_list()`:

```rust
Example {
    file_name: "my-example".to_string(),
    name: "My Example".to_string(),
    description: "What it demonstrates".to_string(),
    qubit_count: 3,
    gate_count: 5,
    category: ExampleCategory::Algorithms,
}
```

## Integration

The browser integrates with:
- **App state** (`KetGridApp::example_library`) - Persistent browser state
- **Menu system** - File → Examples submenu and browse option
- **History system** - Loading an example pushes a ReplaceCircuit operation
- **Simulation** - Auto-triggers simulation refresh after loading

## Available Examples

### Fundamentals (9)
- Hadamard Gate, Pauli-X/Y/Z Gates, Phase Gate (S), T Gate
- Rotation Gates (Rx, Ry, Rz), SWAP Gate, Toffoli (CCNOT)

### Algorithms (9)
- Bell State, GHZ State, Quantum Teleportation, Superdense Coding
- Deutsch-Jozsa, Bernstein-Vazirani, Grover's Algorithm
- Simon's Algorithm, Quantum Fourier Transform

### Error-Correction (3)
- 3-Qubit Bit-Flip Code, 3-Qubit Phase-Flip Code
- Shor's 9-Qubit Code
