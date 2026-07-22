use std::sync::Arc;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

fn load_icon_texture(ui: &egui::Ui, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let img = match image::load_from_memory(bytes) {
        Ok(img) => img.into_rgba8(),
        Err(_) => return {
            let fallback = ColorImage::from_rgba_unmultiplied([1, 1], &[255, 0, 255, 255]);
            ui.ctx().load_texture(name, fallback, egui::TextureOptions::NEAREST)
        },
    };
    let w = img.width() as usize;
    let h = img.height() as usize;
    let raw = img.into_raw();
    let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
    ui.ctx().load_texture(name, ci, egui::TextureOptions::NEAREST)
}

impl PixeshApp {
    pub(crate) fn ui_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tools")
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                // ── main toolbar row ──
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);

                    // logo
                    let logo_tex = self.logo_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "logo", include_bytes!("../../logo.png"))
                    });
                    let logo_sz = Vec2::splat((ROW_H + 6.0) * 1.5);
                    let (lr, _) = ui.allocate_exact_size(logo_sz, Sense::hover());
                    ui.painter().image(logo_tex.id(), lr.translate(Vec2::new(0.0, 4.0)),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

                    separator(ui);
                    ui.add_space(4.0);

                    // tool icons
                    let brush_tex = self.brush_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "brush_icon", include_bytes!("../../tex/brush.png"))
                    });
                    if icon_btn(ui, brush_tex.id(), self.tool == Tool::Brush) { self.tool = Tool::Brush; }

                    let eraser_tex = self.eraser_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "eraser_icon", include_bytes!("../../tex/eraser.png"))
                    });
                    if icon_btn(ui, eraser_tex.id(), self.tool == Tool::Eraser) { self.tool = Tool::Eraser; }

                    let fill_tex = self.fill_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "fill_icon", include_bytes!("../../tex/fill.png"))
                    });
                    if icon_btn(ui, fill_tex.id(), self.tool == Tool::Fill) { self.tool = Tool::Fill; }

                    let drop_tex = self.drop_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "drop_icon", include_bytes!("../../tex/drop.png"))
                    });
                    if icon_btn(ui, drop_tex.id(), self.tool == Tool::Eyedropper) { self.tool = Tool::Eyedropper; }

                    let select_tex = self.select_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "select_icon", include_bytes!("../../tex/select.png"))
                    });
                    if icon_btn(ui, select_tex.id(), self.tool == Tool::Select) { self.tool = Tool::Select; }

                    let move_tex = self.move_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "move_icon", include_bytes!("../../tex/move.png"))
                    });
                    if icon_btn(ui, move_tex.id(), self.tool == Tool::Move) { self.tool = Tool::Move; }

                    // clear
                    separator(ui);

                    let clear_tex = self.clear_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "clear_icon", include_bytes!("../../tex/clear.png"))
                    });
                    if icon_btn(ui, clear_tex.id(), false) {
                        let doc = &mut self.docs[self.active_tab];
                        doc.push_undo();
                        for layer in &mut doc.layers {
                            layer.pixels = Arc::new(vec![Color32::TRANSPARENT; doc.width * doc.height]);
                        }
                        doc.canvas_dirty = true;
                    }

                    // grid + zoom
                    ui.add_space(6.0);
                    separator(ui);
                    ui.add_space(6.0);

                    let cbs = 18.0;
                    let btn_h = ROW_H + 16.0;
                    let grid_w = cbs + 8.0 + "Grid".len() as f32 * CHAR_W;
                    let (grid_rect, grid_resp) = ui.allocate_exact_size(Vec2::new(grid_w, btn_h), Sense::click());
                    let p = ui.painter();
                    let cb_rect = Rect::from_min_size(
                        Pos2::new(grid_rect.min.x, grid_rect.center().y - cbs * 0.5),
                        Vec2::splat(cbs),
                    );
                    p.rect_filled(cb_rect, 0.0, PANEL);
                    p.rect_stroke(cb_rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    if self.docs[self.active_tab].grid {
                        p.rect_filled(cb_rect.shrink(4.0), 0.0, ACCENT);
                    }
                    p.text(
                        Pos2::new(cb_rect.max.x + 8.0, grid_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Grid",
                        egui::FontId::proportional(FONT_SZ),
                        TEXT,
                    );
                    let cb_resp = ui.interact(cb_rect, egui::Id::new("grid_cb"), Sense::click());
                    if cb_resp.clicked() {
                        self.docs[self.active_tab].grid = !self.docs[self.active_tab].grid;
                    }
                    if grid_resp.clicked() && !cb_resp.clicked() {
                        self.docs[self.active_tab].grid = !self.docs[self.active_tab].grid;
                    }
                    ui.add_space(6.0);

                    let mh_tex = self.mirror_h_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "mirror_h", include_bytes!("../../tex/mirror_h.png"))
                    });
                    if icon_btn(ui, mh_tex.id(), false) {
                        let doc = &mut self.docs[self.active_tab];
                        doc.push_undo();
                        doc.mirror_horizontal();
                    }

                    let mv_tex = self.mirror_v_tex.get_or_insert_with(|| {
                        load_icon_texture(ui, "mirror_v", include_bytes!("../../tex/mirror_v.png"))
                    });
                    if icon_btn(ui, mv_tex.id(), false) {
                        let doc = &mut self.docs[self.active_tab];
                        doc.push_undo();
                        doc.mirror_vertical();
                    }

                    ui.add_space(6.0);

                    // zoom text
                    let zoom_text = format!("Zoom: {:.2}", self.docs[self.active_tab].zoom);
                    let zoom_w = zoom_text.len() as f32 * CHAR_W * (20.0 / FONT_SZ) + 10.0;
                    let (zoom_rect, _) = ui.allocate_exact_size(Vec2::new(zoom_w, btn_h), Sense::hover());
                    ui.painter().text(
                        Pos2::new(zoom_rect.min.x, zoom_rect.center().y),
                        egui::Align2::LEFT_CENTER,
                        &zoom_text,
                        egui::FontId::proportional(20.0),
                        TEXT,
                    );
                    ui.add_space(6.0);
                });
                ui.add_space(6.0);

                // ── tab bar row ──
                if self.docs.len() > 0 {
                    ui.add_space(2.0);
                    ui.horizontal(|ui| {
                        ui.add_space(6.0);
                        let tab_h = 28.0;

                        for ti in 0..self.docs.len() {
                            let is_active = self.docs[ti].name == self.docs[ti].name && ti == self.active_tab;
                            let unsaved = self.docs[ti].unsaved;
                            let name = &self.docs[ti].name;
                            let label = if unsaved { format!("*{}", name) } else { name.clone() };
                            let label_w = label.len() as f32 * CHAR_W * (18.0 / FONT_SZ) + 24.0;
                            let close_w = 22.0;
                            let tab_w = label_w + close_w;

                            let (tab_rect, tab_resp) = ui.allocate_exact_size(Vec2::new(tab_w, tab_h), Sense::click());

                            let bg = if is_active { HOVER } else { PANEL };
                            let p = ui.painter();
                            p.rect_filled(tab_rect, 0.0, bg);
                            p.rect_stroke(tab_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Inside);

                            // tab label
                            p.text(
                                Pos2::new(tab_rect.min.x + 8.0, tab_rect.center().y),
                                egui::Align2::LEFT_CENTER,
                                &label,
                                egui::FontId::proportional(18.0),
                                TEXT,
                            );

                            // close button
                            let close_rect = Rect::from_min_size(
                                Pos2::new(tab_rect.max.x - close_w, tab_rect.min.y),
                                Vec2::new(close_w, tab_h),
                            );
                            let close_resp = ui.interact(close_rect, egui::Id::new(("tab_close", ti)), Sense::click());
                            let close_bg = if close_resp.hovered() { ACCENT } else { PANEL };
                            p.rect_filled(close_rect, 0.0, close_bg);
                            p.text(
                                close_rect.center(),
                                egui::Align2::CENTER_CENTER,
                                "x",
                                egui::FontId::proportional(16.0),
                                TEXT,
                            );

                            if close_resp.clicked() {
                                self.close_tab(ti);
                                break;
                            }
                            if tab_resp.clicked() && !close_resp.clicked() {
                                self.active_tab = ti;
                            }
                        }

                        // "+" button to create new tab
                        let plus_w = 28.0;
                        let (plus_rect, plus_resp) = ui.allocate_exact_size(Vec2::new(plus_w, tab_h), Sense::click());
                        let bg = if plus_resp.hovered() { HOVER } else { PANEL };
                        ui.painter().rect_filled(plus_rect, 0.0, bg);
                        ui.painter().rect_stroke(plus_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Inside);
                        ui.painter().text(
                            plus_rect.center(),
                            egui::Align2::CENTER_CENTER,
                            "+",
                            egui::FontId::proportional(20.0),
                            TEXT,
                        );
                        if plus_resp.clicked() {
                            let n = self.docs.len();
                            self.docs.push(super::Document::new(&format!("Untitled {}", n)));
                            self.active_tab = self.docs.len() - 1;
                        }

                        ui.add_space(6.0);
                    });
                    ui.add_space(4.0);
                }
            });
    }
}
