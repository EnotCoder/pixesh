use eframe::egui::Color32;

use super::Document;

impl Document {
    pub(crate) fn handle_brush_press(&mut self, px: i32, py: i32, color: Color32, brush: f32) {
        self.push_undo();
        self.paint_pixel(px, py, color, brush);
        self.last_px_primary = Some((px, py));
    }

    pub(crate) fn handle_brush_drag(&mut self, px: i32, py: i32, color: Color32, brush: f32) {
        if let Some(last) = self.last_px_primary {
            self.paint_line(last.0, last.1, px, py, color, brush);
        } else {
            self.paint_pixel(px, py, color, brush);
        }
        self.last_px_primary = Some((px, py));
    }

    pub(crate) fn handle_fill(&mut self, px: i32, py: i32, color: Color32) {
        self.push_undo();
        self.flood_fill(px, py, color);
    }

    pub(crate) fn handle_move_press(&mut self, px: i32, py: i32) {
        if let Some((x0, y0, x1, y1)) = self.sel {
            if px >= x0 && px <= x1 && py >= y0 && py <= y1 {
                if self.pasting {
                    self.sel_move_origin = Some((px, py));
                    self.sel_move_current = None;
                    return;
                }
                self.push_undo();
                let sw = (x1 - x0 + 1) as usize;
                let sh = (y1 - y0 + 1) as usize;
                let w = self.width;
                let mut buf = Vec::with_capacity(sw * sh);
                for yy in y0..=y1 {
                    for xx in x0..=x1 {
                        let idx = (yy * w as i32 + xx) as usize;
                        buf.push(self.layers[self.active_layer].pixels[idx]);
                    }
                }
                self.sel_buffer = Some(buf);
                self.sel_buf_w = sw;
                self.sel_buf_h = sh;
                self.sel_move_origin = Some((px, py));
                self.sel_move_current = None;
                return;
            }
        }
        self.canvas_move_origin = Some((px, py));
        self.canvas_move_current = None;
    }

    pub(crate) fn handle_move_drag(&mut self, px: i32, py: i32) {
        if self.sel_move_origin.is_some() {
            self.sel_move_current = Some((px, py));
            return;
        }
        if self.canvas_move_origin.is_some() {
            self.canvas_move_current = Some((px, py));
        }
    }

    pub(crate) fn handle_move_release(&mut self) {
        if self.sel_move_origin.is_some() {
            let was_pasting = self.pasting;
            let origin = self.sel_move_origin;
            let current = self.sel_move_current;
            let sel = self.sel;
            let has_move = current.is_some() && origin.is_some() && sel.is_some();

            if has_move {
                let (cx, cy) = current.unwrap();
                let (ox, oy) = origin.unwrap();
                let (x0, y0, x1, y1) = sel.unwrap();
                let w = self.sel_buf_w as i32;
                let h = self.sel_buf_h as i32;
                let dx = cx - ox;
                let dy = cy - oy;
                let nx0 = x0 + dx;
                let ny0 = y0 + dy;
                let cw = self.width as i32;
                let ch = self.height as i32;

                if let Some(buf) = self.sel_buffer.take() {
                    let pixels = self.pixels_mut(self.active_layer);
                    if !was_pasting {
                        for yy in y0..=y1 {
                            for xx in x0..=x1 {
                                pixels[(yy * cw + xx) as usize] = Color32::TRANSPARENT;
                            }
                        }
                    }
                    for yy in 0..h {
                        for xx in 0..w {
                            let src = buf[(yy * w + xx) as usize];
                            if src == Color32::TRANSPARENT { continue; }
                            let px = nx0 + xx;
                            let py = ny0 + yy;
                            if px >= 0 && px < cw && py >= 0 && py < ch {
                                pixels[(py * cw + px) as usize] = src;
                            }
                        }
                    }
                }
                let cl = nx0.max(0);
                let ct = ny0.max(0);
                let cr = (nx0 + w - 1).min(cw - 1);
                let cb = (ny0 + h - 1).min(ch - 1);
                self.sel = if cl <= cr && ct <= cb { Some((cl, ct, cr, cb)) } else { None };
                self.canvas_dirty = true;
            } else if was_pasting {
                // paste without move: commit buffer at current sel position
                if let (Some(buf), Some((x0, y0, _x1, _y1))) = (self.sel_buffer.take(), sel) {
                    let w = self.sel_buf_w as i32;
                    let h = self.sel_buf_h as i32;
                    let cw = self.width as i32;
                    let ch = self.height as i32;
                    let pixels = self.pixels_mut(self.active_layer);
                    for yy in 0..h {
                        for xx in 0..w {
                            let src = buf[(yy * w + xx) as usize];
                            if src == Color32::TRANSPARENT { continue; }
                            let px = x0 + xx;
                            let py = y0 + yy;
                            if px >= 0 && px < cw && py >= 0 && py < ch {
                                pixels[(py * cw + px) as usize] = src;
                            }
                        }
                    }
                    self.canvas_dirty = true;
                }
            }
            self.clear_move_state();
            self.canvas_move_origin = None;
            self.canvas_move_current = None;
            if was_pasting {
                self.sel = None;
                self.pasting = false;
            }
            return;
        }

        if let (Some(origin), Some(current)) = (self.canvas_move_origin, self.canvas_move_current) {
            let dx = current.0 - origin.0;
            let dy = current.1 - origin.1;
            if dx == 0 && dy == 0 {
                self.canvas_move_origin = None;
                self.canvas_move_current = None;
                return;
            }
            self.push_undo();
            let w = self.width as i32;
            let h = self.height as i32;
            for layer_idx in 0..self.layers.len() {
                let pixels = self.pixels_mut(layer_idx);
                let mut new_pixels = vec![Color32::TRANSPARENT; (w * h) as usize];
                for yy in 0..h {
                    for xx in 0..w {
                        let src = pixels[(yy * w + xx) as usize];
                        if src == Color32::TRANSPARENT { continue; }
                        let nx = xx + dx;
                        let ny = yy + dy;
                        if nx >= 0 && nx < w && ny >= 0 && ny < h {
                            new_pixels[(ny * w + nx) as usize] = src;
                        }
                    }
                }
                *pixels = new_pixels;
            }
            self.canvas_dirty = true;
        }
        self.canvas_move_origin = None;
        self.canvas_move_current = None;
    }

    pub(crate) fn handle_select_press(&mut self, px: i32, py: i32) {
        if let Some((x0, y0, x1, y1)) = self.sel {
            if px >= x0 && px <= x1 && py >= y0 && py <= y1 {
                if self.pasting {
                    self.sel_move_origin = Some((px, py));
                    self.sel_move_current = None;
                    return;
                }
                self.push_undo();
                let sw = (x1 - x0 + 1) as usize;
                let sh = (y1 - y0 + 1) as usize;
                let w = self.width;
                let mut buf = Vec::with_capacity(sw * sh);
                for yy in y0..=y1 {
                    for xx in x0..=x1 {
                        let idx = (yy * w as i32 + xx) as usize;
                        buf.push(self.layers[self.active_layer].pixels[idx]);
                    }
                }
                self.sel_buffer = Some(buf);
                self.sel_buf_w = sw;
                self.sel_buf_h = sh;
                self.sel_move_origin = Some((px, py));
                self.sel_move_current = None;
                return;
            }
        }
        self.sel = None;
        self.sel_start = Some((px, py));
        self.sel_end = Some((px, py));
        self.clear_move_state();
    }

    pub(crate) fn handle_select_drag(&mut self, px: i32, py: i32) {
        if self.sel_move_origin.is_some() {
            self.sel_move_current = Some((px, py));
            return;
        }
        if self.sel_start.is_some() {
            self.sel_end = Some((px, py));
        }
    }

    pub(crate) fn handle_select_release(&mut self) {
        if self.sel_move_origin.is_some() {
            let was_pasting = self.pasting;
            let origin = self.sel_move_origin;
            let current = self.sel_move_current;
            let sel = self.sel;
            let has_move = current.is_some() && origin.is_some() && sel.is_some();

            if has_move {
                let (cx, cy) = current.unwrap();
                let (ox, oy) = origin.unwrap();
                let (x0, y0, x1, y1) = sel.unwrap();
                let w = self.sel_buf_w as i32;
                let h = self.sel_buf_h as i32;
                let dx = cx - ox;
                let dy = cy - oy;
                let nx0 = x0 + dx;
                let ny0 = y0 + dy;
                let cw = self.width as i32;
                let ch = self.height as i32;

                if let Some(buf) = self.sel_buffer.take() {
                    let pixels = self.pixels_mut(self.active_layer);
                    if !was_pasting {
                        for yy in y0..=y1 {
                            for xx in x0..=x1 {
                                pixels[(yy * cw + xx) as usize] = Color32::TRANSPARENT;
                            }
                        }
                    }
                    for yy in 0..h {
                        for xx in 0..w {
                            let src = buf[(yy * w + xx) as usize];
                            if src == Color32::TRANSPARENT { continue; }
                            let px = nx0 + xx;
                            let py = ny0 + yy;
                            if px >= 0 && px < cw && py >= 0 && py < ch {
                                pixels[(py * cw + px) as usize] = src;
                            }
                        }
                    }
                }
                let cl = nx0.max(0);
                let ct = ny0.max(0);
                let cr = (nx0 + w - 1).min(cw - 1);
                let cb = (ny0 + h - 1).min(ch - 1);
                self.sel = if cl <= cr && ct <= cb { Some((cl, ct, cr, cb)) } else { None };
                self.canvas_dirty = true;
            } else if was_pasting {
                // paste without move: commit buffer at current sel position
                if let (Some(buf), Some((x0, y0, _x1, _y1))) = (self.sel_buffer.take(), sel) {
                    let w = self.sel_buf_w as i32;
                    let h = self.sel_buf_h as i32;
                    let cw = self.width as i32;
                    let ch = self.height as i32;
                    let pixels = self.pixels_mut(self.active_layer);
                    for yy in 0..h {
                        for xx in 0..w {
                            let src = buf[(yy * w + xx) as usize];
                            if src == Color32::TRANSPARENT { continue; }
                            let px = x0 + xx;
                            let py = y0 + yy;
                            if px >= 0 && px < cw && py >= 0 && py < ch {
                                pixels[(py * cw + px) as usize] = src;
                            }
                        }
                    }
                    self.canvas_dirty = true;
                }
            }
            self.clear_move_state();
            if was_pasting {
                self.sel = None;
                self.pasting = false;
            }
            return;
        }

        if let (Some(start), Some(end)) = (self.sel_start, self.sel_end) {
            let mw = self.width as i32 - 1;
            let mh = self.height as i32 - 1;
            let x0 = start.0.min(end.0).max(0).min(mw);
            let y0 = start.1.min(end.1).max(0).min(mh);
            let x1 = start.0.max(end.0).max(0).min(mw);
            let y1 = start.1.max(end.1).max(0).min(mh);
            if x0 != x1 || y0 != y1 {
                self.sel = Some((x0, y0, x1, y1));
            } else {
                self.sel = None;
            }
        }
        self.sel_start = None;
        self.sel_end = None;
    }

    pub(crate) fn clear_move_state(&mut self) {
        self.sel_move_origin = None;
        self.sel_move_current = None;
        self.sel_buffer = None;
        self.sel_tex = None;
    }

    pub(crate) fn delete_selection(&mut self) {
        if let Some((x0, y0, x1, y1)) = self.sel {
            self.push_undo();
            let w = self.width as i32;
            let pixels = self.pixels_mut(self.active_layer);
            for y in y0..=y1 {
                for x in x0..=x1 {
                    pixels[(y * w + x) as usize] = Color32::TRANSPARENT;
                }
            }
            self.canvas_dirty = true;
            self.sel = None;
            self.clear_move_state();
        }
    }

    pub(crate) fn deselect(&mut self) {
        self.sel = None;
        self.sel_start = None;
        self.sel_end = None;
        self.clear_move_state();
    }

    pub(crate) fn commit_pending_paste(&mut self) {
        if !self.pasting { return; }
        if let (Some(buf), Some((x0, y0, _x1, _y1))) = (self.sel_buffer.take(), self.sel) {
            let w = self.sel_buf_w as i32;
            let h = self.sel_buf_h as i32;
            let cw = self.width as i32;
            let ch = self.height as i32;
            let pixels = self.pixels_mut(self.active_layer);
            for yy in 0..h {
                for xx in 0..w {
                    let src = buf[(yy * w + xx) as usize];
                    if src == Color32::TRANSPARENT { continue; }
                    let px = x0 + xx;
                    let py = y0 + yy;
                    if px >= 0 && px < cw && py >= 0 && py < ch {
                        pixels[(py * cw + px) as usize] = src;
                    }
                }
            }
            self.canvas_dirty = true;
        }
        self.pasting = false;
        self.sel = None;
        self.clear_move_state();
    }
}
