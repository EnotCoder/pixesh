use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::color::*;
use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

impl PixeshApp {
    pub(crate) fn ui_layers(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("layers")
            .resizable(true)
            .default_width(280.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(4.0);

                let header = "Layers";
                let hdr_w = header.len() as f32 * CHAR_W;
                let (hdr, _) = ui.allocate_exact_size(
                    Vec2::new(hdr_w + 8.0, ROW_H + 4.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hdr.min + Vec2::new(4.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    header,
                    egui::FontId::proportional(FONT_SZ),
                    TEXT,
                );

                let n = self.layers.len();
                for i in (0..n).rev() {
                    let name = self.layers[i].name.clone();
                    let is_active = self.active_layer == i;
                    let cb = self.layers[i].visible;

                    let row_h = ROW_H + 6.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(ui.available_size().x, row_h), Sense::click());

                    let bg = if is_active { HOVER } else { PANEL };
                    ui.painter().rect_filled(rect, 0.0, bg);

                    let cbs = 12.0;
                    let cb_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 4.0, rect.center().y - cbs * 0.5),
                        Vec2::splat(cbs),
                    );
                    let p = ui.painter();
                    p.rect_filled(cb_rect, 0.0, PANEL_LIGHT);
                    p.rect_stroke(cb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);
                    if cb {
                        let inner = cb_rect.shrink(2.0);
                        p.rect_filled(inner, 0.0, ACCENT);
                    }

                    let cb_resp =
                        ui.interact(cb_rect, egui::Id::new(("lc", i)), Sense::click());
                    if cb_resp.clicked() {
                        self.layers[i].visible = !self.layers[i].visible;
                    }

                    p.text(
                        Pos2::new(cb_rect.max.x + 4.0, rect.min.y + 3.0),
                        egui::Align2::LEFT_TOP,
                        &name,
                        egui::FontId::proportional(FONT_SZ),
                        TEXT,
                    );

                    if resp.clicked() && !cb_resp.clicked() {
                        self.active_layer = i;
                    }
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    if btn(ui, "+") {
                        self.add_layer();
                    }
                    if btn(ui, "-") {
                        self.remove_layer(self.active_layer);
                    }
                });

                // ── HSV picker ───────────────────────────
                ui.add_space(8.0);
                let hdr = "Color";
                let hw = hdr.len() as f32 * CHAR_W;
                let (hr, _) = ui.allocate_exact_size(
                    Vec2::new(hw + 8.0, ROW_H + 4.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hr.min + Vec2::new(4.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    hdr,
                    egui::FontId::proportional(FONT_SZ),
                    TEXT,
                );

                // preview + RGB readout
                ui.horizontal(|ui| {
                    let ps = 36.0;
                    let (pr, _) = ui.allocate_exact_size(Vec2::new(ps, ps), Sense::hover());
                    let pc = Color32::from_rgb(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8);
                    ui.painter().rect_filled(pr, 0.0, pc);
                    ui.painter().rect_stroke(pr, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                    ui.vertical(|ui| {
                        let mut y = ui.cursor().min.y;
                        for (ch, &v) in [("R", &self.rgb_r), ("G", &self.rgb_g), ("B", &self.rgb_b)] {
                            let txt = format!("{} {}", ch, v as u8);
                            ui.painter().text(
                                Pos2::new(pr.max.x + 6.0, y),
                                egui::Align2::LEFT_TOP,
                                &txt,
                                egui::FontId::proportional(FONT_SZ),
                                TEXT,
                            );
                            y += ROW_H + 2.0;
                        }
                        let _ = ui.allocate_exact_size(Vec2::new(80.0, (ROW_H + 2.0) * 3.0), Sense::hover());
                    });
                });

                // SV field + H strip
                let avail = ui.available_size();
                let fsize = (avail.x - 24.0).min(avail.y).min(180.0).max(40.0);
                let strip_w = 14.0;
                ui.horizontal(|ui| {
                    // ── SV 2D field ──
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(fsize), Sense::click_and_drag());

                    if self.sv_tex.is_none() || (self.sv_tex_h - self.hsv_h).abs() > 0.5 {
                        self.sv_tex_h = self.hsv_h;
                        let ts = 128;
                        let h = self.hsv_h;
                        let mut pix = Vec::with_capacity(ts * ts);
                        for y in 0..ts {
                            for x in 0..ts {
                                let s = x as f32 / (ts - 1) as f32 * 255.0;
                                let v = y as f32 / (ts - 1) as f32 * 255.0;
                                let (r, g, b) = hsv_to_rgb(h, s, v);
                                pix.push(Color32::from_rgb(r, g, b));
                            }
                        }
                        let img = ColorImage { size: [ts, ts], pixels: pix };
                        self.sv_tex = Some(ui.ctx().load_texture("sv", img, egui::TextureOptions::LINEAR));
                    }

                    if let Some(tex) = &self.sv_tex {
                        let p = ui.painter();
                        p.image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                        p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                        let cx = rect.min.x + (self.hsv_s / 255.0) * rect.width();
                        let cy = rect.min.y + (self.hsv_v / 255.0) * rect.height();
                        let cc = if self.hsv_v > 180.0 { Color32::BLACK } else { Color32::WHITE };
                        p.circle_stroke(Pos2::new(cx, cy), 4.0, Stroke::new(1.5, cc));
                        p.circle_filled(Pos2::new(cx, cy), 2.0, cc);
                    }

                    let pick = resp.dragged_by(egui::PointerButton::Primary)
                        || resp.clicked_by(egui::PointerButton::Primary);
                    if pick {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            let rel = pos - rect.min;
                            self.hsv_s = (rel.x / rect.width() * 255.0).clamp(0.0, 255.0);
                            self.hsv_v = (rel.y / rect.height() * 255.0).clamp(0.0, 255.0);
                            let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                            self.rgb_r = r as f32;
                            self.rgb_g = g as f32;
                            self.rgb_b = b as f32;
                            self.color = Color32::from_rgb(r, g, b);
                        }
                    }

                    // ── H strip ──
                    let (srect, sresp) = ui.allocate_exact_size(Vec2::new(strip_w, fsize), Sense::click_and_drag());

                    let ts = 64;
                    let mut spix = Vec::with_capacity(ts);
                    for y in 0..ts {
                        let hh = y as f32 / (ts - 1) as f32 * 360.0;
                        let (r, g, b) = hsv_to_rgb(hh, 255.0, 255.0);
                        spix.push(Color32::from_rgb(r, g, b));
                    }
                    let simg = ColorImage { size: [1, ts], pixels: spix };
                    let stex = ui.ctx().load_texture("hstrip", simg, egui::TextureOptions::LINEAR);
                    let sp = ui.painter();
                    sp.image(stex.id(), srect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    sp.rect_stroke(srect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                    let hy = srect.min.y + (self.hsv_h / 360.0) * srect.height();
                    sp.hline(srect.x_range(), hy, Stroke::new(2.0, Color32::WHITE));

                    let spick = sresp.dragged_by(egui::PointerButton::Primary)
                        || sresp.clicked_by(egui::PointerButton::Primary);
                    if spick {
                        if let Some(pos) = sresp.interact_pointer_pos() {
                            let rel_y = (pos.y - srect.min.y) / srect.height();
                            self.hsv_h = (rel_y * 360.0).clamp(0.0, 359.99);
                            let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                            self.rgb_r = r as f32;
                            self.rgb_g = g as f32;
                            self.rgb_b = b as f32;
                            self.color = Color32::from_rgb(r, g, b);
                        }
                    }
                });
            });
    }
}
