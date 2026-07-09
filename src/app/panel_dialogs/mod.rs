use eframe::egui;
use crate::constants::*;
use super::PixeshApp;

mod resize;
mod export;
mod brush;
mod panels;

impl PixeshApp {
    fn dialog_win(ui: &mut egui::Ui, ctx: &egui::Context, title: &str, add_content: impl FnOnce(&mut egui::Ui, &egui::Context)) {
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Body,
            egui::FontId::proportional(28.0),
        );
        ui.style_mut().text_styles.insert(
            egui::TextStyle::Button,
            egui::FontId::proportional(28.0),
        );
        ui.add_space(6.0);
        ui.vertical_centered(|ui| {
            ui.label(egui::RichText::new(title).size(28.0).color(TEXT));
        });
        ui.add_space(8.0);
        add_content(ui, ctx);
    }

    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        self.ui_resize_dialog(ctx);
        self.ui_export_dialog(ctx);
        self.ui_brush_dialog(ctx);
        self.ui_panels_dialog(ctx);
    }
}
