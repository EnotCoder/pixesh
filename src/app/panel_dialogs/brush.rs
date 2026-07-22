use eframe::egui::{self, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;

impl PixeshApp {
    pub(crate) fn ui_brush_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_brush { return; }
        let i = self.active_tab;
        let max = self.docs[i].width.max(self.docs[i].height) as f32;
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
}
