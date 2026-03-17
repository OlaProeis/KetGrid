//! KetGrid GUI — Native quantum circuit editor and simulator.

use eframe::NativeOptions;

mod app;
mod bloch;
mod circuit_view;
mod editor;
mod examples;
mod gate_palette;
mod history;
mod state_view;
mod stats_panel;

use app::KetGridApp;

fn main() -> eframe::Result {
    #[cfg(not(target_arch = "wasm32"))]
    tracing_subscriber::fmt::init();

    let options = NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "KetGrid — Quantum Circuit Editor",
        options,
        Box::new(|cc| Ok(Box::new(KetGridApp::new(cc)))),
    )
}
