use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;
use super::PixeshApp;

fn click_pixel(resp: &egui::Response, canvas_rect: &Rect, zoom: f32) -> Option<(i32, i32)> {
    let pos = resp.interact_pointer_pos()?;
    if !canvas_rect.contains(pos) { return None; }
    let r = pos - canvas_rect.min;
    Some(((r.x / zoom).round() as i32, (r.y / zoom).round() as i32))
}

impl PixeshApp {
    pub(crate) fn ui_canvas(&mut self, ctx: &egui::Context) {
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG))
            .show(ctx, |ui| {
                let i = self.active_tab;
                let brush = self.brush;
                let tool = self.tool;
                let color = self.color;
                let dialog_open = self.dialog_open();

                let width = self.docs[i].width;
                let height = self.docs[i].height;

                // zoom to fit after loading
                if self.docs[i].needs_zoom_fit {
                    let avail = ui.available_size();
                    let zoom_x = avail.x / width as f32;
                    let zoom_y = avail.y / height as f32;
                    self.docs[i].zoom = zoom_x.min(zoom_y).clamp(0.1, 60.0);
                    self.docs[i].pan = Vec2::ZERO;
                    self.docs[i].needs_zoom_fit = false;
                }

                let zoom = self.docs[i].zoom;
                let pan = self.docs[i].pan;

                let canvas_size = Vec2::new(width as f32 * zoom, height as f32 * zoom);
                let avail = ui.available_size();
                let (area, resp) = ui.allocate_exact_size(avail, Sense::click_and_drag());

                let max_px = (canvas_size.x + area.width()) * 0.5;
                let max_py = (canvas_size.y + area.height()) * 0.5;
                self.docs[i].pan = pan.clamp(
                    Vec2::new(-max_px, -max_py),
                    Vec2::new(max_px, max_py),
                );

                let canvas_rect = Rect::from_center_size(
                    area.center() + self.docs[i].pan,
                    canvas_size,
                );

                // ── render canvas texture ──
                if ui.is_rect_visible(canvas_rect) {
                    if self.docs[i].canvas_dirty || self.docs[i].sel_move_current.is_some() || self.docs[i].canvas_move_current.is_some() || self.docs[i].transforming {
                        self.docs[i].composite_display();

                        // clone sel_buffer to avoid borrow conflict with display_buf
                        let sel_buf_clone = self.docs[i].sel_buffer.clone();
                        let sel_origin = self.docs[i].sel_move_origin;
                        let sel_current = self.docs[i].sel_move_current;
                        let sel_rect = self.docs[i].sel;
                        let sel_bw = self.docs[i].sel_buf_w;
                        let sel_bh = self.docs[i].sel_buf_h;

                        if let (Some(buf), Some(origin), Some(current)) = (sel_buf_clone, sel_origin, sel_current) {
                            if let Some((x0, y0, x1, y1)) = sel_rect {
                                let w = self.docs[i].width as i32;
                                for yy in y0..=y1 {
                                    for xx in x0..=x1 {
                                        let idx = (yy * w + xx) as usize;
                                        if idx < self.docs[i].display_buf.len() {
                                            let ck_a = Color32::from_gray(200);
                                            let ck_b = Color32::from_gray(180);
                                            self.docs[i].display_buf[idx] = if (xx + yy) % 2 == 0 { ck_a } else { ck_b };
                                        }
                                    }
                                }
                                let dx = current.0 - origin.0;
                                let dy = current.1 - origin.1;
                                let bw = sel_bw as i32;
                                let bh = sel_bh as i32;
                                let nx0 = x0 + dx;
                                let ny0 = y0 + dy;
                                let cw = self.docs[i].width as i32;
                                let ch = self.docs[i].height as i32;
                                for yy in 0..bh {
                                    for xx in 0..bw {
                                        let src = buf[(yy * bw + xx) as usize];
                                        if src == Color32::TRANSPARENT { continue; }
                                        let px = nx0 + xx;
                                        let py = ny0 + yy;
                                        if px >= 0 && px < cw && py >= 0 && py < ch {
                                            self.docs[i].display_buf[(py * cw + px) as usize] = src;
                                        }
                                    }
                                }
                            }
                        }

                        if let Some((origin, current)) =
                            self.docs[i].canvas_move_origin.zip(self.docs[i].canvas_move_current)
                        {
                            let dx = current.0 - origin.0;
                            let dy = current.1 - origin.1;
                            if dx != 0 || dy != 0 {
                                let w = self.docs[i].width;
                                let h = self.docs[i].height;
                                let wi = w as i32;
                                let hi = h as i32;
                                let ck_a = Color32::from_gray(200);
                                let ck_b = Color32::from_gray(180);
                                let active = self.docs[i].active_layer;
                                for yy in 0..h {
                                    for xx in 0..w {
                                        let idx = yy * w + xx;
                                        let mut c = Color32::TRANSPARENT;
                                        for (li, layer) in self.docs[i].layers.iter().enumerate().rev() {
                                            if !layer.visible { continue; }
                                            let (sx, sy) = if li == active {
                                                (xx as i32 - dx, yy as i32 - dy)
                                            } else {
                                                (xx as i32, yy as i32)
                                            };
                                            if sx >= 0 && sx < wi && sy >= 0 && sy < hi {
                                                let p = layer.pixels[sy as usize * w + sx as usize];
                                                if p != Color32::TRANSPARENT { c = p; break; }
                                            }
                                        }
                                        let cb = if (xx + yy) % 2 == 0 { ck_a } else { ck_b };
                                        if c == Color32::TRANSPARENT {
                                            c = cb;
                                        } else if c.a() < 255 {
                                            let a = c.a() as u32;
                                            let ia = 255 - a;
                                            c = Color32::from_rgba_premultiplied(
                                                ((c.r() as u32 * a + cb.r() as u32 * ia) / 255) as u8,
                                                ((c.g() as u32 * a + cb.g() as u32 * ia) / 255) as u8,
                                                ((c.b() as u32 * a + cb.b() as u32 * ia) / 255) as u8,
                                                255,
                                            );
                                        }
                                        self.docs[i].display_buf[idx] = c;
                                    }
                                }
                            }
                        }

                        // transform preview: render scaled sel_buffer
                        if self.docs[i].transforming && self.docs[i].sel_buffer.is_some() {
                            if let (Some(sel_rect), scale) = (self.docs[i].sel, self.docs[i].transform_scale) {
                                let buf_clone = self.docs[i].sel_buffer.clone();
                                let orig_clone = self.docs[i].transform_orig_rect;
                                if let (Some(buf), Some(orig)) = (buf_clone, orig_clone) {
                                    let w = self.docs[i].width as i32;
                                    let cw = w;
                                    let ch = self.docs[i].height as i32;
                                    let (ox0, oy0, ox1, oy1) = orig;
                                    let ow = (ox1 - ox0 + 1) as usize;
                                    let oh = (oy1 - oy0 + 1) as usize;
                                    let nw = (ow as f32 * scale).round() as usize;
                                    let nh = (oh as f32 * scale).round() as usize;
                                    if nw > 0 && nh > 0 {
                                        // draw checkerboard under scaled area
                                        let (x0, y0, _x1, _y1) = sel_rect;
                                        for yy in 0..nh as i32 {
                                            for xx in 0..nw as i32 {
                                                let px = x0 + xx;
                                                let py = y0 + yy;
                                                if px >= 0 && px < cw && py >= 0 && py < ch {
                                                    let idx = (py * cw + px) as usize;
                                                    if idx < self.docs[i].display_buf.len() {
                                                        let ck_a = Color32::from_gray(200);
                                                        let ck_b = Color32::from_gray(180);
                                                        self.docs[i].display_buf[idx] = if (xx + yy) % 2 == 0 { ck_a } else { ck_b };
                                                    }
                                                }
                                            }
                                        }
                                        // draw scaled pixels
                                        for ny in 0..nh {
                                            for nx in 0..nw {
                                                let sx = (nx as f64 / nw as f64 * ow as f64) as usize;
                                                let sy = (ny as f64 / nh as f64 * oh as f64) as usize;
                                                let sx = sx.min(ow - 1);
                                                let sy = sy.min(oh - 1);
                                                let src = buf[sy * ow + sx];
                                                if src == Color32::TRANSPARENT { continue; }
                                                let px = x0 + nx as i32;
                                                let py = y0 + ny as i32;
                                                if px >= 0 && px < cw && py >= 0 && py < ch {
                                                    let idx = (py * cw + px) as usize;
                                                    if idx < self.docs[i].display_buf.len() {
                                                        self.docs[i].display_buf[idx] = src;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let pixels = std::mem::take(&mut self.docs[i].display_buf);
                        let img = ColorImage { size: [self.docs[i].width, self.docs[i].height], pixels };
                        let tex = self.docs[i].tex.get_or_insert_with(|| {
                            ui.ctx().load_texture("canvas", img.clone(), egui::TextureOptions::NEAREST)
                        });
                        tex.set(img, egui::TextureOptions::NEAREST);
                        if self.docs[i].sel_move_current.is_none() && self.docs[i].canvas_move_current.is_none() {
                            self.docs[i].canvas_dirty = false;
                        }
                    }

                    let p = ui.painter();
                    if let Some(tex) = &self.docs[i].tex {
                        p.image(
                            tex.id(),
                            canvas_rect,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }

                    // selection marching ants
                    let sel_rect = self.docs[i].sel.or_else(|| {
                        if let (Some(s), Some(e)) = (self.docs[i].sel_start, self.docs[i].sel_end) {
                            Some((
                                s.0.min(e.0).max(0),
                                s.1.min(e.1).max(0),
                                s.0.max(e.0).min(self.docs[i].width as i32 - 1),
                                s.1.max(e.1).min(self.docs[i].height as i32 - 1),
                            ))
                        } else {
                            None
                        }
                    });
                    let sel_draw_pos = sel_rect.and_then(|r| {
                        self.docs[i].sel_move_origin.zip(self.docs[i].sel_move_current).map(|(o, c)| {
                            let dx = c.0 - o.0;
                            let dy = c.1 - o.1;
                            (r.0 + dx, r.1 + dy, r.2 + dx, r.3 + dy)
                        })
                    });
                    if let Some((x0, y0, x1, y1)) = sel_draw_pos.or(sel_rect) {
                        let r = Rect::from_min_size(
                            Pos2::new(canvas_rect.min.x + x0 as f32 * zoom, canvas_rect.min.y + y0 as f32 * zoom),
                            Vec2::new((x1 - x0 + 1) as f32 * zoom, (y1 - y0 + 1) as f32 * zoom),
                        );
                        let white = Color32::from_rgb(255, 255, 255);
                        let black = Color32::from_rgb(0, 0, 0);
                        let segments = 8;
                        let phase = ctx.input(|i| i.time) as f32 * 3.0;
                        for ii in 0..segments {
                            let t0 = ii as f32 / segments as f32;
                            let t1 = (ii + 1) as f32 / segments as f32;
                            let c = if ((ii as f32 + phase) % 2.0) < 1.0 { white } else { black };
                            let s = Stroke::new(6.0, c);
                            p.line_segment([Pos2::new(r.min.x + r.width() * t0, r.min.y), Pos2::new(r.min.x + r.width() * t1, r.min.y)], s);
                            p.line_segment([Pos2::new(r.min.x + r.width() * t0, r.max.y), Pos2::new(r.min.x + r.width() * t1, r.max.y)], s);
                        }
                        for ii in 0..segments {
                            let t0 = ii as f32 / segments as f32;
                            let t1 = (ii + 1) as f32 / segments as f32;
                            let c = if (((ii as f32 + phase) % 2.0) < 1.0) ^ true { white } else { black };
                            let s = Stroke::new(6.0, c);
                            p.line_segment([Pos2::new(r.min.x, r.min.y + r.height() * t0), Pos2::new(r.min.x, r.min.y + r.height() * t1)], s);
                            p.line_segment([Pos2::new(r.max.x, r.min.y + r.height() * t0), Pos2::new(r.max.x, r.min.y + r.height() * t1)], s);
                        }
                    }

                    // grid
                    if self.docs[i].grid {
                        let gc = Color32::from_black_alpha(40);
                        for x in 0..=self.docs[i].width {
                            p.vline(
                                canvas_rect.min.x + x as f32 * zoom,
                                canvas_rect.y_range(),
                                Stroke::new(2.0, gc),
                            );
                        }
                        for y in 0..=self.docs[i].height {
                            p.hline(
                                canvas_rect.x_range(),
                                canvas_rect.min.y + y as f32 * zoom,
                                Stroke::new(2.0, gc),
                            );
                        }
                    }

                    // brush cursor preview
                    let cursor = resp.interact_pointer_pos()
                        .or_else(|| resp.hover_pos());
                    if let Some(pos) = cursor {
                        if canvas_rect.contains(pos) {
                            let (px, py) = self.docs[i].screen_to_pixel(pos, canvas_rect.min);
                            self.cursor_px = Some((px, py));
                            let b = brush.round() as i32;
                            let half = (b - 1) / 2;
                            let bx0 = (px - half).max(0) as f32;
                            let by0 = (py - half).max(0) as f32;
                            let bx1 = (px - half + b).min(self.docs[i].width as i32) as f32;
                            let by1 = (py - half + b).min(self.docs[i].height as i32) as f32;
                            let cr = Rect::from_min_size(
                                Pos2::new(canvas_rect.min.x + bx0 * zoom, canvas_rect.min.y + by0 * zoom),
                                Vec2::new((bx1 - bx0) * zoom, (by1 - by0) * zoom),
                            );
                            p.rect_filled(cr, 0.0, Color32::from_black_alpha(60));
                            p.rect_stroke(cr, 0.0, Stroke::new(2.0, Color32::WHITE.linear_multiply(0.4)), egui::StrokeKind::Inside);
                        } else {
                            self.cursor_px = None;
                        }
                    } else {
                        self.cursor_px = None;
                    }
                }

                // ── middle mouse pan ──
                if resp.dragged_by(egui::PointerButton::Middle) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if let Some(last) = self.docs[i].mid_pan_pos {
                            self.docs[i].pan += pos - last;
                        }
                        self.docs[i].mid_pan_pos = Some(pos);
                    }
                }
                if resp.drag_started() && resp.dragged_by(egui::PointerButton::Middle) {
                    self.docs[i].mid_pan_pos = resp.interact_pointer_pos();
                }

                // ── tool dispatch ──
                if dialog_open {
                    if resp.drag_stopped() {
                        self.docs[i].last_px_primary = None;
                        self.docs[i].last_px_secondary = None;
                    }
                    return;
                }

                if !ctx.input(|i| i.pointer.primary_down()) {
                    self.docs[i].last_px_primary = None;
                }
                if !ctx.input(|i| i.pointer.secondary_down()) {
                    self.docs[i].last_px_secondary = None;
                }

                let cp = |r: &egui::Response| click_pixel(r, &canvas_rect, zoom);

                match tool {
                    Tool::Transform => {
                        // draw corner handles for transform
                        if let Some(sel_rect) = self.docs[i].sel {
                            let (x0, y0, x1, y1) = sel_rect;
                            let r = Rect::from_min_size(
                                Pos2::new(canvas_rect.min.x + x0 as f32 * zoom, canvas_rect.min.y + y0 as f32 * zoom),
                                Vec2::new((x1 - x0 + 1) as f32 * zoom, (y1 - y0 + 1) as f32 * zoom),
                            );
                            let handle_size = 10.0;
                            let corners = [
                                r.min,
                                Pos2::new(r.max.x, r.min.y),
                                r.max,
                                Pos2::new(r.min.x, r.max.y),
                            ];

                            for &c in &corners {
                                let hr = Rect::from_center_size(c, Vec2::splat(handle_size * 2.0));
                                let hresp = ui.interact(hr, egui::Id::new(("xfm_h", c.x.to_bits(), c.y.to_bits())), Sense::click_and_drag());

                                if hresp.drag_started() {
                                    self.docs[i].transform_corner = Some((c.x as i32, c.y as i32));
                                    self.docs[i].transform_orig_rect = Some(sel_rect);
                                    self.docs[i].transform_scale = 1.0;
                                }
                                if hresp.dragged_by(egui::PointerButton::Primary) {
                                    if let Some(pos) = hresp.interact_pointer_pos() {
                                        let center = r.center();
                                        let orig_len = (c - center).length();
                                        let cur_len = (pos - center).length();
                                        if orig_len > 1.0 {
                                            self.docs[i].transform_scale = (cur_len / orig_len).max(0.1).min(10.0);
                                        }
                                    }
                                    self.docs[i].canvas_dirty = true;
                                }
                                if hresp.drag_stopped() && self.docs[i].transform_corner.is_some() {
                                    if self.docs[i].transforming && self.docs[i].transform_scale != 1.0 {
                                        let scale = self.docs[i].transform_scale;
                                        let (sx0, sy0, sx1, sy1) = self.docs[i].transform_orig_rect.unwrap_or(sel_rect);
                                        let orig_w = (sx1 - sx0 + 1) as usize;
                                        let orig_h = (sy1 - sy0 + 1) as usize;
                                        let new_w = (orig_w as f32 * scale).round() as usize;
                                        let new_h = (orig_h as f32 * scale).round() as usize;
                                        if new_w > 0 && new_h > 0 {
                                            self.docs[i].push_undo();
                                            if let Some(ref buf) = self.docs[i].sel_buffer {
                                                let mut new_buf = vec![Color32::TRANSPARENT; new_w * new_h];
                                                for ny in 0..new_h {
                                                    for nx in 0..new_w {
                                                        let sx = (nx as f64 / new_w as f64 * orig_w as f64) as usize;
                                                        let sy = (ny as f64 / new_h as f64 * orig_h as f64) as usize;
                                                        let sx = sx.min(orig_w - 1);
                                                        let sy = sy.min(orig_h - 1);
                                                        new_buf[ny * new_w + nx] = buf[sy * orig_w + sx];
                                                    }
                                                }
                                                self.docs[i].sel_buffer = Some(new_buf);
                                                self.docs[i].sel_buf_w = new_w;
                                                self.docs[i].sel_buf_h = new_h;
                                            }
                                            let nx0 = sx0;
                                            let ny0 = sy0;
                                            self.docs[i].sel = Some((nx0, ny0, nx0 + new_w as i32 - 1, ny0 + new_h as i32 - 1));
                                        }
                                    }
                                    self.docs[i].transforming = false;
                                    self.docs[i].transform_corner = None;
                                    self.docs[i].transform_orig_rect = None;
                                    self.docs[i].transform_scale = 1.0;
                                    self.docs[i].canvas_dirty = true;
                                }

                                ui.painter().rect_filled(Rect::from_center_size(c, Vec2::splat(handle_size)), 0.0, ACCENT);
                                ui.painter().rect_stroke(Rect::from_center_size(c, Vec2::splat(handle_size)), 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                            }
                        }
                    }
                    Tool::Text => {
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            self.docs[i].commit_pending_paste();
                            if let Some(p) = cp(&resp) {
                                self.text_cursor = Some(p);
                                self.text_buffer.clear();
                                self.show_text = true;
                            }
                        }
                    }
                    Tool::Eyedropper => {
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            self.docs[i].commit_pending_paste();
                            if let Some(p) = cp(&resp) {
                                self.handle_eyedropper(p.0, p.1);
                            }
                        }
                    }
                    Tool::Fill => {
                        if resp.clicked_by(egui::PointerButton::Primary) {
                            self.docs[i].commit_pending_paste();
                            if let Some(p) = cp(&resp) {
                                self.docs[i].handle_fill(p.0, p.1, color);
                            }
                        }
                    }
                    Tool::Select => {
                        if resp.drag_started() || (resp.is_pointer_button_down_on() && self.docs[i].sel_start.is_none()) {
                            if self.docs[i].pasting && !self.docs[i].sel_move_origin.is_some() {
                                self.docs[i].commit_pending_paste();
                            }
                            let press_pos = ctx.input(|i| i.pointer.press_origin());
                            if let Some(pos) = press_pos {
                                if canvas_rect.contains(pos) {
                                    let (px, py) = self.docs[i].screen_to_pixel(pos, canvas_rect.min);
                                    self.docs[i].handle_select_press(px, py);
                                }
                            }
                        }
                        if resp.dragged_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.docs[i].handle_select_drag(p.0, p.1);
                            }
                        }
                        if resp.drag_stopped() {
                            self.docs[i].handle_select_release();
                        }
                    }
                    Tool::Move => {
                        if resp.drag_started() {
                            if self.docs[i].pasting && !self.docs[i].sel_move_origin.is_some() {
                                self.docs[i].commit_pending_paste();
                            }
                            if let Some(p) = cp(&resp) {
                                self.docs[i].handle_move_press(p.0, p.1);
                            }
                        }
                        if resp.dragged_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.docs[i].handle_move_drag(p.0, p.1);
                            }
                        }
                        if resp.drag_stopped() {
                            self.docs[i].handle_move_release();
                        }
                    }
                    _ => {
                        self.docs[i].commit_pending_paste();
                        let paint_color = if tool == Tool::Eraser { Color32::TRANSPARENT } else { color };
                        if self.docs[i].last_px_primary.is_none() {
                            if ctx.input(|i| i.pointer.primary_down()) {
                                if let Some(p) = cp(&resp) {
                                    self.docs[i].handle_brush_press(p.0, p.1, paint_color, brush);
                                }
                            }
                        }
                        if resp.dragged_by(egui::PointerButton::Primary) {
                            if let Some(p) = cp(&resp) {
                                self.docs[i].handle_brush_drag(p.0, p.1, paint_color, brush);
                            } else {
                                self.docs[i].last_px_primary = None;
                            }
                        }
                    }
                }

                // right click eraser
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    self.docs[i].commit_pending_paste();
                    if let Some(p) = cp(&resp) {
                        if self.docs[i].last_px_secondary.is_none() { self.docs[i].push_undo(); }
                        if let Some(last) = self.docs[i].last_px_secondary {
                            self.docs[i].paint_line(last.0, last.1, p.0, p.1, Color32::TRANSPARENT, brush);
                        } else {
                            self.docs[i].paint_pixel(p.0, p.1, Color32::TRANSPARENT, brush);
                        }
                        self.docs[i].last_px_secondary = Some(p);
                    } else {
                        self.docs[i].last_px_secondary = None;
                    }
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    self.docs[i].commit_pending_paste();
                    if let Some(p) = cp(&resp) {
                        self.docs[i].push_undo();
                        self.docs[i].paint_pixel(p.0, p.1, Color32::TRANSPARENT, brush);
                    }
                }

                if resp.drag_stopped() {
                    self.docs[i].last_px_primary = None;
                    self.docs[i].last_px_secondary = None;
                    self.docs[i].mid_pan_pos = None;
                }
            });
    }
}
