use std::sync::Arc;

use eframe::egui::{Color32, Pos2};

use super::PixeshApp;

impl PixeshApp {
    // округлённый размер кисти (из f32 -> usize)
    pub(crate) fn brush_i(&self) -> usize {
        self.brush.round() as usize
    }

    // доступ к пикселям слоя через Arc::make_mut (COW для undo)
    pub(crate) fn pixels_mut(&mut self, layer_idx: usize) -> &mut Vec<Color32> {
        Arc::make_mut(&mut self.layers[layer_idx].pixels)
    }

    // композит всех видимых слоёв поверх друг друга
    pub(crate) fn composite(&self) -> Vec<Color32> {
        let mut out = vec![Color32::TRANSPARENT; self.width * self.height];
        for layer in &self.layers {
            if !layer.visible { continue; }
            for (i, &p) in layer.pixels.iter().enumerate() {
                if p != Color32::TRANSPARENT {
                    out[i] = p;
                }
            }
        }
        out
    }

    // композит с шахматным фоном для прозрачных пикселей (отображение)
    pub(crate) fn composite_display(&mut self) -> &Vec<Color32> {
        let ck_a = Color32::from_gray(200);
        let ck_b = Color32::from_gray(180);
        let n = self.width * self.height;
        self.display_buf.clear();
        self.display_buf.reserve(n);
        for y in 0..self.height {
            for x in 0..self.width {
                let idx = y * self.width + x;
                let mut c = Color32::TRANSPARENT;
                for layer in &self.layers {
                    if !layer.visible { continue; }
                    let p = layer.pixels[idx];
                    if p != Color32::TRANSPARENT {
                        c = p;
                        break;
                    }
                }
                let cb = if (x + y) % 2 == 0 { ck_a } else { ck_b };
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
                self.display_buf.push(c);
            }
        }
        &self.display_buf
    }

    // поставить пиксель кистью (квадрат brush_i x brush_i)
    pub(crate) fn paint_pixel(&mut self, px: i32, py: i32, color: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        let b = self.brush_i() as i32;
        let half = (b - 1) / 2;
        let sel = self.sel;
        let pixels = self.pixels_mut(idx);
        for dy in 0..b {
            for dx in 0..b {
                let x = px + dx - half;
                let y = py + dy - half;
                let in_sel = match sel {
                    Some((x0, y0, x1, y1)) => x >= x0 && x <= x1 && y >= y0 && y <= y1,
                    None => true,
                };
                if x >= 0 && x < w && y >= 0 && y < h && in_sel {
                    pixels[(y * w + x) as usize] = color;
                }
            }
        }
        self.canvas_dirty = true;
    }

    // линия по Брезенхему — ставит пиксели от (x0,y0) до (x1,y1)
    pub(crate) fn paint_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        let b = self.brush_i() as i32;
        let half = (b - 1) / 2;
        let sel = self.sel;
        let pixels = self.pixels_mut(idx);
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut cx = x0;
        let mut cy = y0;
        loop {
            for dy2 in 0..b {
                for dx2 in 0..b {
                    let x = cx + dx2 - half;
                    let y = cy + dy2 - half;
                    let in_sel = match sel {
                        Some((x0s, y0s, x1s, y1s)) => x >= x0s && x <= x1s && y >= y0s && y <= y1s,
                        None => true,
                    };
                    if x >= 0 && x < w && y >= 0 && y < h && in_sel {
                        pixels[(y * w + x) as usize] = color;
                    }
                }
            }
            if cx == x1 && cy == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; cx += sx; }
            if e2 <= dx { err += dx; cy += sy; }
        }
        self.canvas_dirty = true;
    }

    // заливка (flood fill) — заменяет все смежные пиксели одного цвета (внутри выделения)
    pub(crate) fn flood_fill(&mut self, px: i32, py: i32, new: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        if px < 0 || px >= w || py < 0 || py >= h { return; }
        let sel = self.sel;
        let in_sel = match sel {
            Some((x0, y0, x1, y1)) => px >= x0 && px <= x1 && py >= y0 && py <= y1,
            None => true,
        };
        if !in_sel { return; }
        let target = self.layers[idx].pixels[(py * w + px) as usize];
        if target == new { return; }
        let pixels = self.pixels_mut(idx);
        let mut stack = vec![(px, py)];
        while let Some((cx, cy)) = stack.pop() {
            let i = cy * w + cx;
            let in_s = match sel {
                Some((x0, y0, x1, y1)) => cx >= x0 && cx <= x1 && cy >= y0 && cy <= y1,
                None => true,
            };
            if !in_s { continue; }
            if pixels[i as usize] != target { continue; }
            pixels[i as usize] = new;
            if cx > 0     { stack.push((cx - 1, cy)); }
            if cx + 1 < w { stack.push((cx + 1, cy)); }
            if cy > 0     { stack.push((cx, cy - 1)); }
            if cy + 1 < h { stack.push((cx, cy + 1)); }
        }
        self.canvas_dirty = true;
    }

    // зеркальное отражение по горизонтали (активный слой)
    pub(crate) fn mirror_horizontal(&mut self) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width;
        let h = self.height;
        let pixels = self.pixels_mut(idx);
        for y in 0..h {
            for x in 0..w / 2 {
                let a = y * w + x;
                let b = y * w + (w - 1 - x);
                pixels.swap(a, b);
            }
        }
        self.canvas_dirty = true;
    }

    // зеркальное отражение по вертикали (активный слой)
    pub(crate) fn mirror_vertical(&mut self) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width;
        let h = self.height;
        let pixels = self.pixels_mut(idx);
        for y in 0..h / 2 {
            for x in 0..w {
                let a = y * w + x;
                let b = (h - 1 - y) * w + x;
                pixels.swap(a, b);
            }
        }
        self.canvas_dirty = true;
    }

    // конвертировать экранные координаты в пиксельные с учётом zoom
    pub(crate) fn screen_to_pixel(&self, pos: Pos2, origin: Pos2) -> (i32, i32) {
        let r = pos - origin;
        ((r.x / self.zoom) as i32, (r.y / self.zoom) as i32)
    }
}
