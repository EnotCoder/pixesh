use eframe::egui::{self, Pos2, Rect, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;

use super::fit::fitted_dialog;

/// Dialog rows in the same order as `PixeshApp::panels_dialog_sel`.
const ROWS: [(PanelId, &str, &str); 4] = [
    (PanelId::Toolbar, "Toolbar", "top"),
    (PanelId::Layers, "Layers", "right"),
    (PanelId::Status, "Status Bar", "status"),
    (PanelId::Timeline, "Timeline", "timeline"),
];

impl PixeshApp {
    pub(crate) fn ui_panels_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_panels { return; }
        egui::Area::new("panels_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(250.0, 320.0);
                fitted_dialog(ui, "panels_scroll", size, |ui, rect| {
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
                    let cbs = 20.0;
                    let sz = 30.0;
                    let row_h = sz + 12.0;
                    let ctx = ui.ctx();

                    // Arrows move the selection, Enter toggles the selected
                    // panel, Escape closes the dialog (same as the OK button).
                    let mut sel = self.panels_dialog_sel.min(ROWS.len() - 1);
                    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown))
                        || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowRight))
                    {
                        sel = (sel + 1) % ROWS.len();
                    }
                    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp))
                        || ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::ArrowLeft))
                    {
                        sel = (sel + ROWS.len() - 1) % ROWS.len();
                    }
                    let toggle = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                    let close = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));

                    for (idx, (panel, label, id_salt)) in ROWS.iter().copied().enumerate() {
                        let selected = idx == sel;
                        let (row_rect, _) = ui.allocate_exact_size(Vec2::new(160.0, row_h), egui::Sense::click());
                        let p = ui.painter();
                        if selected {
                            p.rect_filled(row_rect, 0.0, HOVER);
                            p.rect_stroke(
                                row_rect,
                                0.0,
                                Stroke::new(2.0_f32, ACCENT),
                                egui::StrokeKind::Inside,
                            );
                        }
                        p.text(
                            Pos2::new(row_rect.min.x + 4.0, row_rect.center().y),
                            egui::Align2::LEFT_CENTER,
                            label,
                            egui::FontId::proportional(sz),
                            TEXT,
                        );
                        let cb_rect = Rect::from_min_size(
                            Pos2::new(row_rect.max.x - cbs - 4.0, row_rect.center().y - cbs * 0.5),
                            Vec2::splat(cbs),
                        );
                        p.rect_filled(cb_rect, 0.0, PANEL);
                        p.rect_stroke(cb_rect, 0.0, egui::Stroke::new(4.0_f32, BORDER), egui::StrokeKind::Outside);
                        if self.panel_visible(panel) {
                            let inner = cb_rect.shrink(4.0);
                            p.rect_filled(inner, 0.0, ACCENT);
                        }
                        let clicked = ui.interact(row_rect, egui::Id::new(("row", id_salt)), egui::Sense::click()).clicked()
                            || ui.interact(cb_rect, egui::Id::new(("cb", id_salt)), egui::Sense::click()).clicked();
                        if clicked {
                            sel = idx;
                            self.toggle_panel(panel);
                        }

                        if idx + 1 < ROWS.len() {
                            let line = ui.allocate_exact_size(Vec2::new(160.0, 1.0), egui::Sense::hover()).0;
                            ui.painter().hline(line.x_range(), line.center().y, egui::Stroke::new(2.0_f32, BORDER));
                        }
                    }

                    if toggle {
                        self.toggle_panel(ROWS[sel].0);
                    }
                    if close {
                        self.show_panels = false;
                    }
                    self.panels_dialog_sel = sel;
                });

                // Leave room for the hint and the OK button below the rows.
                child_ui.add_space((child_ui.available_height() - 66.0).max(0.0));
                child_ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("↑ ↓ select · Enter toggle · Esc close")
                            .size(14.0)
                            .color(DIM),
                    );
                });
                child_ui.add_space(4.0);
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
                p.rect_stroke(btn_rect, 0.0, egui::Stroke::new(4.0_f32, BORDER), egui::StrokeKind::Outside);
                p.text(btn_rect.center(), egui::Align2::CENTER_CENTER, "OK", egui::FontId::proportional(FONT_SZ * 1.5), TEXT);
                if btn_resp.clicked() {
                    self.show_panels = false;
                }
                });
            });
    }
}
