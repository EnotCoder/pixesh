use eframe::egui::{self, Pos2, Rect, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;

impl PixeshApp {
    pub(crate) fn ui_panels_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_panels { return; }
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
                child_ui.add_space(6.0);
                child_ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Panels").size(32.0).color(TEXT));
                });
                child_ui.add_space(20.0);
                child_ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
                    let cbs = 24.0;
                    let sz = FONT_SZ * 2.0;
                    let row_h = sz + 16.0;

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
                    p.rect_filled(cb_rect, 0.0, PANEL);
                    p.rect_stroke(cb_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    if self.show_top_panel {
                        let inner = cb_rect.shrink(4.0);
                        p.rect_filled(inner, 0.0, ACCENT);
                    }
                    if ui.interact(row_rect, egui::Id::new("row_top"), egui::Sense::click()).clicked()
                        || ui.interact(cb_rect, egui::Id::new("cb_top"), egui::Sense::click()).clicked()
                    {
                        self.show_top_panel = !self.show_top_panel;
                    }

                    let line = ui.allocate_exact_size(Vec2::new(160.0, 1.0), egui::Sense::hover()).0;
                    ui.painter().hline(line.x_range(), line.center().y, egui::Stroke::new(1.0, BORDER));

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
                    p.rect_filled(cb_rect, 0.0, PANEL);
                    p.rect_stroke(cb_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    if self.show_right_panel {
                        let inner = cb_rect.shrink(4.0);
                        p.rect_filled(inner, 0.0, ACCENT);
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
                let btn_y = rect.max.y - 40.0;
                let btn_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, btn_y),
                    Vec2::new(w, 40.0),
                );
                let p = child_ui.painter();
                let btn_resp = ui.interact(btn_rect, egui::Id::new("btn_ok"), egui::Sense::click());
                let bg = if btn_resp.clicked() { ACCENT } else if btn_resp.hovered() { HOVER } else { PANEL };
                p.rect_filled(btn_rect, 0.0, bg);
                p.rect_stroke(btn_rect, 0.0, egui::Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                p.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "OK", egui::FontId::proportional(FONT_SZ), TEXT);
                if btn_resp.clicked() {
                    self.show_panels = false;
                }
            });
    }
}
