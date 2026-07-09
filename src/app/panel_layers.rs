use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::color::*;
use crate::constants::*;
use super::PixeshApp;

impl PixeshApp {
    // правая панель: список слоёв + HSV-пикер
    pub(crate) fn ui_layers(&mut self, ctx: &egui::Context) {
        egui::SidePanel::right("layers")
            .resizable(true)
            .default_width(280.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(8.0);

                // ── заголовок "Layers" ──
                let header = "Layers";
                let hdr_w = header.len() as f32 * CHAR_W * 1.5;
                let (hdr, _) = ui.allocate_exact_size(
                    Vec2::new(hdr_w + 12.0, ROW_H * 1.5 + 8.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hdr.min + Vec2::new(PANEL_PAD, 4.0),
                    egui::Align2::LEFT_TOP,
                    header,
                    egui::FontId::proportional(FONT_SZ * 1.5),
                    TEXT,
                );

                ui.add_space(4.0);

                // ── список слоёв (снизу вверх) ──
                let n = self.layers.len();
                for i in (0..n).rev() {
                    let name = self.layers[i].name.clone();
                    let is_active = self.active_layer == i;
                    let cb = self.layers[i].visible;

                    let row_h = ROW_H * 1.5 + 16.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(ui.available_size().x, row_h), Sense::click());

                    let bg = if is_active { HOVER } else { PANEL };
                    ui.painter().rect_filled(rect, 4.0, bg);

                    // чекбокс видимости слоя
                    let cbs = 14.0;
                    let cb_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + PANEL_PAD, rect.center().y - cbs * 0.5),
                        Vec2::splat(cbs),
                    );
                    let p = ui.painter();
                    p.rect_filled(cb_rect, 3.0, PANEL_LIGHT);
                    p.rect_stroke(cb_rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    if cb {
                        let inner = cb_rect.shrink(3.0);
                        p.rect_filled(inner, 2.0, ACCENT);
                    }

                    let cb_resp =
                        ui.interact(cb_rect, egui::Id::new(("lc", i)), Sense::click());
                    if cb_resp.clicked() {
                        self.layers[i].visible = !self.layers[i].visible;
                        self.canvas_dirty = true;
                    }

                    // имя слоя
                    p.text(
                        Pos2::new(cb_rect.max.x + 8.0, rect.min.y + 8.0),
                        egui::Align2::LEFT_TOP,
                        &name,
                        egui::FontId::proportional(FONT_SZ * 1.5),
                        TEXT,
                    );

                    if resp.clicked() && !cb_resp.clicked() {
                        self.active_layer = i;
                    }
                }

                // ── кнопки + / - слой ──
                ui.add_space(PANEL_PAD);
                ui.horizontal(|ui| {
                    let sz = FONT_SZ * 2.5 + 8.0;
                    ui.add_space(PANEL_PAD);

                    // кнопка "+"
                    let (r_plus, resp_plus) = ui.allocate_exact_size(Vec2::splat(sz), Sense::click());
                    let bg = if resp_plus.clicked() { ACCENT } else if resp_plus.hovered() { HOVER } else { PANEL };
                    ui.painter().rect_filled(r_plus, 6.0, bg);
                    ui.painter().rect_stroke(r_plus, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    ui.painter().text(r_plus.center(), egui::Align2::CENTER_CENTER, "+", egui::FontId::proportional(FONT_SZ * 2.5), TEXT);
                    if resp_plus.clicked() { self.add_layer(); }

                    ui.add_space(6.0);

                    // кнопка "-"
                    let (r_minus, resp_minus) = ui.allocate_exact_size(Vec2::splat(sz), Sense::click());
                    let bg = if resp_minus.clicked() { ACCENT } else if resp_minus.hovered() { HOVER } else { PANEL };
                    ui.painter().rect_filled(r_minus, 6.0, bg);
                    ui.painter().rect_stroke(r_minus, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    ui.painter().text(r_minus.center(), egui::Align2::CENTER_CENTER, "-", egui::FontId::proportional(FONT_SZ * 2.5), TEXT);
                    if resp_minus.clicked() { self.remove_layer(self.active_layer); }
                });

                // ── HSV picker ───────────────────────────
                ui.add_space(12.0);
                let hdr = "Color";
                let hw = hdr.len() as f32 * CHAR_W * 1.5;
                let (hr, _) = ui.allocate_exact_size(
                    Vec2::new(hw + 12.0, ROW_H * 1.5 + 8.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hr.min + Vec2::new(PANEL_PAD, 4.0),
                    egui::Align2::LEFT_TOP,
                    hdr,
                    egui::FontId::proportional(FONT_SZ * 1.5),
                    TEXT,
                );

                ui.add_space(4.0);

                // preview + RGB readout
                ui.horizontal(|ui| {
                    ui.add_space(PANEL_PAD);
                    let ps = 72.0;
                    let (pr, _) = ui.allocate_exact_size(Vec2::new(ps, ps), Sense::hover());
                    let pv = pr.translate(Vec2::new(0.0, -4.0));
                    let pc = Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);
                    ui.painter().rect_filled(pv, 4.0, pc);
                    ui.painter().rect_stroke(pv, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);

                    ui.add_space(PANEL_PAD);
                    ui.vertical(|ui| {
                        let mut y = ui.cursor().min.y;
                        for (ch, &v) in [("R", &self.rgb_r), ("G", &self.rgb_g), ("B", &self.rgb_b), ("A", &self.rgb_a)] {
                            let txt = format!("{} {}", ch, v as u8);
                            ui.painter().text(
                                Pos2::new(pr.max.x + 10.0, y),
                                egui::Align2::LEFT_TOP,
                                &txt,
                                egui::FontId::proportional(FONT_SZ),
                                TEXT,
                            );
                            y += ROW_H + 4.0;
                        }
                        let _ = ui.allocate_exact_size(Vec2::new(80.0, (ROW_H + 4.0) * 4.0), Sense::hover());
                    });
                });

                // ── слайдер альфы ──
                ui.add_space(4.0);
                ui.scope(|ui| {
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(28.0),
                    );
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(28.0));
                    ui.horizontal(|ui| {
                        ui.add_space(PANEL_PAD);
                        ui.add(egui::Label::new("a"));
                        ui.add_sized(
                            Vec2::new(ui.available_width() - 70.0, 48.0),
                            egui::Slider::new(&mut self.rgb_a, 0.0..=255.0).show_value(false),
                        );
                        ui.add_sized(
                            Vec2::new(60.0, 48.0),
                            egui::DragValue::new(&mut self.rgb_a)
                                .range(0..=255)
                                .speed(1.0),
                        );
                    });
                });
                self.color = Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);

                // SV field + H strip
                let avail = ui.available_size();
                let fsize = (avail.x - 24.0).min(avail.y).min(180.0).max(40.0);
                let strip_w = 14.0;
                ui.horizontal(|ui| {

                    ui.add_space(PANEL_PAD);

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

                        // курсор на SV-поле
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
                            self.color = Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                        }
                    }

                    // ── H strip (вертикальный) ──
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

                    // курсор на H-стрипе
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
                            self.color = Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                        }
                    }
                });
            });
    }
}
