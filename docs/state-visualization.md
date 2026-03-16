# State Visualization

Probability histogram and amplitude table for quantum state vectors.

## Probability Histogram

Located in `crates/ketgrid-gui/src/state_view.rs`.

### Features

- **Phase-aware coloring**: Bar colors represent the quantum phase of each basis state
  - Red (0° hue) = 0 phase angle
  - Blue (240° hue) = π (180°) phase angle
  - Intermediate phases interpolate through purple
  - Gray = zero probability (no phase defined)

- **Measurement probabilities**: Each bar shows |amplitude|² as a percentage

- **Basis state labels**: Formatted as |00⟩, |01⟩, etc. with proper qubit count

### Implementation

```rust
// Phase to color mapping uses HSV color space
fn phase_to_color(phase: f64) -> Color32 {
    // Maps [-π, π] to hue gradient red → blue → red
}
```

## Amplitude Table

Toggle-enabled table showing complex amplitudes:

| Column | Description |
|--------|-------------|
| State | Basis state label |
| Real | Real part of amplitude |
| Imaginary | Imaginary part of amplitude |
| Phase | Angle in degrees (-180° to +180°) |

Zero-probability states are filtered out for clarity.

## Usage

The `StateView` struct is used in the main app to display simulation results:

```rust
pub struct StateView {
    show_amplitude_table: bool,
}
```

Checkbox toggles the amplitude table visibility below the histogram.
