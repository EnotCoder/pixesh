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
    pub(crate) fn composite_display(&self) -> Vec<Color32> {
        let ck_a = Color32::from_gray(200);
        let ck_b = Color32::from_gray(180);
        let mut out = Vec::with_capacity(self.width * self.height);
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
                if c == Color32::TRANSPARENT {
                    c = if (x + y) % 2 == 0 { ck_a } else { ck_b };
                }
                out.push(c);
            }
        }
        out
    }

    // поставить пиксель кистью (квадрат brush_i x brush_i)
    pub(crate) fn paint_pixel(&mut self, px: i32, py: i32, color: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        let b = self.brush_i() as i32;
        let half = (b - 1) / 2;
        let pixels = self.pixels_mut(idx);
        for dy in 0..b {
            for dx in 0..b {
                let x = px + dx - half;
                let y = py + dy - half;
                if x >= 0 && x < w && y >= 0 && y < h {
                    pixels[(y * w + x) as usize] = color;
                }
            }
        }
        self.canvas_dirty = true;
    }

    // линия по Брезенхему — ставит пиксели от (x0,y0) до (x1,y1)
    pub(crate) fn paint_line(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color32) {
        let dx = (x1 - x0).abs();
        let dy = -(y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx + dy;
        let mut x = x0;
        let mut y = y0;
        loop {
            self.paint_pixel(x, y, color);
            if x == x1 && y == y1 { break; }
            let e2 = 2 * err;
            if e2 >= dy { err += dy; x += sx; }
            if e2 <= dx { err += dx; y += sy; }
        }
    }

    // заливка (flood fill) — заменяет все смежные пиксели одного цвета
    pub(crate) fn flood_fill(&mut self, px: i32, py: i32, new: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        if px < 0 || px >= w || py < 0 || py >= h { return; }
        let target = self.layers[idx].pixels[(py * w + px) as usize];
        if target == new { return; }
        let pixels = self.pixels_mut(idx);
        let mut stack = vec![(px, py)];
        while let Some((cx, cy)) = stack.pop() {
            let i = cy * w + cx;
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
