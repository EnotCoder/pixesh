use eframe::egui::{self, Pos2, Rect, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;

impl PixeshApp {
    pub(crate) fn ui_settings_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_settings { return; }
        egui::Area::new("settings_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(300.0, 280.0);
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
                    ui.label(egui::RichText::new("Settings").size(32.0).color(TEXT));
                });
                child_ui.add_space(16.0);

                // ── Arrow Speed ──
                child_ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Arrow Speed").size(24.0).color(TEXT));
                    ui.add_space(4.0);
                    ui.add_sized(
                        Vec2::new(200.0, 32.0),
                        egui::Slider::new(&mut self.arrow_speed, 5.0..=200.0)
                            .show_value(true)
                            .step_by(5.0),
                    );
                });
                child_ui.add_space(12.0);

                // ── Zoom Speed ──
                child_ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new("Zoom Speed").size(24.0).color(TEXT));
                    ui.add_space(4.0);
                    ui.add_sized(
                        Vec2::new(200.0, 32.0),
                        egui::Slider::new(&mut self.zoom_speed, 0.05..=1.0)
                            .show_value(true)
                            .step_by(0.05),
                    );
                });

                // ── Enter / Escape ──
                let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                let escape = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
                if enter || escape {
                    self.show_settings = false;
                }

                // ── OK button ──
                child_ui.add_space(child_ui.available_height() - 44.0);
                let w = rect.width();
                let btn_y = rect.max.y - 40.0;
                let btn_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x, btn_y),
                    Vec2::new(w, 40.0),
                );
                let btn_resp = ui.interact(btn_rect, egui::Id::new("btn_settings_ok"), egui::Sense::click());
                let bg = if btn_resp.clicked() { ACCENT } else if btn_resp.hovered() { HOVER } else { PANEL };
                let p2 = child_ui.painter();
                p2.rect_filled(btn_rect, 4.0, bg);
                p2.rect_stroke(btn_rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                p2.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "OK", egui::FontId::proportional(FONT_SZ), TEXT);
                if btn_resp.clicked() {
                    self.show_settings = false;
                }
            });
    }
}
