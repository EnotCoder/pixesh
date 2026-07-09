use eframe::egui::{self, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;

impl PixeshApp {
    pub(crate) fn ui_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export { return; }
        egui::Window::new("Export PNG")
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .collapsible(false)
            .title_bar(false)
            .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
            .show(ctx, |ui| {
                Self::dialog_win(ui, ctx, "Export PNG", |ui, ctx| {
                    ui.horizontal(|ui| {
                        ui.label("Folder:");
                        let display = if self.export_path.is_empty() { "." } else { &self.export_path };
                        ui.add_sized(Vec2::new(280.0, 44.0), egui::Label::new(display));
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
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        ui.add_sized(
                            Vec2::new(280.0, 44.0),
                            egui::TextEdit::singleline(&mut self.export_name),
                        );
                    });
                    ui.add_space(12.0);

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

                    ui.horizontal(|ui| {
                        if btn_min_w(ui, "Save", 121.0) {
                            let path = if self.export_path.is_empty() {
                                self.export_name.clone()
                            } else {
                                format!("{}/{}", self.export_path, self.export_name)
                            };
                            self.save_png(&path);
                            self.show_export = false;
                        }
                        if btn_min_w(ui, "Cancel", 121.0) {
                            self.show_export = false;
                        }
                    });
                });
            });
    }
}
