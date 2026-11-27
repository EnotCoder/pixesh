use eframe::egui::{self, Stroke, Vec2};

use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::btn_min_w;

impl PixeshApp {
    /// Mirrors the quit confirmation (`state/update.rs`): same 260 × 260 frame,
    /// same fade/scale animation and the same `btn_min_w` action buttons.
    pub(crate) fn ui_clear_confirm_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_clear_confirm { return; }
        egui::Area::new("clear_confirm_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::splat(260.0);

                let t = ui.ctx().animate_bool(
                    ui.make_persistent_id("clear_confirm_anim"),
                    self.show_clear_confirm,
                );
                let size = size * (0.8 + 0.2 * t);

                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let p = ui.painter();

                let bg = PANEL.gamma_multiply(t);
                let brd = BORDER.gamma_multiply(t);

                p.rect_filled(rect, 0.0, bg);
                p.rect_stroke(rect, 0.0, Stroke::new(4.0_f32, brd), egui::StrokeKind::Outside);
                let mut child = ui.new_child(
                    egui::UiBuilder::new()
                        .layout(egui::Layout::top_down(egui::Align::Center))
                        .max_rect(rect),
                );
                child.style_mut().text_styles.insert(
                    egui::TextStyle::Body,
                    egui::FontId::proportional(24.0),
                );
                child.style_mut().text_styles.insert(
                    egui::TextStyle::Button,
                    egui::FontId::proportional(28.0),
                );
                child.add_space(24.0);
                child.label(egui::RichText::new("Clear all layers?").size(26.0).color(TEXT));
                child.add_space(12.0);
                child.label(
                    egui::RichText::new("This can be undone with Ctrl+Z.")
                        .size(20.0)
                        .color(DIM),
                );
                child.add_space(child.available_height() - 44.0);
                let spacing = child.style().spacing.item_spacing.x;
                let half = (child.available_width() - spacing) / 2.0;
                child.horizontal(|ui| {
                    if btn_min_w(ui, "Yes", half) {
                        let tab = self.active_tab;
                        self.docs[tab].clear_all_layers();
                        self.show_clear_confirm = false;
                    }
                    if btn_min_w(ui, "No", half) {
                        self.show_clear_confirm = false;
                    }
                });
            });
    }
}
