# Status Bar

The status bar displays real-time circuit metrics at the bottom of the application window.

## Features

### Metrics Display
- **Qubit count**: Current number of qubit wires in the circuit
- **Gate count**: Total number of placed gates
- **Memory estimate**: Approximate RAM required for state vector simulation (2^n × 16 bytes)

### Memory Warnings
The status bar shows colored warnings when memory usage approaches limits:
- **Red warning (⚠)**: State vector would exceed 90% of available system RAM
- **Yellow warning**: High qubit count (>30 qubits, ~16GB+ required)

### Memory Formatting
Memory estimates display in human-readable units:
- Bytes for <1KB
- KB for 1KB-1MB
- MB for 1MB-1GB
- GB for >1GB

## Implementation

Location: `crates/ketgrid-gui/src/app.rs`

Key functions:
- `estimate_state_vector_memory_bytes(n)` - Calculates 2^n × 16 bytes for complex amplitudes
- `format_memory(bytes)` - Formats bytes into B/KB/MB/GB
- `get_system_memory()` - Windows API query for available RAM

The status bar updates instantly via egui's `update()` loop whenever the circuit changes.

## Testing

Verified behaviors:
- 12 qubits displays ~64KB (2^12 × 16 = 65,536 bytes)
- Circuit modifications update metrics immediately
- Memory warnings trigger at appropriate thresholds
