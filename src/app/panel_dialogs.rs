use eframe::egui::{self, Stroke, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

impl PixeshApp {
    // модальные окна: Resize Canvas и Export PNG
    pub(crate) fn ui_dialogs(&mut self, ctx: &egui::Context) {
        // ── диалог изменения размера холста ──
        if self.show_resize {
            egui::Window::new("Resize Canvas")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(28.0),
                    );
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        ui.label("W:");
                        let mut w = self.resize_w as i32;
                        ui.add_sized(Vec2::new(120.0, 44.0), egui::DragValue::new(&mut w).range(1..=4096));
                        self.resize_w = w as f32;
                    });
                    ui.horizontal(|ui| {
                        ui.label("H:");
                        let mut h = self.resize_h as i32;
                        ui.add_sized(Vec2::new(120.0, 44.0), egui::DragValue::new(&mut h).range(1..=4096));
                        self.resize_h = h as f32;
                    });
                    ui.add_space(12.0);

                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    if enter {
                        if self.resize_w as usize != self.width
                            || self.resize_h as usize != self.height
                        {
                            self.resize_canvas(self.resize_w as usize, self.resize_h as usize);
                        }
                        self.show_resize = false;
                    }

                    ui.horizontal(|ui| {
                        if btn(ui, "Apply") {
                            if self.resize_w as usize != self.width
                                || self.resize_h as usize != self.height
                            {
                                self.resize_canvas(self.resize_w as usize, self.resize_h as usize);
                            }
                            self.show_resize = false;
                        }
                        if btn(ui, "Cancel") {
                            self.show_resize = false;
                        }
                    });
                });
        }

        // ── диалог экспорта в PNG ──
        if self.show_export {
            egui::Window::new("Export PNG")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(28.0),
                    );
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    ui.add_space(8.0);
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
                        if btn(ui, "Save") {
                            let path = if self.export_path.is_empty() {
                                self.export_name.clone()
                            } else {
                                format!("{}/{}", self.export_path, self.export_name)
                            };
                            self.save_png(&path);
                            self.show_export = false;
                        }
                        if btn(ui, "Cancel") {
                            self.show_export = false;
                        }
                    });
                });
        }

        // ── диалог размера кисти ──
        if self.show_brush {
            let max = self.width.max(self.height) as f32;
            egui::Window::new("Brush Size")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(28.0),
                    );
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    ui.add_space(8.0);

                    ui.horizontal(|ui| {
                        ui.label("Size:");
                        let mut val = self.brush as i32;
                        ui.add_sized(Vec2::new(100.0, 44.0),
                            egui::DragValue::new(&mut val)
                                .range(1..=max as i32)
                                .speed(1.0)
                        );
                        self.brush = val as f32;
                    });

                    ui.add_space(12.0);

                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    if enter { self.show_brush = false; }

                    ui.horizontal(|ui| {
                        if btn(ui, "OK") {
                            self.show_brush = false;
                        }
                    });
                });
        }
    }
}
