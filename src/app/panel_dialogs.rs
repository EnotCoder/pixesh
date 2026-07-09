use eframe::egui::{self, Pos2, Rect, Stroke, Vec2};

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
                    let inner = rect.shrink2(Vec2::splat(6.0));
                    let mut child_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::top_down(egui::Align::Center))
                            .max_rect(inner)
                    );
                    child_ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(28.0),
                    );
                    child_ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    // ── отступ сверху ──
                    child_ui.add_space(6.0);
                    child_ui.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("Panels").size(32.0).color(TEXT));
                    });
                    child_ui.add_space(20.0);
                    child_ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                        let cbs = 24.0;
                        let sz = FONT_SZ * 2.0;
                        let row_h = sz + 16.0;

                        // ── Toolbar ──
                        let (row_rect, _) = ui.allocate_exact_size(Vec2::new(160.0, row_h), egui::Sense::click());
                        let p = ui.painter();
                        let text_y = row_rect.center().y - sz * 0.5;
                        p.text(
                            Pos2::new(row_rect.min.x + 4.0, text_y),
                            egui::Align2::LEFT_TOP,
                            "Toolbar",
                            egui::FontId::proportional(sz),
                            TEXT,
                        );
                        let cb_rect = Rect::from_min_size(
                            Pos2::new(row_rect.max.x - cbs - 4.0, row_rect.center().y - cbs * 0.5),
                            Vec2::splat(cbs),
                        );
                        p.rect_filled(cb_rect, 3.0, PANEL);
                        p.rect_stroke(cb_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                        if self.show_top_panel {
                            let inner = cb_rect.shrink(4.0);
                            p.rect_filled(inner, 2.0, ACCENT);
                        }
                        if ui.interact(row_rect, egui::Id::new("row_top"), egui::Sense::click()).clicked()
                            || ui.interact(cb_rect, egui::Id::new("cb_top"), egui::Sense::click()).clicked()
                        {
                            self.show_top_panel = !self.show_top_panel;
                        }

                        // ── разделитель ──
                        let line = ui.allocate_exact_size(Vec2::new(160.0, 1.0), egui::Sense::hover()).0;
                        ui.painter().hline(line.x_range(), line.center().y, egui::Stroke::new(1.0, BORDER));

                        // ── Layers ──
                        let (row_rect, _) = ui.allocate_exact_size(Vec2::new(160.0, row_h), egui::Sense::click());
                        let p = ui.painter();
                        let text_y = row_rect.center().y - sz * 0.5;
                        p.text(
                            Pos2::new(row_rect.min.x + 4.0, text_y),
                            egui::Align2::LEFT_TOP,
                            "Layers",
                            egui::FontId::proportional(sz),
                            TEXT,
                        );
                        let cb_rect = Rect::from_min_size(
                            Pos2::new(row_rect.max.x - cbs - 4.0, row_rect.center().y - cbs * 0.5),
                            Vec2::splat(cbs),
                        );
                        p.rect_filled(cb_rect, 3.0, PANEL);
                        p.rect_stroke(cb_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                        if self.show_right_panel {
                            let inner = cb_rect.shrink(4.0);
                            p.rect_filled(inner, 2.0, ACCENT);
                        }
                        if ui.interact(row_rect, egui::Id::new("row_right"), egui::Sense::click()).clicked()
                            || ui.interact(cb_rect, egui::Id::new("cb_right"), egui::Sense::click()).clicked()
                        {
                            self.show_right_panel = !self.show_right_panel;
                        }
                    });
                    let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
                    if enter || escape {
                        self.show_panels = false;
                    }
                    child_ui.add_space(child_ui.available_height() - 44.0);
                    let w = rect.width();
                    let btn_y = rect.max.y - 44.0;
                    let btn_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x, btn_y),
                        Vec2::new(w, 40.0),
                    );
                    let p = child_ui.painter();
                    p.rect_filled(btn_rect, 4.0, PANEL);
                    p.rect_stroke(btn_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    p.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "OK", egui::FontId::proportional(FONT_SZ), TEXT);
                    if ui.interact(btn_rect, egui::Id::new("btn_ok"), egui::Sense::click()).clicked() {
                        self.show_panels = false;
                    }
                });
        }
    }
}
