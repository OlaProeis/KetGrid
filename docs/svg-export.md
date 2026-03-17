# SVG Circuit Export

Publication-quality vector export of quantum circuit diagrams for academic papers, presentations, and documentation.

## Overview

The SVG export feature generates scalable vector graphics that faithfully reproduce the circuit visualization shown in the KetGrid editor. The exported SVG uses the same layout constants and rendering logic as the interactive circuit view, ensuring visual consistency between the application and exported figures.

## Usage

### From the GUI

1. Design your quantum circuit in the editor
2. Select **File → Export to SVG...**
3. Choose a filename and location
4. The SVG file is saved and can be opened in browsers, vector editors (Inkscape, Illustrator), or embedded in documents

### Programmatic API

```rust
use ketgrid_core::Circuit;
use ketgrid_core::format::svg::circuit_to_svg;

let circuit = Circuit::new(2);
// ... add gates ...

// Export to SVG string
let svg_xml = circuit_to_svg(&circuit)?;
std::fs::write("circuit.svg", svg_xml)?;
```

Or using the convenience method:

```rust
let svg_xml = circuit.to_svg()?;
```

## Generated SVG Structure

### Layout

The SVG follows the same layout system as the interactive circuit view:

| Constant | Value | Description |
|----------|-------|-------------|
| `WIRE_SPACING` | 60.0 | Vertical distance between qubit wires |
| `COLUMN_WIDTH` | 64.0 | Horizontal spacing between gate columns |
| `LABEL_WIDTH` | 55.0 | Space reserved for qubit labels |
| `GATE_BOX_SIZE` | 36.0 | Size of single-qubit gate boxes |

### Visual Elements

**Qubit Wires**
- Horizontal lines spanning the circuit width
- Qubit labels (e.g., `|q₀⟩`, `|q₁⟩`) on the left

**Single-Qubit Gates**
- Rounded rectangle boxes with gate labels
- H, X, Y, Z, S, T gates shown as single letters
- Rotation gates (Rx, Ry, Rz) show angle in degrees: `Rx(3.14)`

**Multi-Qubit Gates**
- **CNOT/Toffoli**: ● (filled circle) on control qubits, ⊕ (circled plus) on target
- **CZ**: ● on both qubits with vertical connecting line
- **SWAP**: ✕ marks on both qubits with vertical line
- **Barrier**: Dashed vertical line (visual separator)

**Measurements**
- Box with semicircular arc and arrow (meter symbol)

### Styling

The SVG uses embedded CSS with publication-ready styling:

```css
.wire { stroke: #333; stroke-width: 1.5; }
.gate-box { fill: #f8f9fa; stroke: #495057; }
.gate-text { font-family: system-ui, sans-serif; font-size: 13px; }
.qubit-label { font-size: 14px; fill: #212529; }
```

Colors are optimized for:
- **Print**: Dark-on-light theme for laser printers
- **Digital**: Clean appearance on screens and projectors
- **Accessibility**: High contrast black text on light backgrounds

## Technical Details

### Module Location

- **Implementation**: `crates/ketgrid-core/src/format/svg.rs`
- **Public API**: `ketgrid_core::format::svg::circuit_to_svg`
- **Circuit Method**: `Circuit::to_svg()`

### SVG Features

- **ViewBox**: Automatically sized to fit circuit with proper margins
- **Scaling**: `preserveAspectRatio="xMidYMid meet"` for responsive sizing
- **Standalone**: No external dependencies, all styles embedded
- **XML Encoding**: Proper escaping for special characters in labels

### Limitations

- Single-page output (no pagination for very large circuits)
- No animation or interactivity (static vector graphic)
- Entanglement colors from the UI are not included (publication-ready neutral theme)

## Example Output

A Bell state circuit (H on q0, CNOT q0→q1) produces SVG with:
- Two horizontal qubit wires with labels
- H gate box on first qubit
- CNOT with control dot on q0 and target ⊕ on q1
- Clean styling suitable for LaTeX documents or PowerPoint slides

## Testing

The SVG export is tested against:
- All gate types (single-qubit, controlled, multi-qubit)
- Parameterized rotation gates
- Bell state and GHZ state circuits
- XML escaping for special characters
- SVG header structure and namespace correctness
