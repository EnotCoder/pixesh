use eframe::egui::Color32;

use super::Document;
use crate::constants::BrushShape;

impl Document {
    pub(crate) fn handle_brush_press(&mut self, px: i32, py: i32, color: Color32, brush: f32, shape: BrushShape) {
        self.push_undo();
        self.paint_pixel(px, py, color, brush, shape);
        self.last_px_primary = Some((px, py));
    }

    pub(crate) fn handle_brush_drag(&mut self, px: i32, py: i32, color: Color32, brush: f32, shape: BrushShape) {
        if let Some(last) = self.last_px_primary {
            self.paint_line(last.0, last.1, px, py, color, brush, shape);
        } else {
            self.paint_pixel(px, py, color, brush, shape);
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
                        buf.push(self.layers[self.active_layer].cels[self.active_frame][idx]);
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
                    if was_pasting {
                        // handle_*_press returns early for a paste without
                        // taking a snapshot, and we do not call
                        // commit_pending_paste here — without this Ctrl+Z
                        // would not revert the dragged block
                        self.push_undo();
                    }
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
            let pixels = self.pixels_mut(self.active_layer);
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
                        buf.push(self.layers[self.active_layer].cels[self.active_frame][idx]);
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
                    if was_pasting {
                        // handle_*_press returns early for a paste without
                        // taking a snapshot, and we do not call
                        // commit_pending_paste here — without this Ctrl+Z
                        // would not revert the dragged block
                        self.push_undo();
                    }
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
        // An unfinished paste lives in sel_buffer and has not reached the
        // layer yet: erasing the rectangle would cut out the old content
        // underneath it.
        if self.pasting { return; }
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

    /// Ctrl+A: select the whole canvas, like in Photoshop.
    pub(crate) fn select_all(&mut self) {
        // The floating paste has not landed on the layer yet. Commit it first:
        // otherwise replacing sel would leave pasting=true with an emptied
        // sel_buffer, and Enter would then behave strangely.
        self.commit_pending_paste();
        // The range is inclusive, hence -1: a canvas of width w has coordinates
        // 0..=w-1, and going outside them is not allowed — delete_selection
        // would panic on the index.
        self.sel = Some((0, 0, self.width as i32 - 1, self.height as i32 - 1));
        // The drag remainder is hidden by the painter while sel is empty.
        // Clear it, otherwise the old marquee would come back after deselecting.
        self.sel_start = None;
        self.sel_end = None;
        self.clear_move_state();
    }

    pub(crate) fn commit_pending_paste(&mut self) {
        if !self.pasting { return; }
        if let (Some(buf), Some((x0, y0, _x1, _y1))) = (self.sel_buffer.take(), self.sel) {
            self.push_undo();
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

    /// Cancels an unfinished paste: the floating selection disappears, the
    /// layer was never modified, so there is nothing to undo in history.
    pub(crate) fn cancel_pending_paste(&mut self) -> bool {
        if !self.pasting { return false; }
        self.pasting = false;
        self.sel = None;
        self.sel_start = None;
        self.sel_end = None;
        self.clear_move_state();
        self.canvas_dirty = true;
        true
    }

    /// Copies the contents of the selection to the clipboard.
    /// Returns false if there is no selection.
    pub(crate) fn copy_selection(&mut self) -> bool {
        let Some((x0, y0, x1, y1)) = self.sel else { return false };

        // While the paste is unconfirmed, the screen shows sel_buffer rather
        // than the layer pixels — that is what must be copied.
        if self.pasting {
            let Some(buf) = self.sel_buffer.clone() else { return false };
            self.clipboard = Some(buf);
            self.clip_w = self.sel_buf_w;
            self.clip_h = self.sel_buf_h;
            return true;
        }

        let sw = (x1 - x0 + 1) as usize;
        let sh = (y1 - y0 + 1) as usize;
        let w = self.width as i32;
        let mw = self.width as i32 - 1;
        let mh = self.height as i32 - 1;
        let cel = &self.layers[self.active_layer].cels[self.active_frame];
        let mut buf = Vec::with_capacity(sw * sh);
        for yy in y0..=y1.min(mh) {
            for xx in x0..=x1.min(mw) {
                buf.push(cel[(yy * w + xx) as usize]);
            }
        }
        self.clipboard = Some(buf);
        self.clip_w = sw;
        self.clip_h = sh;
        true
    }

    /// Pastes the clipboard as a "floating" selection that is immediately
    /// ready to be dragged with the mouse. Returns false if the clipboard is empty.
    pub(crate) fn paste_clipboard(&mut self) -> bool {
        // Commit the unfinished paste first, otherwise Ctrl+V over Ctrl+V
        // would lose the first copy.
        self.commit_pending_paste();

        let Some(clip) = self.clipboard.clone() else { return false };
        let cw = self.clip_w as i32;
        let ch = self.clip_h as i32;
        if cw <= 0 || ch <= 0 || clip.len() != (cw * ch) as usize {
            return false;
        }

        let w = self.width as i32;
        let h = self.height as i32;
        // If there is an existing selection, paste offset from it; otherwise
        // the block would land exactly in place and the "move" would be invisible.
        let (mut cx, mut cy) = match self.sel {
            Some((x0, y0, ..)) => (x0 + 2, y0 + 2),
            None => ((w - cw) / 2, (h - ch) / 2),
        };
        cx = cx.clamp(0, (w - cw).max(0));
        cy = cy.clamp(0, (h - ch).max(0));

        self.sel = Some((cx, cy, cx + cw - 1, cy + ch - 1));
        self.sel_buffer = Some(clip);
        self.sel_buf_w = self.clip_w;
        self.sel_buf_h = self.clip_h;
        self.sel_start = None;
        self.sel_end = None;
        self.canvas_move_origin = None;
        self.canvas_move_current = None;
        self.sel_tex = None;
        self.pasting = true;
        // start in drag mode right away so it can be grabbed immediately
        let center = (cx + cw / 2, cy + ch / 2);
        self.sel_move_origin = Some(center);
        self.sel_move_current = Some(center);
        self.canvas_dirty = true;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::Document;
    use std::sync::Arc;

    const W: i32 = 16;

    fn red() -> Color32 { Color32::from_rgb(255, 0, 0) }

    /// A 16x16 document with the given pixels on the active layer.
    fn doc_with(px: &[(i32, i32, Color32)]) -> Document {
        let mut d = Document::new_sized("t", 16, 16);
        {
            let layer = &mut d.layers[0];
            let cel = Arc::make_mut(&mut layer.cels[0]);
            for &(x, y, c) in px {
                cel[(y * W + x) as usize] = c;
            }
        }
        d
    }

    fn at(d: &Document, x: i32, y: i32) -> Color32 {
        d.layers[0].cels[0][(y * W + x) as usize]
    }

    #[test]
    fn copy_needs_a_selection() {
        let mut d = doc_with(&[]);
        assert!(!d.copy_selection());
        assert!(d.clipboard.is_none());
    }

    #[test]
    fn copy_grabs_selection_pixels() {
        let mut d = doc_with(&[(3, 3, red()), (4, 4, red())]);
        d.sel = Some((3, 3, 4, 4));
        assert!(d.copy_selection());
        assert_eq!(d.clip_w, 2);
        assert_eq!(d.clip_h, 2);
        assert_eq!(d.clipboard.as_ref().unwrap().len(), 4);
    }

    #[test]
    fn select_all_covers_the_whole_canvas() {
        let mut d = doc_with(&[]);
        d.select_all();
        // The range is inclusive: coordinates go up to w-1/h-1, not w/h.
        assert_eq!(d.sel, Some((0, 0, W - 1, W - 1)));
    }

    /// The main protection against width/height confusion: on a rectangular
    /// canvas the selection must follow its shape instead of becoming square.
    #[test]
    fn select_all_respects_non_square_documents() {
        let mut d = Document::new_sized("t", 8, 5);
        d.select_all();
        assert_eq!(d.sel, Some((0, 0, 7, 4)), "8 по горизонтали, 5 по вертикали");
    }

    #[test]
    fn select_all_clears_a_half_dragged_marquee() {
        let mut d = doc_with(&[]);
        d.sel_start = Some((2, 2));
        d.sel_end = Some((6, 6));
        d.select_all();
        assert_eq!(d.sel_start, None, "остаток перетаскивания убран");
        assert_eq!(d.sel_end, None, "остаток перетаскивания убран");
    }

    /// The floating paste has not landed on the layer yet. If sel were simply
    /// replaced, pasting would stay true with an empty sel_buffer — Enter would
    /// behave strangely.
    #[test]
    fn select_all_finishes_a_pending_paste() {
        let mut d = doc_with(&[(3, 3, red())]);
        d.sel = Some((3, 3, 3, 3));
        assert!(d.copy_selection());
        assert!(d.paste_clipboard());
        assert!(d.pasting, "вставка висит до select_all");

        d.select_all();
        assert!(!d.pasting, "вставка подтверждена, а не брошена");
        assert_eq!(d.sel, Some((0, 0, W - 1, W - 1)));
        assert!(d.sel_buffer.is_none());
    }

    #[test]
    fn paste_needs_a_clipboard() {
        let mut d = doc_with(&[]);
        assert!(!d.paste_clipboard());
        assert!(!d.pasting);
    }

    #[test]
    fn paste_lands_offset_and_leaves_layer_untouched() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 4, 4));
        assert!(d.copy_selection());
        let before = d.layers[0].cels[0].clone();

        assert!(d.paste_clipboard());
        assert!(d.pasting, "вставка должна остаться незавершённой");
        assert_eq!(d.sel, Some((4, 4, 6, 6)), "блок вставлен со сдвигом +2,+2");
        assert_eq!(d.layers[0].cels[0], before, "до подтверждения слой не меняется");
        assert!(d.sel_buffer.is_some());
    }

    #[test]
    fn pasted_block_can_be_moved_and_committed() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 4, 4));
        d.copy_selection();
        d.paste_clipboard();

        // grab the floating block (already at 4,4) and drag it by +6,+6 → 10,10
        d.sel_move_origin = Some((5, 5));
        d.sel_move_current = Some((11, 11));
        d.handle_move_release();

        assert!(!d.pasting, "после отпускания вставка подтверждена");
        assert_eq!(at(&d, 10, 10), red(), "блок лёг на новое место");
        assert_eq!(at(&d, 2, 2), red(), "оригинал на месте не тронут: это paste, а не move");
        assert!(d.sel.is_none());
    }

    #[test]
    fn undo_after_paste_restores_original() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 4, 4));
        d.copy_selection();
        d.paste_clipboard();
        d.sel_move_origin = Some((5, 5));
        d.sel_move_current = Some((11, 11));
        d.handle_move_release();
        assert_eq!(at(&d, 10, 10), red());

        d.undo();
        assert_eq!(at(&d, 10, 10), Color32::TRANSPARENT, "Ctrl+Z убирает вставку");
        assert_eq!(at(&d, 2, 2), red(), "оригинал вернулся");
    }

    #[test]
    fn escape_cancels_paste_without_touching_layer() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 4, 4));
        d.copy_selection();
        let before = d.layers[0].cels[0].clone();
        d.paste_clipboard();
        assert!(d.pasting);

        assert!(d.cancel_pending_paste());
        assert!(!d.pasting);
        assert!(d.sel.is_none());
        assert_eq!(d.layers[0].cels[0], before, "отмена не должна менять слой");
        assert!(!d.cancel_pending_paste(), "повторный вызов — no-op");
    }

    #[test]
    fn delete_ignores_pending_paste() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 4, 4));
        d.copy_selection();
        d.paste_clipboard();
        // the paste rectangle has not been written to the layer yet — nothing to erase
        d.delete_selection();
        assert_eq!(at(&d, 2, 2), red(), "Delete не вырезал старые пиксели");
        assert!(d.pasting, "и не сбросил незавершённую вставку");
    }

    #[test]
    fn second_paste_commits_the_first() {
        let mut d = doc_with(&[(2, 2, red())]);
        d.sel = Some((2, 2, 2, 2));
        d.copy_selection();
        d.paste_clipboard();
        // the first block is left unfinished
        d.paste_clipboard();
        assert!(d.pasting, "вторая вставка снова незавершённая");
        // the first one has been committed by now
        assert_eq!(at(&d, 4, 4), red());
    }
}
