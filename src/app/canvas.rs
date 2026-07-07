use eframe::egui::{Color32, Pos2};

use super::PixeshApp;

impl PixeshApp {
    pub(crate) fn brush_i(&self) -> usize {
        self.brush.round() as usize
    }

    pub(crate) fn composite(&self) -> Vec<Color32> {
        let mut out = vec![Color32::TRANSPARENT; self.width * self.height];
        for layer in &self.layers {
            if !layer.visible {
                continue;
            }
            for (i, &p) in layer.pixels.iter().enumerate() {
                if p != Color32::TRANSPARENT {
                    out[i] = p;
                }
            }
        }
        out
    }

    pub(crate) fn paint_pixel(&mut self, px: i32, py: i32, color: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() {
            return;
        }
        let w = self.width as i32;
        let h = self.height as i32;
        let b = self.brush_i() as i32;
        let half = (b - 1) / 2;
        let layer = &mut self.layers[idx];
        for dy in 0..b {
            for dx in 0..b {
                let x = px + dx - half;
                let y = py + dy - half;
                if x >= 0 && x < w && y >= 0 && y < h {
                    layer.pixels[(y * w + x) as usize] = color;
                }
            }
        }
    }

    pub(crate) fn flood_fill(&mut self, px: i32, py: i32, new: Color32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() { return; }
        let w = self.width as i32;
        let h = self.height as i32;
        if px < 0 || px >= w || py < 0 || py >= h { return; }
        let layer = &self.layers[idx];
        let target = layer.pixels[(py * w + px) as usize];
        if target == new { return; }
        let mut stack = vec![(px, py)];
        let layer = &mut self.layers[idx];
        while let Some((cx, cy)) = stack.pop() {
            let i = cy * w + cx;
            if layer.pixels[i as usize] != target { continue; }
            layer.pixels[i as usize] = new;
            if cx > 0     { stack.push((cx - 1, cy)); }
            if cx + 1 < w { stack.push((cx + 1, cy)); }
            if cy > 0     { stack.push((cx, cy - 1)); }
            if cy + 1 < h { stack.push((cx, cy + 1)); }
        }
    }

    pub(crate) fn screen_to_pixel(&self, pos: Pos2, origin: Pos2) -> (i32, i32) {
        let r = pos - origin;
        ((r.x / self.zoom) as i32, (r.y / self.zoom) as i32)
    }
}
