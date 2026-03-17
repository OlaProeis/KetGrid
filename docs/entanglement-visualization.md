# Entanglement Visualization

Color-codes qubit wires by entanglement cluster so users can see at a glance which qubits share quantum correlations.

## How It Works

### Detection Algorithm

1. **Single-qubit purity** — For each qubit, compute the purity Tr(ρ²) of its reduced density matrix. Purity = 1.0 means the qubit is separable; purity < 1.0 means it is entangled with at least one other qubit.

2. **Pairwise correlation** — For each pair of non-pure qubits, compute the 4×4 two-qubit reduced density matrix ρ_ij by partial trace. Compare its purity against the product of single-qubit purities. A deviation exceeding the threshold (0.001) indicates quantum correlation between the pair.

3. **Cluster grouping** — Union-find groups pairwise-correlated qubits into clusters. Each cluster is a set of mutually entangled qubits.

### Visual Encoding

- **Entangled qubits**: wire and label drawn in a distinct cluster color (from an 8-color palette), with a small colored dot next to the qubit label and a thicker wire (2.5px).
- **Unentangled qubits**: default wire and label color.
- **State panel**: shows cluster membership and per-qubit purity percentages, color-coded to match the circuit view.

### Palette

| Index | Color   | RGB             |
|-------|---------|-----------------|
| 0     | Red     | (255, 100, 100) |
| 1     | Blue    | (100, 160, 255) |
| 2     | Green   | (100, 220, 100) |
| 3     | Purple  | (200, 130, 255) |
| 4     | Orange  | (255, 180, 60)  |
| 5     | Pink    | (255, 120, 200) |
| 6     | Teal    | (60, 220, 200)  |
| 7     | Yellow  | (220, 200, 80)  |

Colors cycle if more than 8 entanglement clusters exist.

## Integration Points

- Computed after every simulation update (debounced background sim, step-through mode).
- Works with both the main simulator and the step simulator.
- Cached in `KetGridApp` to avoid per-frame recomputation.

## Test Strategy

- **Bell state** (H → CNOT): q0 and q1 share the same cluster color.
- **Single H**: q0 and q1 in separate clusters, no entanglement coloring.
- **GHZ state**: all 3 qubits in one cluster.
- **Partial entanglement** (Bell on q0,q1 + idle q2): two clusters.

## Key Types

- `EntanglementInfo` — clusters, per-qubit purities, qubit-to-cluster mapping (in `ketgrid-sim`).
- `compute_entanglement_info(sv)` — entry point for detection (in `ketgrid-sim`).
- `entanglement_wire_colors(info)` — maps clusters to `Option<Color32>` per qubit (in `ketgrid-gui`).
