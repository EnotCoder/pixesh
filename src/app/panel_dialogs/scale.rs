use eframe::egui::{self, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;

impl PixeshApp {
    // ── диалог масштабирования изображения ──
    // меняет размер картинки (пиксели сэмплируются заново), а не холста
    pub(crate) fn ui_scale_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_scale { return; }

        egui::Area::new("scale_dialog".into())
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
                    ui.label(egui::RichText::new("Scale Canvas").size(32.0).color(TEXT));
                });

                child_ui.add_space(20.0);
                child_ui.vertical_centered(|ui| {
                    let mut w = self.scale_w as i32;
                    ui.add(
                        egui::DragValue::new(&mut w)
                            .range(1..=4096)
                            .prefix("W: "),
                    );
                    self.scale_w = w as f32;
                });
                child_ui.add_space(12.0);
                child_ui.vertical_centered(|ui| {
                    let mut h = self.scale_h as i32;
                    ui.add(
                        egui::DragValue::new(&mut h)
                            .range(1..=4096)
                            .prefix("H: "),
                    );
                    self.scale_h = h as f32;
                });

                let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                if enter {
                    if self.scale_w as usize != self.width
                        || self.scale_h as usize != self.height
                    {
                        self.scale_image(self.scale_w as usize, self.scale_h as usize);
                    }
                    self.show_scale = false;
                }

                child_ui.add_space(child_ui.available_height() - 44.0);
                let spacing = child_ui.style().spacing.item_spacing.x;
                let half_w = (child_ui.available_width() - spacing) / 2.0;
                child_ui.horizontal(|ui| {
                    if btn_min_w(ui, "Apply", half_w) {
                        if self.scale_w as usize != self.width
                            || self.scale_h as usize != self.height
                        {
                            self.scale_image(self.scale_w as usize, self.scale_h as usize);
                        }
                        self.show_scale = false;
                    }
                    if btn_min_w(ui, "Cancel", half_w) {
                        self.show_scale = false;
                    }
                });
            });
    }
}
