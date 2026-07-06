use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, TextureHandle, TextureOptions, Vec2};

const PALETTE: &[Color32] = &[
    Color32::BLACK,
    Color32::WHITE,
    Color32::RED,
    Color32::GREEN,
    Color32::BLUE,
    Color32::YELLOW,
    Color32::MAGENTA,
    Color32::CYAN,
    Color32::from_rgb(255, 128, 0),
    Color32::from_rgb(128, 64, 0),
    Color32::from_rgb(128, 0, 255),
    Color32::from_rgb(0, 255, 128),
    Color32::from_gray(64),
    Color32::from_gray(128),
    Color32::from_gray(192),
];

struct Layer {
    name: String,
    pixels: Vec<Color32>,
    visible: bool,
}

struct Snapshot {
    layers: Vec<Vec<Color32>>,
    active: usize,
}

struct PixeshApp {
    layers: Vec<Layer>,
    active_layer: usize,
    width: usize,
    height: usize,

    color: Color32,
    brush: usize,
    last_px_primary: Option<(i32, i32)>,
    last_px_secondary: Option<(i32, i32)>,

    grid: bool,
    zoom: f32,
    tex: Option<TextureHandle>,

    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,

    show_resize: bool,
    resize_w: usize,
    resize_h: usize,
}

impl PixeshApp {
    fn new() -> Self {
        Self {
            layers: vec![Layer {
                name: "Background".into(),
                pixels: vec![Color32::WHITE; 64 * 64],
                visible: true,
            }],
            active_layer: 0,
            width: 64,
            height: 64,
            color: Color32::BLACK,
            brush: 1,
            last_px_primary: None,
            last_px_secondary: None,
            grid: true,
            zoom: 10.0,
            tex: None,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_resize: false,
            resize_w: 64,
            resize_h: 64,
        }
    }

    fn composite(&self) -> Vec<Color32> {
        let mut out = vec![Color32::WHITE; self.width * self.height];
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

    fn paint_pixel(&mut self, px: i32, py: i32) {
        let idx = self.active_layer;
        if idx >= self.layers.len() {
            return;
        }
        let w = self.width as i32;
        let h = self.height as i32;
        let half = (self.brush as i32 - 1) / 2;
        let layer = &mut self.layers[idx];
        for dy in 0..self.brush as i32 {
            for dx in 0..self.brush as i32 {
                let x = px + dx - half;
                let y = py + dy - half;
                if x >= 0 && x < w && y >= 0 && y < h {
                    layer.pixels[(y * w + x) as usize] = self.color;
                }
            }
        }
    }

    fn screen_to_pixel(&self, pos: Pos2, origin: Pos2) -> (i32, i32) {
        let r = pos - origin;
        ((r.x / self.zoom) as i32, (r.y / self.zoom) as i32)
    }

    fn draw_line(&mut self, from: (i32, i32), to: (i32, i32)) {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let steps = dx.abs().max(dy.abs());
        if steps <= 1 {
            self.paint_pixel(to.0, to.1);
            return;
        }
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            self.paint_pixel(
                (from.0 as f32 + dx as f32 * t + 0.5) as i32,
                (from.1 as f32 + dy as f32 * t + 0.5) as i32,
            );
        }
    }

    fn push_undo(&mut self) {
        self.undo_stack.push(Snapshot {
            layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
            active: self.active_layer,
        });
        self.redo_stack.clear();
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
            });
            for (i, p) in state.layers.into_iter().enumerate() {
                if i < self.layers.len() {
                    self.layers[i].pixels = p;
                }
            }
            self.active_layer = state.active;
        }
    }

    fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
            });
            for (i, p) in state.layers.into_iter().enumerate() {
                if i < self.layers.len() {
                    self.layers[i].pixels = p;
                }
            }
            self.active_layer = state.active;
        }
    }

    fn add_layer(&mut self) {
        self.push_undo();
        self.layers.push(Layer {
            name: format!("Layer {}", self.layers.len()),
            pixels: vec![Color32::TRANSPARENT; self.width * self.height],
            visible: true,
        });
        self.active_layer = self.layers.len() - 1;
    }

    fn remove_layer(&mut self, idx: usize) {
        if self.layers.len() <= 1 {
            return;
        }
        self.push_undo();
        self.layers.remove(idx);
        if self.active_layer >= self.layers.len() {
            self.active_layer = self.layers.len() - 1;
        }
    }

    fn save_png(&self, path: &str) {
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

    fn load_png(&mut self, path: &str) {
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

    fn resize_canvas(&mut self, new_w: usize, new_h: usize) {
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

impl eframe::App for PixeshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- keyboard ---
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.undo();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
                self.redo();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .set_file_name("pixesh.png")
                    .save_file()
                {
                    self.save_png(&path.to_string_lossy());
                }
            }
        });

        // --- toolbar ---
        egui::TopBottomPanel::top("tools").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Pixesh");
                ui.separator();

                // Palette
                for &c in PALETTE {
                    let (r, resp) =
                        ui.allocate_exact_size(Vec2::splat(20.0), Sense::click());
                    let p = ui.painter();
                    p.rect_filled(r, 2.0, c);
                    p.rect_stroke(
                        r,
                        2.0,
                        Stroke::new(
                            if c == self.color { 2.5 } else { 1.0 },
                            Color32::from_gray(120),
                        ),
                        egui::StrokeKind::Outside,
                    );
                    if resp.clicked() {
                        self.color = c;
                    }
                }

                ui.separator();
                ui.add(egui::Slider::new(&mut self.brush, 1..=10).text("Brush"));

                if ui.button("Clear").clicked() {
                    self.push_undo();
                    for layer in &mut self.layers {
                        layer.pixels = if layer.name == "Background" {
                            vec![Color32::WHITE; self.width * self.height]
                        } else {
                            vec![Color32::TRANSPARENT; self.width * self.height]
                        };
                    }
                }

                ui.checkbox(&mut self.grid, "Grid");
                ui.add(egui::Slider::new(&mut self.zoom, 2.0..=40.0).text("Zoom"));

                ui.separator();
                if ui.button("Save").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .set_file_name("pixesh.png")
                        .save_file()
                    {
                        self.save_png(&path.to_string_lossy());
                    }
                }
                if ui.button("Load").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .pick_file()
                    {
                        let path = path.to_string_lossy().to_string();
                        self.load_png(&path);
                    }
                }

                if ui.button("Resize").clicked() {
                    self.resize_w = self.width;
                    self.resize_h = self.height;
                    self.show_resize = true;
                }

                ui.separator();
                if ui.button("Undo").clicked() {
                    self.undo();
                }
                if ui.button("Redo").clicked() {
                    self.redo();
                }
            });
        });

        // --- layers ---
        egui::SidePanel::right("layers")
            .resizable(true)
            .default_width(150.0)
            .show(ctx, |ui| {
                ui.heading("Layers");
                ui.separator();

                let n = self.layers.len();
                for i in (0..n).rev() {
                    let visible = self.layers[i].visible;
                    let name = self.layers[i].name.clone();
                    let mut cb = visible;
                    ui.horizontal(|ui| {
                        ui.checkbox(&mut cb, "");
                        if ui.selectable_label(self.active_layer == i, &name).clicked() {
                            self.active_layer = i;
                        }
                    });
                    self.layers[i].visible = cb;
                }

                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button("+").clicked() {
                        self.add_layer();
                    }
                    if ui.button("-").clicked() {
                        self.remove_layer(self.active_layer);
                    }
                });
            });

        // --- canvas ---
        egui::CentralPanel::default().show(ctx, |ui| {
            let canvas_size = Vec2::new(
                self.width as f32 * self.zoom,
                self.height as f32 * self.zoom,
            );
            let avail = ui.available_size();

            // Center the canvas
            let top = ((avail.y - canvas_size.y) * 0.5).max(0.0);
            ui.add_space(top);

            ui.vertical_centered(|ui| {
                // Composite all layers for display
                let flat = self.composite();
                let img = ColorImage {
                    size: [self.width, self.height],
                    pixels: flat,
                };

                let tex = self.tex.get_or_insert_with(|| {
                    ui.ctx()
                        .load_texture("canvas", img.clone(), TextureOptions::NEAREST)
                });
                tex.set(img, TextureOptions::NEAREST);

                let (rect, resp) = ui.allocate_exact_size(
                    canvas_size,
                    Sense::click_and_drag(),
                );

                if resp.hovered() {
                    let scroll = ctx.input(|i| i.raw_scroll_delta.y);
                    if scroll != 0.0 {
                        self.zoom = (self.zoom - scroll * 2.0).clamp(2.0, 40.0);
                    }
                }

                if ui.is_rect_visible(rect) {
                    let p = ui.painter();
                    p.image(
                        tex.id(),
                        rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if self.grid {
                        let gc = Color32::from_black_alpha(40);
                        for x in 0..=self.width {
                            p.vline(
                                rect.min.x + x as f32 * self.zoom,
                                rect.y_range(),
                                Stroke::new(1.0, gc),
                            );
                        }
                        for y in 0..=self.height {
                            p.hline(
                                rect.x_range(),
                                rect.min.y + y as f32 * self.zoom,
                                Stroke::new(1.0, gc),
                            );
                        }
                    }
                }

                // --- draw LMB ---
                if resp.dragged_by(egui::PointerButton::Primary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        let px = self.screen_to_pixel(pos, rect.min);
                        if self.last_px_primary.is_none() {
                            self.push_undo();
                            self.paint_pixel(px.0, px.1);
                        } else if let Some(last) = self.last_px_primary {
                            self.draw_line(last, px);
                        }
                        self.last_px_primary = Some(px);
                    }
                }
                if resp.clicked_by(egui::PointerButton::Primary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        self.push_undo();
                        let px = self.screen_to_pixel(pos, rect.min);
                        self.paint_pixel(px.0, px.1);
                    }
                }

                // --- erase RMB ---
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        let old = self.color;
                        self.color = Color32::WHITE;
                        let px = self.screen_to_pixel(pos, rect.min);
                        if self.last_px_secondary.is_none() {
                            self.push_undo();
                            self.paint_pixel(px.0, px.1);
                        } else if let Some(last) = self.last_px_secondary {
                            self.draw_line(last, px);
                        }
                        self.last_px_secondary = Some(px);
                        self.color = old;
                    }
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        self.push_undo();
                        let old = self.color;
                        self.color = Color32::WHITE;
                        let px = self.screen_to_pixel(pos, rect.min);
                        self.paint_pixel(px.0, px.1);
                        self.color = old;
                    }
                }

                if resp.drag_stopped() {
                    self.last_px_primary = None;
                    self.last_px_secondary = None;
                }
            });
        });

        // --- resize dialog ---
        if self.show_resize {
            egui::Window::new("Resize Canvas")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ctx, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.resize_w, 1..=512).text("Width"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.resize_h, 1..=512).text("Height"),
                    );
                    ui.horizontal(|ui| {
                        if ui.button("Apply").clicked() {
                            if self.resize_w != self.width
                                || self.resize_h != self.height
                            {
                                self.resize_canvas(self.resize_w, self.resize_h);
                            }
                            self.show_resize = false;
                        }
                        if ui.button("Cancel").clicked() {
                            self.show_resize = false;
                        }
                    });
                });
        }
    }
}

fn main() -> eframe::Result {
    eframe::run_native(
        "Pixesh",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 700.0])
                .with_min_inner_size([400.0, 300.0]),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(PixeshApp::new()))),
    )
}
