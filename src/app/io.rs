use eframe::egui::Color32;

use super::{Layer, PixeshApp};

impl PixeshApp {
    pub(crate) fn add_layer(&mut self) {
        self.push_undo();
        self.layers.push(Layer {
            name: format!("Layer {}", self.layers.len()),
            pixels: vec![Color32::TRANSPARENT; self.width * self.height],
            visible: true,
        });
        self.active_layer = self.layers.len() - 1;
    }

    pub(crate) fn remove_layer(&mut self, idx: usize) {
        if self.layers.len() <= 1 {
            return;
        }
        self.push_undo();
        self.layers.remove(idx);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
    }

    pub(crate) fn save_png(&self, path: &str) {
        let flat = self.composite();
        let mut img = image::RgbaImage::new(self.width as u32, self.height as u32);
        for y in 0..self.height {
            for x in 0..self.width {
                let c = flat[y * self.width + x];
                img.put_pixel(
                    x as u32,
                    y as u32,
                    image::Rgba([c.r(), c.g(), c.b(), c.a()]),
                );
            }
        }
        let _ = img.save(path);
    }

    pub(crate) fn load_png(&mut self, path: &str) {
        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return,
        };
        let (w, h) = img.dimensions();
        self.push_undo();
        for layer in &mut self.layers {
            layer.pixels = vec![Color32::TRANSPARENT; (w * h) as usize];
        }
        self.width = w as usize;
        self.height = h as usize;
        let layer = &mut self.layers[0];
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = img.get_pixel(x as u32, y as u32);
                layer.pixels[y * self.width + x] = if p[3] < 128 {
                    Color32::TRANSPARENT
                } else {
                    Color32::from_rgb(p[0], p[1], p[2])
                };
            }
        }
        self.active_layer = 0;
        self.tex = None;
    }

    pub(crate) fn resize_canvas(&mut self, new_w: usize, new_h: usize) {
        self.push_undo();
        for layer in &mut self.layers {
            let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
            for y in 0..self.height.min(new_h) {
                for x in 0..self.width.min(new_w) {
                    np[y * new_w + x] = layer.pixels[y * self.width + x];
                }
            }
            layer.pixels = np;
        }
        self.width = new_w;
        self.height = new_h;
        self.tex = None;
    }
}
