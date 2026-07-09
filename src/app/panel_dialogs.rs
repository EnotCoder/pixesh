use eframe::egui::{self, Stroke, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

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
        // ── диалог изменения размера холста ──
        if self.show_resize {
            egui::Area::new("resize_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(300.0);
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
                        ui.label(egui::RichText::new("Resize Canvas").size(32.0).color(TEXT));
                    });
                    child_ui.add_space(20.0);
                    child_ui.vertical_centered(|ui| {
                        let mut w = self.resize_w as i32;
                        ui.add(
                            egui::DragValue::new(&mut w)
                                .range(1..=4096)
                                .prefix("W: "),
                        );
                        self.resize_w = w as f32;
                    });
                    child_ui.add_space(12.0);
                    child_ui.vertical_centered(|ui| {
                        let mut h = self.resize_h as i32;
                        ui.add(
                            egui::DragValue::new(&mut h)
                                .range(1..=4096)
                                .prefix("H: "),
                        );
                        self.resize_h = h as f32;
                    });
                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    if enter {
                        if self.resize_w as usize != self.width
                            || self.resize_h as usize != self.height
                        {
                            self.resize_canvas(self.resize_w as usize, self.resize_h as usize);
                        }
                        self.show_resize = false;
                    }
                    child_ui.add_space(child_ui.available_height() - 44.0);
                    let spacing = child_ui.style().spacing.item_spacing.x;
                    let half_w = (child_ui.available_width() - spacing) / 2.0;
                    child_ui.horizontal(|ui| {
                        if btn_min_w(ui, "Apply", half_w) {
                            if self.resize_w as usize != self.width
                                || self.resize_h as usize != self.height
                            {
                                self.resize_canvas(self.resize_w as usize, self.resize_h as usize);
                            }
                            self.show_resize = false;
                        }
                        if btn_min_w(ui, "Cancel", half_w) {
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

        // ── диалог размера кисти ──
        if self.show_brush {
            let max = self.width.max(self.height) as f32;
            egui::Area::new("brush_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(300.0);
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(rect, 0.0, PANEL);
                    p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    let mut child_ui = ui.new_child(egui::UiBuilder::new().layout(egui::Layout::top_down(egui::Align::Min)).max_rect(rect));
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
                        ui.label(egui::RichText::new("Brush Size").size(32.0).color(TEXT));
                    });
                    child_ui.add_space(50.0);
                    child_ui.vertical_centered(|ui| {
                        let mut val = self.brush as i32;
                        ui.add_sized(Vec2::new(120.0, 48.0),
                            egui::DragValue::new(&mut val)
                                .range(1..=max as i32)
                                .speed(1.0)
                        );
                        self.brush = val as f32;
                    });
                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    if enter { self.show_brush = false; }
                    child_ui.add_space(child_ui.available_height() - 44.0);
                    let w = child_ui.available_width();
                    if btn_min_w(&mut child_ui, "OK", w) {
                        self.show_brush = false;
                    }
                });
        }

        // ── диалог управления панелями ──
        if self.show_panels {
            egui::Area::new("panels_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(250.0);
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
                        ui.label(egui::RichText::new("Panels").size(32.0).color(TEXT));
                    });
                    child_ui.add_space(20.0);
                    child_ui.vertical_centered(|ui| {
                        checkbox_w(ui, "Toolbar", &mut self.show_top_panel, 180.0);
                        ui.add_space(8.0);
                        checkbox_w(ui, "Layers", &mut self.show_right_panel, 180.0);
                    });
                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
                    if enter || escape {
                        self.show_panels = false;
                    }
                    child_ui.add_space(child_ui.available_height() - 44.0);
                    let w = child_ui.available_width();
                    if btn_min_w(&mut child_ui, "OK", w) {
                        self.show_panels = false;
                    }
                });
        }
    }
}
