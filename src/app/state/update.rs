use eframe::egui::{self, Stroke, Vec2};

use crate::ui::btn_min_w;

use crate::constants::*;

use super::PixeshApp;

// ── eframe::App ──────────────────────────────────────
impl eframe::App for PixeshApp {
    /// eframe calls this once per auto_save_interval and on exit. eframe 0.31
    /// has no restore method, so reading lives in load_prefs.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "persistence")]
        eframe::set_value(storage, crate::app::config::PREFS_KEY, &self.prefs());
        #[cfg(not(feature = "persistence"))]
        let _ = storage;
    }

    /// Default is 30 seconds. That is how much is lost on a crash, and a panel
    /// is dragged with the mouse, so there is plenty of time to close the app
    /// in the meantime.
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(10)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.last_time = ctx.input(|i| i.time);
        self.handle_input(ctx);
        self.update_playback(ctx);
        self.handle_dropped_files(ctx);

        // ── Desktop-style layout for everyone ──
        // panel order is computed from the layout: top and bottom strips
        // first, then side panels, then the canvas
        for p in self.draw_order() {
            match p {
                PanelId::Toolbar => self.ui_toolbar(ctx),
                PanelId::Layers => self.ui_layers(ctx),
                PanelId::Status => self.ui_status(ctx),
                PanelId::Timeline => self.ui_timeline(ctx),
            }
        }
        self.ui_canvas(ctx);

        self.ui_dialogs(ctx);
        self.draw_drop_overlay(ctx);
        self.ui_panel_drag(ctx);

        if ctx.input(|i| i.viewport().close_requested()) && self.any_unsaved() && !self.show_quit_dialog && !self.close_handled {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_quit_dialog = true;
        }
        if !ctx.input(|i| i.viewport().close_requested()) {
            self.close_handled = false;
        }
        if self.show_quit_dialog {
            egui::Area::new("quit_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(260.0);

                    let t = ui.ctx().animate_bool(ui.make_persistent_id("quit_anim"), self.show_quit_dialog);
                    let size = size * (0.8 + 0.2 * t);

                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let p = ui.painter();

                    let _alpha = (t * 255.0) as u8;
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
                    child.label(egui::RichText::new("Unsaved changes!").size(26.0).color(TEXT));
                    child.add_space(12.0);
                    child.label(egui::RichText::new("Quit anyway?").size(22.0).color(TEXT));
                    child.add_space(child.available_height() - 44.0);
                    let spacing = child.style().spacing.item_spacing.x;
                    let half = (child.available_width() - spacing) / 2.0;
                    child.horizontal(|ui| {
                        if btn_min_w(ui, "Quit", half) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if btn_min_w(ui, "Cancel", half) {
                            self.show_quit_dialog = false;
                            self.close_handled = true;
                        }
                    });
                });
        }
    }
}
