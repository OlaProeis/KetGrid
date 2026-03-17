# Debounced Background Simulation

Real-time simulation feedback with 100ms debounce and background threading.

## Architecture

All circuit edits call `mark_sim_dirty()` or `mark_sim_dirty_at(column)` instead of running simulation synchronously. A debounce timer + background `std::thread` ensures the UI stays responsive during rapid edits.

### Key Components

| Component | Location | Purpose |
|-----------|----------|---------|
| `mark_sim_dirty()` | `app.rs` | Full circuit change — clears dirty_column and simulator |
| `mark_sim_dirty_at(col)` | `app.rs` | Edit at specific column — tracks earliest dirty column |
| `start_background_sim()` | `app.rs` | Spawns background thread with incremental or optimized sim |
| `poll_simulation_results()` | `app.rs` | Checks `mpsc` channel for results, fires debounce timer |
| `force_simulate()` | `app.rs` | Manual trigger for large circuits (>20 qubits) |

### Constants

| Constant | Value | Purpose |
|----------|-------|---------|
| `DEBOUNCE_DURATION` | 100ms | Wait time after last edit before auto-sim |
| `AUTO_SIM_MAX_QUBITS` | 15 | Qubit threshold; above this, auto-sim is disabled |

## Flow

```
Edit at column C → mark_sim_dirty_at(Some(C)) → sim_stale = true, dirty_column = min(existing, C)
Full circuit change → mark_sim_dirty() → sim_stale = true, dirty_column = None, simulator = None
                                ↓
         poll_simulation_results() [called every frame]
                                ↓
         100ms elapsed && !sim_running && qubits ≤ 15?
                    ↓ yes                    ↓ no (>15 qubits)
         start_background_sim()        Show "▶ Simulate" button
                    ↓
         Has existing simulator + dirty_column?
            ↓ yes                          ↓ no
         Incremental: restore from     Full: new simulator +
         column checkpoint, re-sim     apply_circuit_optimized()
         from dirty_column onward      (with gate fusion)
                    ↓
         Sends result via mpsc channel
                    ↓
         Next frame: poll picks up result → sim_stale = false
```

## Incremental Simulation

When a gate is edited at a specific column, the app tracks `dirty_column: Option<usize>` — the earliest modified column. On the next simulation cycle:

1. The existing `StateVectorSimulator` (with its column checkpoints) is passed to the background thread.
2. `apply_circuit_from_column(circuit, dirty_column)` restores state from the nearest checkpoint and re-simulates only from that column forward.
3. This avoids re-simulating the entire circuit for small edits in the middle or end.

When `dirty_column` is `None` (structural change like adding/removing qubits), a fresh simulator is created with `apply_circuit_optimized()` which uses gate fusion.

## Threading Model

- Uses `std::thread` + `std::sync::mpsc` (no tokio dependency).
- Only one background sim runs at a time (`sim_running` guard).
- If edits occur during a running sim, the result is accepted but `sim_stale` remains true, and the debounce cycle restarts.
- `ctx.request_repaint_after()` schedules egui repaints for debounce expiry and result polling.

## Large Circuit Handling (>20 qubits)

- Auto-sim is disabled; `sim_dirty_since` is not set.
- State panel shows a "▶ Simulate" button and "Results outdated" message.
- `force_simulate()` bypasses the debounce and immediately spawns a background sim.
- Spinner + "Simulating…" label shows while the background thread runs.

## Staleness Indicators

- When `sim_stale == true` and qubit count changes, old simulator results are cleared (`simulator = None`) to avoid showing wrong-qubit-count data.
- When `sim_stale == true` and qubit count is unchanged, old results remain visible as a reasonable approximation while the new sim runs.

## Struct Fields

```rust
sim_dirty_since: Option<Instant>,              // Debounce timestamp
sim_result_tx: mpsc::Sender<StateVectorSimulator>,   // Thread → main channel
sim_result_rx: mpsc::Receiver<StateVectorSimulator>,  // Main polls this
sim_running: bool,                              // One-sim-at-a-time guard
sim_stale: bool,                                // Display freshness flag
dirty_column: Option<usize>,                   // Earliest modified column for incremental sim
```
