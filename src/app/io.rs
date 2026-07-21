use std::sync::Arc;

use eframe::egui::Color32;

use super::{Layer, PixeshApp};

impl PixeshApp {
    // создать новый пустой слой и сделать его активным
    pub(crate) fn add_layer(&mut self) {
        self.push_undo();
        self.layers.push(Layer {
            name: format!("Layer {}", self.layers.len()),
            pixels: Arc::new(vec![Color32::TRANSPARENT; self.width * self.height]),
            visible: true,
        });
        self.active_layer = self.layers.len() - 1;
        self.canvas_dirty = true;
    }

    // удалить слой по индексу (нельзя удалить последний)
    pub(crate) fn remove_layer(&mut self, idx: usize) {
        if self.layers.len() <= 1 { return; }
        self.push_undo();
        self.layers.remove(idx);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
        self.canvas_dirty = true;
    }

    // сохранить композит canvas в PNG по заданному пути
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

    // загрузить PNG из файла — заменяет все слои, подгоняет размер холста
    pub(crate) fn load_png(&mut self, path: &str) {
        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return,
        };
        let (w, h) = img.dimensions();
        self.push_undo();
        for layer in &mut self.layers {
            layer.pixels = Arc::new(vec![Color32::TRANSPARENT; (w * h) as usize]);
        }
        self.width = w as usize;
        self.height = h as usize;
        let layer = &mut self.layers[0];
        let pixels = Arc::make_mut(&mut layer.pixels);
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = img.get_pixel(x as u32, y as u32);
                pixels[y * self.width + x] = if p[3] < 128 {
                    Color32::TRANSPARENT
                } else {
                    Color32::from_rgb(p[0], p[1], p[2])
                };
            }
        }
        self.active_layer = 0;
        self.tex = None;
        self.canvas_dirty = true;

        // сохраняем путь загруженного файла для авто-заполнения диалога экспорта
        let p = std::path::Path::new(path);
        if let Some(parent) = p.parent() {
            self.export_path = parent.to_string_lossy().into();
        }
        if let Some(name) = p.file_name() {
            self.export_name = name.to_string_lossy().into();
        }
    }

    // изменить размер холста (с обрезкой/расширением всех слоёв)
    pub(crate) fn resize_canvas(&mut self, new_w: usize, new_h: usize) {
        self.push_undo();
        for layer in &mut self.layers {
            let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
            for y in 0..self.height.min(new_h) {
                for x in 0..self.width.min(new_w) {
                    np[y * new_w + x] = layer.pixels[y * self.width + x];
                }
            }
            layer.pixels = Arc::new(np);
        }
        self.width = new_w;
        self.height = new_h;
        self.tex = None;
        self.canvas_dirty = true;
    }

    // обрезать холст по выделению
    pub(crate) fn crop_to_selection(&mut self) {
        if let Some((x0, y0, x1, y1)) = self.sel {
            let sx = x0.min(x1).max(0) as usize;
            let sy = y0.min(y1).max(0) as usize;
            let ex = (x0.max(x1) as usize).min(self.width - 1);
            let ey = (y0.max(y1) as usize).min(self.height - 1);
            let new_w = ex - sx + 1;
            let new_h = ey - sy + 1;
            if new_w == 0 || new_h == 0 { return; }
            self.push_undo();
            for layer in &mut self.layers {
                let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
                for y in 0..new_h {
                    for x in 0..new_w {
                        np[y * new_w + x] = layer.pixels[(sy + y) * self.width + (sx + x)];
                    }
                }
                layer.pixels = Arc::new(np);
            }
            self.width = new_w;
            self.height = new_h;
            self.sel = None;
            self.sel_start = None;
            self.sel_end = None;
            self.tex = None;
            self.canvas_dirty = true;
        }
    }

    // масштабировать изображение (nearest-neighbour) — меняет размер всех слоёв
    pub(crate) fn scale_image(&mut self, new_w: usize, new_h: usize) {
        if new_w == 0 || new_h == 0 { return; }
        self.push_undo();
        let ow = self.width;
        let oh = self.height;
        for layer in &mut self.layers {
            let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
            for dy in 0..new_h {
                for dx in 0..new_w {
                    let sx = (dx as f64 * ow as f64 / new_w as f64) as usize;
                    let sy = (dy as f64 * oh as f64 / new_h as f64) as usize;
                    let sx = sx.min(ow - 1);
                    let sy = sy.min(oh - 1);
                    np[dy * new_w + dx] = layer.pixels[sy * ow + sx];
                }
            }
            layer.pixels = Arc::new(np);
        }
        self.width = new_w;
        self.height = new_h;
        self.tex = None;
        self.canvas_dirty = true;
    }
}
