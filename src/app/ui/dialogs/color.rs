use eframe::egui::{self, Vec2};

use crate::app::state::color::{format_rgb_input, parse_rgb_input};
use crate::app::PixeshApp;
use crate::constants::*;

use super::fit::fitted_dialog;

impl PixeshApp {
    /// Ctrl+K colour entry. Parsing happens on every frame, so the colour
    /// updates while typing. It calls `set_color`, not `push_history`: this is
    /// a picker, not a preset, and must not leave undo/history entries.
    pub(crate) fn ui_color_input_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_color_input { return; }

        egui::Area::new("color_input_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                // egui 0.31 has no TextEdit::auto_focus; request focus on our
                // explicit id instead so typing works as soon as Ctrl+K opens.
                let field_id = egui::Id::new("color_input_field");
                ui.memory_mut(|m| m.request_focus(field_id));
                let size = Vec2::new(340.0, 190.0);
                fitted_dialog(ui, "color_input_scroll", size, |ui, rect| {
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::top_down(egui::Align::Center))
                            .max_rect(rect),
                    );
                    child.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(24.0),
                    );

                    child.add_space(12.0);
                    child.vertical_centered(|ui| {
                        ui.label(egui::RichText::new("RGB colour").size(28.0).color(TEXT));
                    });
                    child.add_space(12.0);

                    let field_w = (rect.width() - 32.0).max(64.0);
                    child.horizontal(|ui| {
                        ui.add_space(16.0);
                        ui.add_sized(
                            Vec2::new(field_w, 38.0),
                            egui::TextEdit::singleline(&mut self.color_input_buf)
                                .id(field_id)
                                .desired_width(field_w)
                                .hint_text("r,g,b[,a] or #RRGGBB[AA]")
                                .return_key(None),
                        );
                    });

                    match parse_rgb_input(&self.color_input_buf) {
                        Some(c) => {
                            self.color_input_error = None;
                            // Live application: no commit button and no history entry.
                            self.set_color(c);
                        }
                        None => {
                            self.color_input_error = Some("Use 0-255 per channel or #RRGGBB".into());
                        }
                    }

                    child.add_space(8.0);
                    if let Some(err) = &self.color_input_error {
                        child.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(err).size(18.0).color(egui::Color32::RED));
                        });
                    } else {
                        let text = format_rgb_input(self.color);
                        child.vertical_centered(|ui| {
                            ui.label(egui::RichText::new(text).size(18.0).color(TEXT));
                        });
                    }

                    child.add_space(6.0);
                    child.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Enter / Esc to close - no history entry")
                                .size(16.0)
                                .color(egui::Color32::GRAY),
                        );
                    });

                    // TextEdit has return_key(None), so Enter belongs to us.
                    if ui.ctx().input_mut(|i| {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    }) {
                        self.show_color_input = false;
                        self.color_input_buf.clear();
                        self.color_input_error = None;
                    }
                });
            });
    }
}
