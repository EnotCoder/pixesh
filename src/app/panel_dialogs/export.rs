use eframe::egui::{self, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;

impl PixeshApp {
    pub(crate) fn ui_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export { return; }
        egui::Area::new("export_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(380.0, 260.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, 0.0, PANEL);
                p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                let mut child_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .layout(egui::Layout::top_down(egui::Align::Center))
                        .max_rect(rect)
                );
                child_ui.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::proportional(28.0),
                );
                child_ui.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::proportional(28.0),
                );
                child_ui.add_space(8.0);
                child_ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Export PNG").size(32.0).color(TEXT));
                });
                child_ui.add_space(20.0);
                child_ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label("Folder:");
                    let display = if self.export_path.is_empty() { "." } else { &self.export_path };
                    ui.add_sized(Vec2::new(220.0, 44.0), egui::Label::new(display));
                    if btn(ui, "…") {
                        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                        if let Some(p) = rfd::FileDialog::new()
                            .set_directory(&home)
                            .pick_folder()
                        {
                            self.export_path = p.to_string_lossy().into();
                        }
                    }
                });
                child_ui.add_space(12.0);
                child_ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.label("File:");
                    ui.add_sized(
                        Vec2::new(220.0, 44.0),
                        egui::TextEdit::singleline(&mut self.export_name),
                    );
                });
                let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                if enter {
                    let path = if self.export_path.is_empty() {
                        self.export_name.clone()
                    } else {
                        format!("{}/{}", self.export_path, self.export_name)
                    };
                    self.save_png(&path);
                    self.show_export = false;
                }
                child_ui.add_space(child_ui.available_height() - 44.0);
                let spacing = child_ui.style().spacing.item_spacing.x;
                let half_w = (child_ui.available_width() - spacing) / 2.0;
                child_ui.horizontal(|ui| {
                    if btn_min_w(ui, "Save", half_w) {
                        let path = if self.export_path.is_empty() {
                            self.export_name.clone()
                        } else {
                            format!("{}/{}", self.export_path, self.export_name)
                        };
                        self.save_png(&path);
                        self.show_export = false;
                    }
                    if btn_min_w(ui, "Cancel", half_w) {
                        self.show_export = false;
                    }
                });
            });
    }
}
