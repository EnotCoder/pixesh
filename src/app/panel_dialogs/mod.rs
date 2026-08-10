use eframe::egui;
use super::PixeshApp;

mod resize;
mod export;
mod brush;
mod panels;
mod settings;
mod scale;
mod text;
mod welcome;

impl PixeshApp {
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        self.ui_welcome_dialog(ctx);
        self.ui_resize_dialog(ctx);
        self.ui_export_dialog(ctx);
        self.ui_brush_dialog(ctx);
        self.ui_panels_dialog(ctx);
        self.ui_settings_dialog(ctx);
        self.ui_scale_dialog(ctx);
        self.ui_text_dialog(ctx);
    }
}
