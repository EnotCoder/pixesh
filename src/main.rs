// без этой строки Windows открывает рядом с окном чёрное окно консоли:
// main печатает в stdout только панорамные сообщения, терять нечего
#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

use eframe::egui;

// ── desktop entry ────────────────────────────────────
fn main() -> eframe::Result {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1400.0, 900.0])   // начальный размер окна
            .with_min_inner_size([400.0, 300.0]) // минимальный размер
            .with_maximized(true),
        ..Default::default()
    };
    pixesh::run_native_app(native_options)
}
