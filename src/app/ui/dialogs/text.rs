use eframe::egui::{self, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;
use crate::app::document::text::draw_text;

use super::fit::fitted_dialog;

impl PixeshApp {
    pub(crate) fn ui_text_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_text { return; }
        let i = self.active_tab;

        egui::Area::new("text_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(340.0, 240.0);
                fitted_dialog(ui, "text_scroll", size, |ui, rect| {
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
                    ui.label(egui::RichText::new("Add Text").size(32.0).color(TEXT));
                });

                child_ui.add_space(16.0);

                // text input
                let avail = child_ui.available_width() - 24.0;
                child_ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.add_sized(Vec2::new(avail, 40.0),
                        egui::TextEdit::singleline(&mut self.text_buffer)
                            .desired_width(avail)
                            .hint_text("Enter text..."));
                });

                child_ui.add_space(12.0);

                // scale selector
                child_ui.horizontal(|ui| {
                    ui.add_space(12.0);
                    ui.label("Scale:");
                    let mut scale_val = self.text_scale as i32;
                    ui.add(egui::DragValue::new(&mut scale_val).range(1..=8).speed(1.0));
                    self.text_scale = scale_val.max(1).min(8);
                });

                // Footer is anchored to the bottom of the dialog. The preview sits
                // just above it, so typing text cannot shift the buttons down.
                let footer_h = 44.0;
                let footer = egui::Rect::from_min_size(
                    egui::pos2(rect.min.x, rect.max.y - footer_h),
                    Vec2::new(rect.width(), footer_h),
                );

                // preview
                if !self.text_buffer.is_empty() {
                    if let Some((cx, cy)) = self.text_cursor {
                        let preview = format!("@({},{})", cx, cy);
                        let row_h = child_ui.text_style_height(&egui::TextStyle::Body);
                        let preview_rect = egui::Rect::from_min_size(
                            egui::pos2(rect.min.x + 12.0, footer.min.y - row_h),
                            Vec2::new(rect.width() - 24.0, row_h),
                        );
                        let mut preview_ui = child_ui.new_child(
                            egui::UiBuilder::new()
                                .layout(egui::Layout::left_to_right(egui::Align::Center))
                                .max_rect(preview_rect),
                        );
                        preview_ui.label(egui::RichText::new(preview).color(egui::Color32::GRAY));
                    }
                }

                let spacing = child_ui.style().spacing.item_spacing.x;
                let half_w = (footer.width() - spacing) / 2.0;
                let mut footer_ui = child_ui.new_child(
                    egui::UiBuilder::new()
                        .layout(egui::Layout::left_to_right(egui::Align::Center))
                        .max_rect(footer),
                );
                if btn_min_w(&mut footer_ui, "Draw", half_w) {
                    if !self.text_buffer.is_empty() {
                        if let Some((cx, cy)) = self.text_cursor {
                            draw_text(
                                &mut self.docs[i],
                                cx, cy,
                                &self.text_buffer,
                                self.color,
                                self.text_scale,
                            );
                        }
                    }
                    self.show_text = false;
                    self.text_buffer.clear();
                    self.text_cursor = None;
                }
                if btn_min_w(&mut footer_ui, "Cancel", half_w) {
                    self.show_text = false;
                    self.text_buffer.clear();
                    self.text_cursor = None;
                }
                });
            });
    }
}
