use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::color::*;
use crate::constants::*;
use super::PixeshApp;

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
                    if self.canvas_dirty {
                        let flat = self.composite_display();
                        let img = ColorImage {
                            size: [self.width, self.height],
                            pixels: flat,
                        };
                        let tex = self.tex.get_or_insert_with(|| {
                            ui.ctx().load_texture(
                                "canvas",
                                img.clone(),
                                egui::TextureOptions::NEAREST,
                            )
                        });
                        tex.set(img, egui::TextureOptions::NEAREST);
                        self.canvas_dirty = false;
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

                    if self.grid {
                        let gc = Color32::from_black_alpha(40);
                        for x in 0..=self.width {
                            p.vline(
                                canvas_rect.min.x + x as f32 * self.zoom,
                                canvas_rect.y_range(),
                                Stroke::new(1.0, gc),
                            );
                        }
                        for y in 0..=self.height {
                            p.hline(
                                canvas_rect.x_range(),
                                canvas_rect.min.y + y as f32 * self.zoom,
                                Stroke::new(1.0, gc),
                            );
                        }
                    }

                    // brush cursor
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
                                Pos2::new(
                                    canvas_rect.min.x + bx0 * self.zoom,
                                    canvas_rect.min.y + by0 * self.zoom,
                                ),
                                Vec2::new(
                                    (bx1 - bx0) * self.zoom,
                                    (by1 - by0) * self.zoom,
                                ),
                            );
                            p.rect_filled(cr, 0.0, Color32::from_black_alpha(60));
                            p.rect_stroke(cr, 0.0, Stroke::new(1.0, Color32::WHITE.linear_multiply(0.4)), egui::StrokeKind::Inside);
                        }
                    }
                }

                // LMB
                if self.tool == Tool::Eyedropper {
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                                    let c = self.composite()[(py * self.width as i32 + px) as usize];
                                    self.color = c;
                                    self.rgb_r = c.r() as f32;
                                    self.rgb_g = c.g() as f32;
                                    self.rgb_b = c.b() as f32;
                                    let (h, s, v) = rgb_to_hsv(c.r(), c.g(), c.b());
                                    self.hsv_h = h;
                                    self.hsv_s = s;
                                    self.hsv_v = v;
                                }
                            }
                        }
                    }
                } else if self.tool == Tool::Fill {
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                self.push_undo();
                                let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                                self.flood_fill(px, py, self.color);
                            }
                        }
                    }
                } else {
                    let paint_color = if self.tool == Tool::Eraser { Color32::TRANSPARENT } else { self.color };
                    if self.last_px_primary.is_none() {
                        let pressed = ctx.input(|i| i.pointer.primary_down());
                        if pressed {
                            if let Some(pos) = resp.interact_pointer_pos() {
                                if canvas_rect.contains(pos) {
                                    let px = self.screen_to_pixel(pos, canvas_rect.min);
                                    self.push_undo();
                                    self.paint_pixel(px.0, px.1, paint_color);
                                    self.last_px_primary = Some(px);
                                }
                            }
                        }
                    }
                    if resp.dragged_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                let px = self.screen_to_pixel(pos, canvas_rect.min);
                                self.paint_pixel(px.0, px.1, paint_color);
                                self.last_px_primary = Some(px);
                            }
                        }
                    }
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                self.push_undo();
                                let px = self.screen_to_pixel(pos, canvas_rect.min);
                                self.paint_pixel(px.0, px.1, paint_color);
                            }
                        }
                    }
                }

                // RMB always erases (transparent)
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            if self.last_px_secondary.is_none() {
                                self.push_undo();
                            }
                            self.paint_pixel(px.0, px.1, Color32::TRANSPARENT);
                            self.last_px_secondary = Some(px);
                        }
                    }
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            self.push_undo();
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            self.paint_pixel(px.0, px.1, Color32::TRANSPARENT);
                        }
                    }
                }

                if resp.drag_stopped() {
                    self.last_px_primary = None;
                    self.last_px_secondary = None;
                }
            });
    }
}
