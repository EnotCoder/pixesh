use eframe::egui;
use super::PixeshApp;

mod resize;
mod export;
mod brush;
mod panels;
mod settings;

impl PixeshApp {
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        self.ui_resize_dialog(ctx);
        self.ui_export_dialog(ctx);
        self.ui_brush_dialog(ctx);
        self.ui_panels_dialog(ctx);
        self.ui_settings_dialog(ctx);
    }
}
