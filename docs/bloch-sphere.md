# Bloch Sphere Visualization

Per-qubit Bloch sphere display in the state visualization panel, showing each qubit's reduced state on the Bloch sphere.

## Location

`crates/ketgrid-gui/src/bloch.rs` — `BlochSphere` widget and `compute_bloch_vector` function.

## How It Works

### Partial Trace

For an n-qubit state vector, each qubit's reduced density matrix is computed by tracing out all other qubits. The implementation iterates over pairs of basis state indices that differ only at the target qubit's bit position, accumulating the four components of the 2x2 density matrix directly as f64 accumulators (no intermediate matrix allocation).

Big-endian qubit ordering: qubit k corresponds to bit position (n-1-k) in the state vector index, matching `StateVector`'s convention.

### Bloch Vector

From the reduced density matrix rho:
- r_x = 2 Re(rho_01)
- r_y = -2 Im(rho_01)
- r_z = rho_00 - rho_11

Convention: |0> = north pole (+z), |1> = south pole (-z), |+> = +x, |+i> = +y.

The vector magnitude |r| indicates purity: 1.0 for pure single-qubit states, 0.0 for maximally mixed (entangled) qubits.

### Rendering

Orthographic 3D-to-2D projection using two view angles (azimuth and elevation). The sphere shows:
- Circle outline with dark fill
- Equator (XY plane) and meridian (XZ plane) wireframes with front/back depth distinction
- Axis labels: |0>, |1> at poles; |+>, |-> at equator (shown only when facing viewer)
- Bloch vector arrow from center to the state point
- Color intensity reflects purity (bright cyan = pure, dim = mixed)

### Interaction

Drag any sphere to rotate the shared viewing angle. All spheres share the same azimuth/elevation for consistent visualization.

Hover tooltip shows exact Bloch vector coordinates (x, y, z) and purity percentage.

## Limits

Display is capped at 8 qubits. Beyond that, the partial trace computation becomes expensive (O(2^n) per qubit) and the visual layout impractical.

## Integration

`BlochSphere` is owned by `StateView` and rendered between the amplitude table and state metrics sections. It receives the `StateVector` reference from the simulator and recomputes Bloch vectors each frame (immediate mode).

Toggle: "Show Bloch Spheres" checkbox in the state panel.
