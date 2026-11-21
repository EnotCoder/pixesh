// Without this attribute Windows opens a black console window alongside the app:
// main prints nothing to stdout, so hiding the console is safe.
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;

// ── desktop entry ────────────────────────────────────
fn main() -> eframe::Result {
    // Arguments are passed through to the library: files given on the command
    // line open as tabs at startup, --version/--help exit immediately.
    let args: Vec<std::ffi::OsString> = std::env::args_os().skip(1).collect();
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])   // initial window size
            .with_min_inner_size([400.0, 300.0]) // minimum window size
            .with_maximized(true),
        ..Default::default()
    };
    pixesh::run_native_app(native_options, args)
}
