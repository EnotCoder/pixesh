use eframe::egui::{self, Stroke};

use crate::constants::*;
use crate::ui::separator;
use super::PixeshApp;

impl PixeshApp {
    pub(crate) fn ui_status(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("status")
            .frame(egui::Frame::new().fill(PANEL))
            .show_separator_line(false)
            .show(ctx, |ui| {
                let panel_left = ui.max_rect().left();
                let panel_right = ui.max_rect().right();
                let panel_top = ui.max_rect().top();
                ui.painter().hline(panel_left..=panel_right, panel_top, Stroke::new(8.0, BORDER));
                ui.add_space(2.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);

                    let i = self.active_tab;
                    let doc = &self.docs[i];

                    // tool name
                    let tool_name = match self.tool {
                        Tool::Brush => "Brush",
                        Tool::Eraser => "Eraser",
                        Tool::Fill => "Fill",
                        Tool::Eyedropper => "Dropper",
                        Tool::Select => "Select",
                        Tool::Move => "Move",
                        Tool::Text => "Text",
                    };
                    ui.label(
                        egui::RichText::new(format!("Tool: {}", tool_name))
                            .size(FONT_SZ)
                            .color(TEXT),
                    );

                    separator(ui);

                    // canvas size
                    ui.label(
                        egui::RichText::new(format!("Canvas: {}x{}", doc.width, doc.height))
                            .size(FONT_SZ)
                            .color(TEXT),
                    );

                    separator(ui);

                    // cursor position
                    if let Some((px, py)) = self.cursor_px {
                        ui.label(
                            egui::RichText::new(format!("Pos: ({}, {})", px, py))
                                .size(FONT_SZ)
                                .color(TEXT),
                        );
                    } else {
                        ui.label(
                            egui::RichText::new("Pos: --")
                                .size(FONT_SZ)
                                .color(DIM),
                        );
                    }

                    separator(ui);

                    // zoom
                    let zoom_pct = (doc.zoom * 12.5) as u32;
                    ui.label(
                        egui::RichText::new(format!("Zoom: {}%", zoom_pct))
                            .size(FONT_SZ)
                            .color(TEXT),
                    );

                    separator(ui);

                    // active layer name
                    if let Some(layer) = doc.layers.get(doc.active_layer) {
                        ui.label(
                            egui::RichText::new(format!("Layer: {}", layer.name))
                                .size(FONT_SZ)
                                .color(TEXT),
                        );
                    }

                    // push remaining info to the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(6.0);

                        // brush size
                        ui.label(
                            egui::RichText::new(format!("Brush: {}", self.brush as i32))
                                .size(FONT_SZ)
                                .color(DIM),
                        );

                        separator(ui);

                        // unsaved indicator
                        if doc.unsaved {
                            ui.label(
                                egui::RichText::new("Unsaved")
                                    .size(FONT_SZ)
                                    .color(ACCENT),
                            );
                        }
                    });
                });
                ui.add_space(2.0);
            });
    }
}
