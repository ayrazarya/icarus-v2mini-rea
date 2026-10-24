//! Graphical User Interface (GUI) for ICARUS V2 MINI.
//!
//! Recreating the original Windows OEM application with all reverse-engineered
//! features, original icons, vector mouse canvas, and real-time lighting previews.

mod app;
pub mod state;
pub mod tabs;
pub mod theme;
pub mod widgets;

pub use state::{IcarusApp, PairingState, Tab};

/// Run the native Desktop GUI window.
pub fn run_gui() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([960.0, 680.0])
            .with_min_inner_size([900.0, 620.0])
            .with_title("ICARUS V2 MINI — Mouse Configuration"),
        ..Default::default()
    };

    eframe::run_native(
        "ICARUS V2 MINI",
        native_options,
        Box::new(|cc| Ok(Box::new(IcarusApp::new(cc)))),
    )
}
