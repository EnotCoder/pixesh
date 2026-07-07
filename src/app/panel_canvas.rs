use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;
use super::PixeshApp;

fn click_pixel(resp: &egui::Response, canvas_rect: &Rect, zoom: f32) -> Option<(i32, i32)> {
    let pos = resp.interact_pointer_pos()?;
    if !canvas_rect.contains(pos) { return None; }
    let r = pos - canvas_rect.min;
    Some(((r.x / zoom) as i32, (r.y / zoom) as i32))
}

impl PixeshApp {
    pub(crate) fn ui_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG))
            .show(ctx, |ui| {
                let canvas_size = Vec2::new(
                    self.width as f32 * self.zoom,
                    self.height as f32 * self.zoom,
                );
                let avail = ui.available_size();
                let (area, resp) = ui.allocate_exact_size(avail, Sense::click_and_drag());
                let zoom = self.zoom;

                let max_px = (canvas_size.x + area.width()) * 0.5;
                let max_py = (canvas_size.y + area.height()) * 0.5;
                self.pan = self.pan.clamp(
                    Vec2::new(-max_px, -max_py),
                    Vec2::new(max_px, max_py),
                );

                let canvas_rect = Rect::from_center_size(
                    area.center() + self.pan,
                    canvas_size,
                );

                if ui.is_rect_visible(canvas_rect) {
                    if self.canvas_dirty || self.sel_move_current.is_some() {
                        let mut flat = self.composite_display();

                        if let (Some(buf), Some(origin), Some(current)) =
                            (self.sel_buffer.as_ref(), self.sel_move_origin, self.sel_move_current)
                        {
                            if let Some((x0, y0, _, _)) = self.sel {
                                let dx = current.0 - origin.0;
                                let dy = current.1 - origin.1;
                                let w = self.sel_buf_w as i32;
                                let h = self.sel_buf_h as i32;
                                let nx0 = (x0 + dx).max(0).min(self.width as i32 - w);
                                let ny0 = (y0 + dy).max(0).min(self.height as i32 - h);
                                for yy in 0..h {
                                    for xx in 0..w {
                                        let src = buf[(yy * w + xx) as usize];
                                        if src != Color32::TRANSPARENT {
                                            let idx = ((ny0 + yy) * self.width as i32 + nx0 + xx) as usize;
                                            if idx < flat.len() {
                                                flat[idx] = src;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let img = ColorImage { size: [self.width, self.height], pixels: flat };
                        let tex = self.tex.get_or_insert_with(|| {
                            ui.ctx().load_texture("canvas", img.clone(), egui::TextureOptions::NEAREST)
                        });
                        tex.set(img, egui::TextureOptions::NEAREST);
                        if self.sel_move_current.is_none() {
                            self.canvas_dirty = false;
                        }
                    }

                    let p = ui.painter();
                    if let Some(tex) = &self.tex {
                        p.image(
                            tex.id(),
                            canvas_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }

                    let sel_rect = self.sel.or_else(|| {
                        if let (Some(s), Some(e)) = (self.sel_start, self.sel_end) {
                            Some((
                                s.0.min(e.0).max(0),
                                s.1.min(e.1).max(0),
                                s.0.max(e.0).min(self.width as i32 - 1),
                                s.1.max(e.1).min(self.height as i32 - 1),
                            ))
                        } else {
                            None
                        }
                    });
                    if let Some((x0, y0, x1, y1)) = sel_rect {
                        let r = Rect::from_min_size(
                            Pos2::new(canvas_rect.min.x + x0 as f32 * zoom, canvas_rect.min.y + y0 as f32 * zoom),
                            Vec2::new((x1 - x0 + 1) as f32 * zoom, (y1 - y0 + 1) as f32 * zoom),
                        );
                        let sel_color = Color32::from_rgb(255, 255, 255);
                        let steps = 4;
                        let phase = ctx.input(|i| i.time) as f32 * 3.0;
                        for i in 0..steps {
                            let t = i as f32 / steps as f32;
                            let t2 = (i + 1) as f32 / steps as f32;
                            if (i as f32 + phase) % 2.0 < 1.0 { continue; }
                            p.line_segment([
                                Pos2::new(r.min.x + r.width() * t, r.min.y),
                                Pos2::new(r.min.x + r.width() * t2, r.min.y),
                            ], Stroke::new(1.5, sel_color));
                            p.line_segment([
                                Pos2::new(r.min.x + r.width() * t, r.max.y),
                                Pos2::new(r.min.x + r.width() * t2, r.max.y),
                            ], Stroke::new(1.5, sel_color));
                            p.line_segment([
                                Pos2::new(r.min.x, r.min.y + r.height() * t),
                                Pos2::new(r.min.x, r.min.y + r.height() * t2),
                            ], Stroke::new(1.5, sel_color));
                            p.line_segment([
                                Pos2::new(r.max.x, r.min.y + r.height() * t),
                                Pos2::new(r.max.x, r.min.y + r.height() * t2),
                            ], Stroke::new(1.5, sel_color));
                        }
                    }

                    if self.grid {
                        let gc = Color32::from_black_alpha(40);
                        for x in 0..=self.width {
                            p.vline(
                                canvas_rect.min.x + x as f32 * zoom,
                                canvas_rect.y_range(),
                                Stroke::new(1.0, gc),
                            );
                        }
                        for y in 0..=self.height {
                            p.hline(
                                canvas_rect.x_range(),
                                canvas_rect.min.y + y as f32 * zoom,
                                Stroke::new(1.0, gc),
                            );
                        }
                    }

                    let cursor = resp.interact_pointer_pos()
                        .or_else(|| resp.hover_pos());
                    if let Some(pos) = cursor {
                        if canvas_rect.contains(pos) {
                            let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                            let b = self.brush_i() as i32;
                            let half = (b - 1) / 2;
                            let bx0 = (px - half).max(0) as f32;
                            let by0 = (py - half).max(0) as f32;
                            let bx1 = (px - half + b).min(self.width as i32) as f32;
                            let by1 = (py - half + b).min(self.height as i32) as f32;
                            let cr = Rect::from_min_size(
                                Pos2::new(canvas_rect.min.x + bx0 * zoom, canvas_rect.min.y + by0 * zoom),
                                Vec2::new((bx1 - bx0) * zoom, (by1 - by0) * zoom),
                            );
                            p.rect_filled(cr, 0.0, Color32::from_black_alpha(60));
                            p.rect_stroke(cr, 0.0, Stroke::new(1.0, Color32::WHITE.linear_multiply(0.4)), egui::StrokeKind::Inside);
                        }
                    }
                }

                // ── Tool dispatch ─────────────────────
                if !ctx.input(|i| i.pointer.primary_down()) {
                    self.last_px_primary = None;
                }
                if !ctx.input(|i| i.pointer.secondary_down()) {
                    self.last_px_secondary = None;
                }

                let cp = |r: &egui::Response| click_pixel(r, &canvas_rect, zoom);

                match self.tool {
                    Tool::Eyedropper => {
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.handle_eyedropper(p.0, p.1);
                            }
                        }
                    }
                    Tool::Fill => {
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.handle_fill(p.0, p.1);
                            }
                        }
                    }
                    Tool::Select => {
                        if resp.drag_started() {
                            if let Some(p) = cp(&resp) {
                                self.handle_select_press(p.0, p.1);
                            }
                        }
                        if resp.dragged_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.handle_select_drag(p.0, p.1);
                            }
                        }
                        if resp.drag_stopped() {
                            self.handle_select_release();
                        }
                    }
                    _ => {
                        let paint_color = if self.tool == Tool::Eraser { Color32::TRANSPARENT } else { self.color };
                        if self.last_px_primary.is_none() {
                            if ctx.input(|i| i.pointer.primary_down()) {
                                if let Some(p) = cp(&resp) {
                                    self.handle_brush_press(p.0, p.1, paint_color);
                                }
                            }
                        }
                        if resp.dragged_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.handle_brush_drag(p.0, p.1, paint_color);
                            } else {
                                self.last_px_primary = None;
                            }
                        }
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.handle_brush_click(p.0, p.1, paint_color);
                            }
                        }
                    }
                }

                if resp.dragged_by(egui::PointerButton::Secondary) {
                    if let Some(p) = cp(&resp) {
                        if self.last_px_secondary.is_none() { self.push_undo(); }
                        if let Some(last) = self.last_px_secondary {
                            self.paint_line(last.0, last.1, p.0, p.1, Color32::TRANSPARENT);
                        } else {
                            self.paint_pixel(p.0, p.1, Color32::TRANSPARENT);
                        }
                        self.last_px_secondary = Some(p);
                    } else {
                        self.last_px_secondary = None;
                    }
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(p) = cp(&resp) {
                        self.push_undo();
                        self.paint_pixel(p.0, p.1, Color32::TRANSPARENT);
                    }
                }

                if resp.drag_stopped() {
                    self.last_px_primary = None;
                    self.last_px_secondary = None;
                }
            });
    }
}
