use std::sync::Arc;

use eframe::egui::Color32;

use super::{Document, Layer};
use crate::constants::ExportBg;

impl Document {
    pub(crate) fn add_layer(&mut self) {
        self.push_undo();
        self.layers.push(Layer {
            name: format!("Layer {}", self.layers.len()),
            cels: vec![Arc::new(vec![Color32::TRANSPARENT; self.width * self.height]); self.frames],
            visible: true,
        });
        self.active_layer = self.layers.len() - 1;
        self.canvas_dirty = true;
    }

    pub(crate) fn remove_layer(&mut self, idx: usize) {
        if self.layers.len() <= 1 { return; }
        self.push_undo();
        self.layers.remove(idx);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
        self.canvas_dirty = true;
    }

    pub(crate) fn save_png(&self, path: &str, scale: u32, bg: ExportBg) -> Result<(), String> {
        let flat = self.composite();
        write_png(&flat, self.width, self.height, path, scale, bg)
    }

    pub(crate) fn save_layer_pngs(&self, dir: &str, scale: u32, bg: ExportBg) -> Result<(), String> {
        for layer in &self.layers {
            let safe: String = layer.name.chars().map(|c| match c {
                '/' | '\\' | ':' => '-',
                c => c,
            }).collect();
            let path = format!("{}/{}.png", dir, safe);
            write_png(&layer.cels[self.active_frame], self.width, self.height, &path, scale, bg)?;
        }
        Ok(())
    }

    pub(crate) fn save_png_sheet(&self, path: &str, scale: u32, bg: ExportBg) -> Result<(), String> {
        let ow = self.width;
        let oh = self.height;
        let s = scale.max(1) as usize;
        let n = self.frames;
        let w = ow * s * n;
        let h = oh * s;
        let ck_a = [200u8, 200, 200];
        let ck_b = [180u8, 180, 180];
        let mut img = image::RgbaImage::new(w as u32, h as u32);
        for fr in 0..n {
            let flat = self.composite_at(fr);
            let ox = fr * ow * s;
            for gy in 0..h {
                let fy = gy / s;
                for gx in 0..ow * s {
                    let fx = gx / s;
                    let c = flat[fy * ow + fx];
                    let a = c.a();
                    let (r, g, b, a) = match bg {
                        ExportBg::Transparent => (c.r(), c.g(), c.b(), a),
                        ExportBg::White => blended(c.r(), c.g(), c.b(), a, 255, 255, 255),
                        ExportBg::Black => blended(c.r(), c.g(), c.b(), a, 0, 0, 0),
                        ExportBg::Checker => {
                            if a == 0 {
                                let cb = if (fx + fy) % 2 == 0 { ck_a } else { ck_b };
                                (cb[0], cb[1], cb[2], 255)
                            } else {
                                blended(c.r(), c.g(), c.b(), a, 255, 255, 255)
                            }
                        }
                    };
                    img.put_pixel((ox + gx) as u32, gy as u32, image::Rgba([r, g, b, a]));
                }
            }
        }
        img.save(path).map_err(|e| format!("Failed to save: {}", e))
    }

    pub(crate) fn load_png(&mut self, path: &str) {
        let img = match image::open(path) {
            Ok(i) => i.to_rgba8(),
            Err(_) => return,
        };
        let (w, h) = img.dimensions();
        self.push_undo();
        self.frames = 1;
        self.active_frame = 0;
        for layer in &mut self.layers {
            layer.cels = vec![Arc::new(vec![Color32::TRANSPARENT; (w * h) as usize])];
        }
        self.width = w as usize;
        self.height = h as usize;
        let layer = &mut self.layers[0];
        let pixels = Arc::make_mut(&mut layer.cels[self.active_frame]);
        for y in 0..h as usize {
            for x in 0..w as usize {
                let p = img.get_pixel(x as u32, y as u32);
                pixels[y * self.width + x] = Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]);
            }
        }
        self.active_layer = 0;
        self.tex = None;
        self.canvas_dirty = true;
        self.needs_zoom_fit = true;

        let p = std::path::Path::new(path);
        if let Some(parent) = p.parent() {
            self.export_path = parent.to_string_lossy().into();
        }
        if let Some(name) = p.file_name() {
            self.export_name = name.to_string_lossy().into();
        }
    }

    pub(crate) fn resize_canvas(&mut self, new_w: usize, new_h: usize) {
        self.push_undo();
        let ow = self.width;
        let oh = self.height;
        let fr = self.frames;
        for layer in &mut self.layers {
            let new_cels: Vec<Arc<Vec<Color32>>> = (0..fr).map(|f| {
                let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
                let old = &layer.cels[f];
                for y in 0..oh.min(new_h) {
                    for x in 0..ow.min(new_w) {
                        np[y * new_w + x] = old[y * ow + x];
                    }
                }
                Arc::new(np)
            }).collect();
            layer.cels = new_cels;
        }
        self.width = new_w;
        self.height = new_h;
        self.tex = None;
        self.canvas_dirty = true;
    }

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
            let ow = self.width;
            let fr = self.frames;
            for layer in &mut self.layers {
                let new_cels: Vec<Arc<Vec<Color32>>> = (0..fr).map(|f| {
                    let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
                    let old = &layer.cels[f];
                    for y in 0..new_h {
                        for x in 0..new_w {
                            np[y * new_w + x] = old[(sy + y) * ow + (sx + x)];
                        }
                    }
                    Arc::new(np)
                }).collect();
                layer.cels = new_cels;
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

    pub(crate) fn flatten_layers(&mut self) {
        self.push_undo();
        let mut cels = Vec::with_capacity(self.frames);
        for f in 0..self.frames {
            cels.push(Arc::new(self.composite_at(f)));
        }
        self.layers.clear();
        self.layers.push(Layer {
            name: "Flattened".into(),
            cels,
            visible: true,
        });
        self.active_layer = 0;
        self.canvas_dirty = true;
    }

    pub(crate) fn duplicate_layer(&mut self, idx: usize) {
        self.push_undo();
        let dup = Layer {
            name: format!("{} copy", self.layers[idx].name),
            cels: self.layers[idx].cels.clone(),
            visible: true,
        };
        let insert_pos = (idx + 1).min(self.layers.len());
        self.layers.insert(insert_pos, dup);
        self.active_layer = insert_pos;
        self.canvas_dirty = true;
    }

    pub(crate) fn scale_image(&mut self, new_w: usize, new_h: usize) {
        if new_w == 0 || new_h == 0 { return; }
        self.push_undo();
        let ow = self.width;
        let oh = self.height;
        let fr = self.frames;
        for layer in &mut self.layers {
            let new_cels: Vec<Arc<Vec<Color32>>> = (0..fr).map(|f| {
                let mut np = vec![Color32::TRANSPARENT; new_w * new_h];
                let old = &layer.cels[f];
                for dy in 0..new_h {
                    for dx in 0..new_w {
                        let sx = (dx as f64 * ow as f64 / new_w as f64) as usize;
                        let sy = (dy as f64 * oh as f64 / new_h as f64) as usize;
                        let sx = sx.min(ow - 1);
                        let sy = sy.min(oh - 1);
                        np[dy * new_w + dx] = old[sy * ow + sx];
                    }
                }
                Arc::new(np)
            }).collect();
            layer.cels = new_cels;
        }
        self.width = new_w;
        self.height = new_h;
        self.tex = None;
        self.canvas_dirty = true;
    }
}

fn write_png(buf: &[Color32], ow: usize, oh: usize, path: &str, scale: u32, bg: ExportBg) -> Result<(), String> {
    let w = ow * scale.max(1) as usize;
    let h = oh * scale.max(1) as usize;
    let s = scale.max(1) as usize;
    let ck_a = [200, 200, 200];
    let ck_b = [180, 180, 180];
    let mut img = image::RgbaImage::new(w as u32, h as u32);
    for y in 0..h {
        for x in 0..w {
            let c = buf[(y / s) * ow + (x / s)];
            let a = c.a();
            let (r, g, b, a) = match bg {
                ExportBg::Transparent => (c.r(), c.g(), c.b(), a),
                ExportBg::White => blended(c.r(), c.g(), c.b(), a, 255, 255, 255),
                ExportBg::Black => blended(c.r(), c.g(), c.b(), a, 0, 0, 0),
                ExportBg::Checker => {
                    if a == 0 {
                        let cb = if (x / s + y / s) % 2 == 0 { ck_a } else { ck_b };
                        (cb[0], cb[1], cb[2], 255)
                    } else {
                        blended(c.r(), c.g(), c.b(), a, 255, 255, 255)
                    }
                }
            };
            img.put_pixel(x as u32, y as u32, image::Rgba([r, g, b, a]));
        }
    }
    img.save(path).map_err(|e| format!("Failed to save: {}", e))
}

fn blended(r: u8, g: u8, b: u8, a: u8, br: u8, bg: u8, bb: u8) -> (u8, u8, u8, u8) {
    let a = a as u32;
    if a == 0 { return (br, bg, bb, 255); }
    let ia = 255 - a;
    let mix = |c: u32, bc: u32| ((c * a + bc * ia) / 255) as u8;
    (mix(r as u32, br as u32), mix(g as u32, bg as u32), mix(b as u32, bb as u32), 255)
}
