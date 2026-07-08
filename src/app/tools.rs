use eframe::egui::Color32;

use crate::color::*;
use super::PixeshApp;

impl PixeshApp {
    // ── Eyedropper ──────────────────────────────────────
    // взять цвет пикселя под курсором и установить как текущий
    pub(crate) fn handle_eyedropper(&mut self, px: i32, py: i32) {
        let w = self.width as i32;
        let h = self.height as i32;
        if px >= 0 && px < w && py >= 0 && py < h {
            let c = self.composite()[(py * w + px) as usize];
            self.color = c;
            self.rgb_r = c.r() as f32;
            self.rgb_g = c.g() as f32;
            self.rgb_b = c.b() as f32;
            self.rgb_a = c.a() as f32;
            let (h_, s, v) = rgb_to_hsv(c.r(), c.g(), c.b());
            self.hsv_h = h_;
            self.hsv_s = s;
            self.hsv_v = v;
        }
    }

    // ── Fill ────────────────────────────────────────────
    // залить область текущим цветом (flood fill)
    pub(crate) fn handle_fill(&mut self, px: i32, py: i32) {
        self.push_undo();
        self.flood_fill(px, py, self.color);
    }

    // ── Brush / Eraser ─────────────────────────────────
    // начало рисования кистью — сохранить undo, поставить пиксель
    pub(crate) fn handle_brush_press(&mut self, px: i32, py: i32, color: Color32) {
        self.push_undo();
        self.paint_pixel(px, py, color);
        self.last_px_primary = Some((px, py));
    }

    // движение кисти — рисовать линию от предыдущего пикселя до текущего
    pub(crate) fn handle_brush_drag(&mut self, px: i32, py: i32, color: Color32) {
        if let Some(last) = self.last_px_primary {
            self.paint_line(last.0, last.1, px, py, color);
        } else {
            self.paint_pixel(px, py, color);
        }
        self.last_px_primary = Some((px, py));
    }

    // клик кистью — поставить пиксель с сохранением undo
    pub(crate) fn handle_brush_click(&mut self, px: i32, py: i32, color: Color32) {
        self.push_undo();
        self.paint_pixel(px, py, color);
    }

    // ── Selection ──────────────────────────────────────
    // начало работы с выделением: если клик внутри существующего — начать перемещение
    // иначе — начать рисовать новый прямоугольник выделения
    pub(crate) fn handle_select_press(&mut self, px: i32, py: i32) {
        if let Some((x0, y0, x1, y1)) = self.sel {
            if px >= x0 && px <= x1 && py >= y0 && py <= y1 {
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

    // движение мыши при активном выделении: обновить прямоугольник или позицию перемещения
    pub(crate) fn handle_select_drag(&mut self, px: i32, py: i32) {
        if self.sel_move_origin.is_some() {
            self.sel_move_current = Some((px, py));
            return;
        }
        if self.sel_start.is_some() {
            self.sel_end = Some((px, py));
        }
    }

    // отпускание мыши: завершить перемещение или финализировать прямоугольник выделения
    pub(crate) fn handle_select_release(&mut self) {
        // завершение мува
        if self.sel_move_origin.is_some() {
            if let (Some(current), Some((x0, y0, x1, y1))) = (self.sel_move_current, self.sel) {
                if let Some(origin) = self.sel_move_origin {
                    let w = self.sel_buf_w as i32;
                    let h = self.sel_buf_h as i32;
                    let dx = current.0 - origin.0;
                    let dy = current.1 - origin.1;
                    let nx0 = (x0 + dx).max(0).min(self.width as i32 - w);
                    let ny0 = (y0 + dy).max(0).min(self.height as i32 - h);
                    let nx1 = nx0 + w - 1;
                    let ny1 = ny0 + h - 1;
                    let cw = self.width as i32;

                    if let Some(buf) = self.sel_buffer.take() {
                        let pixels = self.pixels_mut(self.active_layer);
                        // стереть старое положение
                        for yy in y0..=y1 {
                            for xx in x0..=x1 {
                                pixels[(yy * cw + xx) as usize] = Color32::TRANSPARENT;
                            }
                        }
                        // вставить в новое
                        for yy in 0..h {
                            for xx in 0..w {
                                let src = buf[(yy * w + xx) as usize];
                                if src != Color32::TRANSPARENT {
                                    pixels[((ny0 + yy) * cw + nx0 + xx) as usize] = src;
                                }
                            }
                        }
                    }
                    self.sel = Some((nx0, ny0, nx1, ny1));
                    self.canvas_dirty = true;
                }
            }
            self.clear_move_state();
            return;
        }

        // завершение селекта
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

    // сбросить всё состояние перемещения выделения
    pub(crate) fn clear_move_state(&mut self) {
        self.sel_move_origin = None;
        self.sel_move_current = None;
        self.sel_buffer = None;
        self.sel_tex = None;
    }

    // удалить пиксели внутри выделения и сбросить его
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

    // снять выделение без удаления пикселей
    pub(crate) fn deselect(&mut self) {
        self.sel = None;
        self.sel_start = None;
        self.sel_end = None;
        self.clear_move_state();
    }
}
