use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

// ── Theme colours ────────────────────────────────────
const BG: Color32 = Color32::from_rgb(24, 24, 32);
const PANEL: Color32 = Color32::from_rgb(32, 32, 40);
const PANEL_LIGHT: Color32 = Color32::from_rgb(44, 44, 54);
const BORDER: Color32 = Color32::from_rgb(80, 80, 90);
const TEXT: Color32 = Color32::from_rgb(220, 220, 230);
const ACCENT: Color32 = Color32::from_rgb(200, 120, 60);
const HOVER: Color32 = Color32::from_rgb(60, 60, 72);
const FONT_SZ: f32 = 20.0;
const CHAR_W: f32 = 11.0;
const ROW_H: f32 = 22.0;

// ── Layer / Snapshot ─────────────────────────────────
struct Layer {
    name: String,
    pixels: Vec<Color32>,
    visible: bool,
}

struct Snapshot {
    layers: Vec<Vec<Color32>>,
    active: usize,
}

// ── App ──────────────────────────────────────────────
struct PixeshApp {
    layers: Vec<Layer>,
    active_layer: usize,
    width: usize,
    height: usize,

    color: Color32,
    rgb_r: f32,
    rgb_g: f32,
    rgb_b: f32,
    brush: f32,
    last_px_primary: Option<(i32, i32)>,
    last_px_secondary: Option<(i32, i32)>,

    grid: bool,
    zoom: f32,
    pan: Vec2,
    tex: Option<egui::TextureHandle>,
    rg_tex: Option<egui::TextureHandle>,
    rg_tex_b: f32,

    undo_stack: Vec<Snapshot>,
    redo_stack: Vec<Snapshot>,

    show_resize: bool,
    resize_w: usize,
    resize_h: usize,

    show_export: bool,
    export_name: String,
}

impl PixeshApp {
    fn new() -> Self {
        Self {
            layers: vec![Layer {
                name: "Background".into(),
                pixels: vec![Color32::WHITE; 16 * 16],
                visible: true,
            }],
            active_layer: 0,
            width: 16,
            height: 16,
            color: Color32::BLACK,
            rgb_r: 0.0,
            rgb_g: 0.0,
            rgb_b: 0.0,
            brush: 1.0,
            last_px_primary: None,
            last_px_secondary: None,
            grid: true,
            zoom: 46.0,
            pan: Vec2::ZERO,
            tex: None,
            rg_tex: None,
            rg_tex_b: -1.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_resize: false,
            resize_w: 64,
            resize_h: 64,
            show_export: false,
            export_name: "pixesh.png".into(),
        }
    }

    fn brush_i(&self) -> usize {
        self.brush.round() as usize
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
        let b = self.brush_i() as i32;
        let half = (b - 1) / 2;
        let layer = &mut self.layers[idx];
        for dy in 0..b {
            for dx in 0..b {
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

// ── custom widget helpers ────────────────────────────

fn btn(ui: &mut egui::Ui, label: &str) -> bool {
    let label_w = label.len() as f32 * CHAR_W;
    let pad = Vec2::new(8.0, 4.0);
    let size = Vec2::new(label_w + pad.x * 2.0, ROW_H + pad.y * 2.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let bg = if resp.clicked() {
        ACCENT
    } else if resp.hovered() {
        HOVER
    } else {
        PANEL
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);
    p.text(rect.min + pad, egui::Align2::LEFT_TOP, label, egui::FontId::proportional(FONT_SZ), TEXT);

    resp.clicked()
}

fn checkbox(ui: &mut egui::Ui, label: &str, checked: &mut bool) {
    let cbs = 16.0;
    let total_h = ROW_H.max(cbs) + 4.0;
    let label_w = label.len() as f32 * CHAR_W;
    let total_w = cbs + 8.0 + label_w;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click());

    // checkbox square
    let cb_rect = Rect::from_min_size(
        Pos2::new(rect.min.x, rect.center().y - cbs * 0.5),
        Vec2::splat(cbs),
    );
    let p = ui.painter();
    p.rect_filled(cb_rect, 0.0, PANEL);
    p.rect_stroke(cb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    if *checked {
        let inner = cb_rect.shrink(3.0);
        p.rect_filled(inner, 0.0, ACCENT);
    }

    let cb_resp = ui.interact(cb_rect, egui::Id::new(label), Sense::click());
    if cb_resp.clicked() {
        *checked = !*checked;
    }

    let label_rect = Rect::from_min_max(
        Pos2::new(cb_rect.max.x + 4.0, rect.min.y),
        Pos2::new(rect.max.x, rect.max.y),
    );
    let lresp = ui.interact(label_rect, egui::Id::new(format!("{}_l", label)), Sense::click());
    if lresp.clicked() {
        *checked = !*checked;
    }

    p.text(
        Pos2::new(cb_rect.max.x + 6.0, rect.center().y - ROW_H * 0.5),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(FONT_SZ),
        TEXT,
    );
}

fn slider(ui: &mut egui::Ui, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
    let track_w = 80.0;
    let thumb_w = 10.0;
    let label_w = (label.len() as f32 * CHAR_W) + 50.0;
    let total_w = track_w + 8.0 + label_w;
    let total_h = ROW_H + 8.0;

    let mut changed = false;
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click_and_drag());
    let p = ui.painter();

    // label
    let label_str = format!("{}{}", label, *value as i32);
    p.text(
        Pos2::new(rect.max.x - label_w, rect.center().y - ROW_H * 0.5),
        egui::Align2::LEFT_TOP,
        &label_str,
        egui::FontId::proportional(FONT_SZ),
        TEXT,
    );

    // track
    let ty = rect.center().y - 4.0;
    let track_rect = Rect::from_min_size(
        Pos2::new(rect.min.x, ty),
        Vec2::new(track_w, 8.0),
    );
    p.rect_filled(track_rect, 0.0, Color32::from_gray(30));
    p.rect_stroke(track_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    // thumb
    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let thumb_x = track_rect.min.x + t * (track_w - thumb_w);
    let thumb_rect = Rect::from_min_size(
        Pos2::new(thumb_x, rect.center().y - 7.0),
        Vec2::new(thumb_w, 14.0),
    );
    let thumb_bg = if resp.dragged() || resp.hovered() {
        ACCENT
    } else {
        PANEL_LIGHT
    };
    p.rect_filled(thumb_rect, 0.0, thumb_bg);
    p.rect_stroke(thumb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    if resp.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t2 = ((pos.x - track_rect.min.x) / track_w).clamp(0.0, 1.0);
            *value = min + t2 * (max - min);
            changed = true;
        }
    }

    changed
}

fn separator(ui: &mut egui::Ui) {
    let h = ROW_H + 6.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(2.0, h), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, BORDER),
    );
}

// ── eframe::App ──────────────────────────────────────
impl eframe::App for PixeshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // keyboard
        ctx.input_mut(|i| {
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.undo();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
                self.redo();
            }
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                self.export_name = "pixesh.png".into();
                self.show_export = true;
            }
        });

        // zoom (anywhere in window)
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let old = self.zoom;
            self.zoom = (self.zoom - scroll * 0.2).clamp(1.0, 60.0);
            self.pan *= self.zoom / old;
        }

        // arrow pan
        ctx.input(|i| {
            let speed = if i.modifiers.shift { 80.0 } else { 20.0 };
            if i.key_down(egui::Key::ArrowLeft) {
                self.pan.x += speed;
            }
            if i.key_down(egui::Key::ArrowRight) {
                self.pan.x -= speed;
            }
            if i.key_down(egui::Key::ArrowUp) {
                self.pan.y += speed;
            }
            if i.key_down(egui::Key::ArrowDown) {
                self.pan.y -= speed;
            }
        });

        // ── toolbar ──────────────────────────────────
        egui::TopBottomPanel::top("tools")
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    // title
                    let title = "Pixesh";
                    let title_w = title.len() as f32 * CHAR_W;
                    let (tr, _) = ui.allocate_exact_size(
                        Vec2::new(title_w + 8.0, ROW_H + 4.0),
                        Sense::hover(),
                    );
                    ui.painter().text(
                        tr.min + Vec2::new(4.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        title,
                        egui::FontId::proportional(FONT_SZ),
                        ACCENT,
                    );

                    separator(ui);

                    slider(ui, "B", &mut self.brush, 1.0, 10.0);

                    if btn(ui, "Clear") {
                        self.push_undo();
                        for layer in &mut self.layers {
                            layer.pixels = if layer.name == "Background" {
                                vec![Color32::WHITE; self.width * self.height]
                            } else {
                                vec![Color32::TRANSPARENT; self.width * self.height]
                            };
                        }
                    }

                    checkbox(ui, "Grid", &mut self.grid);

                    // Z slider with inverted display (left=60=pixels, right=1=full canvas)
                    let display_z = 61.0 - self.zoom;
                    let mut dz = display_z;
                    if slider(ui, "Z", &mut dz, 1.0, 60.0) {
                        self.zoom = (61.0 - dz).clamp(1.0, 60.0);
                    }

                    separator(ui);

                    if btn(ui, "Save") {
                        self.export_name = "pixesh.png".into();
                        self.show_export = true;
                    }
                    if btn(ui, "Load") {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("PNG", &["png"])
                            .pick_file()
                        {
                            self.load_png(&path.to_string_lossy());
                        }
                    }
                    if btn(ui, "Resize") {
                        self.resize_w = self.width;
                        self.resize_h = self.height;
                        self.show_resize = true;
                    }

                    separator(ui);

                    if btn(ui, "Undo") {
                        self.undo();
                    }
                    if btn(ui, "Redo") {
                        self.redo();
                    }
                });
                ui.add_space(4.0);
            });

        // ── layers ───────────────────────────────────
        egui::SidePanel::right("layers")
            .resizable(true)
            .default_width(280.0)
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(4.0);

                let header = "Layers";
                let hdr_w = header.len() as f32 * CHAR_W;
                let (hdr, _) = ui.allocate_exact_size(
                    Vec2::new(hdr_w + 8.0, ROW_H + 4.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hdr.min + Vec2::new(4.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    header,
                    egui::FontId::proportional(FONT_SZ),
                    TEXT,
                );

                let n = self.layers.len();
                for i in (0..n).rev() {
                    let name = self.layers[i].name.clone();
                    let is_active = self.active_layer == i;
                    let cb = self.layers[i].visible;

                    let row_h = ROW_H + 6.0;
                    let (rect, resp) =
                        ui.allocate_exact_size(Vec2::new(ui.available_size().x, row_h), Sense::click());

                    // bg
                    let bg = if is_active { HOVER } else { PANEL };
                    ui.painter().rect_filled(rect, 0.0, bg);

                    // checkbox inline
                    let cbs = 12.0;
                    let cb_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + 4.0, rect.center().y - cbs * 0.5),
                        Vec2::splat(cbs),
                    );
                    let p = ui.painter();
                    p.rect_filled(cb_rect, 0.0, PANEL_LIGHT);
                    p.rect_stroke(cb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);
                    if cb {
                        let inner = cb_rect.shrink(2.0);
                        p.rect_filled(inner, 0.0, ACCENT);
                    }

                    let cb_resp =
                        ui.interact(cb_rect, egui::Id::new(("lc", i)), Sense::click());
                    if cb_resp.clicked() {
                        self.layers[i].visible = !self.layers[i].visible;
                    }

                    // name
                    p.text(
                        Pos2::new(cb_rect.max.x + 4.0, rect.min.y + 3.0),
                        egui::Align2::LEFT_TOP,
                        &name,
                        egui::FontId::proportional(FONT_SZ),
                        TEXT,
                    );

                    if resp.clicked() && !cb_resp.clicked() {
                        self.active_layer = i;
                    }
                }

                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    if btn(ui, "+") {
                        self.add_layer();
                    }
                    if btn(ui, "-") {
                        self.remove_layer(self.active_layer);
                    }
                });

                // ── Adobe-style RGB picker ──────────────
                ui.add_space(8.0);
                let hdr = "Color";
                let hw = hdr.len() as f32 * CHAR_W;
                let (hr, _) = ui.allocate_exact_size(
                    Vec2::new(hw + 8.0, ROW_H + 4.0),
                    Sense::hover(),
                );
                ui.painter().text(
                    hr.min + Vec2::new(4.0, 2.0),
                    egui::Align2::LEFT_TOP,
                    hdr,
                    egui::FontId::proportional(FONT_SZ),
                    TEXT,
                );

                // preview + RGB readout
                ui.horizontal(|ui| {
                    let ps = 36.0;
                    let (pr, _) = ui.allocate_exact_size(Vec2::new(ps, ps), Sense::hover());
                    let pc = Color32::from_rgb(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8);
                    ui.painter().rect_filled(pr, 0.0, pc);
                    ui.painter().rect_stroke(pr, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                    ui.vertical(|ui| {
                        let mut y = ui.cursor().min.y;
                        for (ch, &v) in [("R", &self.rgb_r), ("G", &self.rgb_g), ("B", &self.rgb_b)] {
                            let txt = format!("{} {}", ch, v as u8);
                            ui.painter().text(
                                Pos2::new(pr.max.x + 6.0, y),
                                egui::Align2::LEFT_TOP,
                                &txt,
                                egui::FontId::proportional(FONT_SZ),
                                TEXT,
                            );
                            y += ROW_H + 2.0;
                        }
                        // consume space
                        let _ = ui.allocate_exact_size(Vec2::new(80.0, (ROW_H + 2.0) * 3.0), Sense::hover());
                    });
                });

                // RG field + B strip
                let avail = ui.available_size();
                let fsize = (avail.x - 24.0).min(avail.y).min(180.0).max(40.0);
                let strip_w = 14.0;
                ui.horizontal(|ui| {
                    // ── RG 2D field ──
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(fsize), Sense::click_and_drag());

                    if self.rg_tex.is_none() || (self.rg_tex_b - self.rgb_b).abs() > 0.5 {
                        self.rg_tex_b = self.rgb_b;
                        let ts = 128;
                        let bb = self.rgb_b as u8;
                        let mut pix = Vec::with_capacity(ts * ts);
                        for y in 0..ts {
                            for x in 0..ts {
                                let rr = (x as f32 / (ts - 1) as f32 * 255.0) as u8;
                                let gg = (y as f32 / (ts - 1) as f32 * 255.0) as u8;
                                pix.push(Color32::from_rgb(rr, gg, bb));
                            }
                        }
                        let img = ColorImage { size: [ts, ts], pixels: pix };
                        self.rg_tex = Some(ui.ctx().load_texture("rg", img, egui::TextureOptions::LINEAR));
                    }

                    if let Some(tex) = &self.rg_tex {
                        let p = ui.painter();
                        p.image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                        p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                        let cx = rect.min.x + (self.rgb_r / 255.0) * rect.width();
                        let cy = rect.min.y + (self.rgb_g / 255.0) * rect.height();
                        let cc = if self.rgb_r > 180.0 || self.rgb_g > 180.0 { Color32::BLACK } else { Color32::WHITE };
                        p.circle_stroke(Pos2::new(cx, cy), 4.0, Stroke::new(1.5, cc));
                        p.circle_filled(Pos2::new(cx, cy), 2.0, cc);
                    }

                    let pick = resp.dragged_by(egui::PointerButton::Primary)
                        || resp.clicked_by(egui::PointerButton::Primary);
                    if pick {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            let rel = pos - rect.min;
                            self.rgb_r = (rel.x / rect.width() * 255.0).clamp(0.0, 255.0);
                            self.rgb_g = (rel.y / rect.height() * 255.0).clamp(0.0, 255.0);
                            self.color = Color32::from_rgb(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8);
                        }
                    }

                    // ── B strip ──
                    let (srect, sresp) = ui.allocate_exact_size(Vec2::new(strip_w, fsize), Sense::click_and_drag());

                    let ts = 64;
                    let r = self.rgb_r as u8;
                    let g = self.rgb_g as u8;
                    let mut spix = Vec::with_capacity(ts);
                    for y in 0..ts {
                        let bb = (y as f32 / (ts - 1) as f32 * 255.0) as u8;
                        spix.push(Color32::from_rgb(r, g, bb));
                    }
                    let simg = ColorImage { size: [1, ts], pixels: spix };
                    let stex = ui.ctx().load_texture("bstrip", simg, egui::TextureOptions::LINEAR);
                    let sp = ui.painter();
                    sp.image(stex.id(), srect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    sp.rect_stroke(srect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                    let by = srect.min.y + (self.rgb_b / 255.0) * srect.height();
                    let sc = if self.rgb_b > 180.0 { Color32::BLACK } else { Color32::WHITE };
                    sp.hline(srect.x_range(), by, Stroke::new(2.0, sc));

                    let spick = sresp.dragged_by(egui::PointerButton::Primary)
                        || sresp.clicked_by(egui::PointerButton::Primary);
                    if spick {
                        if let Some(pos) = sresp.interact_pointer_pos() {
                            let rel_y = (pos.y - srect.min.y) / srect.height();
                            self.rgb_b = (rel_y * 255.0).clamp(0.0, 255.0);
                            self.color = Color32::from_rgb(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8);
                        }
                    }
                });
            });

        // ── canvas ───────────────────────────────────
        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(BG))
            .show(ctx, |ui| {
                let canvas_size = Vec2::new(
                    self.width as f32 * self.zoom,
                    self.height as f32 * self.zoom,
                );
                let avail = ui.available_size();

                let (area, resp) = ui.allocate_exact_size(avail, Sense::click_and_drag());

                // clamp pan so canvas never fully leaves the screen
                let max_px = (canvas_size.x + area.width()) * 0.5;
                let max_py = (canvas_size.y + area.height()) * 0.5;
                self.pan = self.pan.clamp(
                    Vec2::new(-max_px, -max_py),
                    Vec2::new(max_px, max_py),
                );

                let canvas_rect = Rect::from_center_size(
                    area.center() + self.pan,
                    canvas_size,
                );

                if ui.is_rect_visible(canvas_rect) {
                    let p = ui.painter();

                    // checkerboard
                    let ck_a = Color32::from_gray(200);
                    let ck_b = Color32::from_gray(180);
                    if self.zoom > 4.0 {
                        for y in 0..self.height {
                            for x in 0..self.width {
                                let r2 = Rect::from_min_size(
                                    Pos2::new(
                                        canvas_rect.min.x + x as f32 * self.zoom,
                                        canvas_rect.min.y + y as f32 * self.zoom,
                                    ),
                                    Vec2::splat(self.zoom),
                                );
                                p.rect_filled(
                                    r2,
                                    0.0,
                                    if (x + y) % 2 == 0 { ck_a } else { ck_b },
                                );
                            }
                        }
                    }

                    let flat = self.composite();
                    let img = ColorImage {
                        size: [self.width, self.height],
                        pixels: flat,
                    };
                    let tex = self.tex.get_or_insert_with(|| {
                        ui.ctx().load_texture(
                            "canvas",
                            img.clone(),
                            egui::TextureOptions::NEAREST,
                        )
                    });
                    tex.set(img, egui::TextureOptions::NEAREST);

                    p.image(
                        tex.id(),
                        canvas_rect,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    if self.grid {
                        let gc = Color32::from_black_alpha(40);
                        for x in 0..=self.width {
                            p.vline(
                                canvas_rect.min.x + x as f32 * self.zoom,
                                canvas_rect.y_range(),
                                Stroke::new(1.0, gc),
                            );
                        }
                        for y in 0..=self.height {
                            p.hline(
                                canvas_rect.x_range(),
                                canvas_rect.min.y + y as f32 * self.zoom,
                                Stroke::new(1.0, gc),
                            );
                        }
                    }
                }

                // draw LMB
                if resp.dragged_by(egui::PointerButton::Primary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            if self.last_px_primary.is_none() {
                                self.push_undo();
                                self.paint_pixel(px.0, px.1);
                            } else if let Some(last) = self.last_px_primary {
                                self.draw_line(last, px);
                            }
                            self.last_px_primary = Some(px);
                        }
                    }
                }
                if resp.clicked_by(egui::PointerButton::Primary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            self.push_undo();
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            self.paint_pixel(px.0, px.1);
                        }
                    }
                }

                // erase RMB
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            let old = self.color;
                            self.color = Color32::WHITE;
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
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
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            self.push_undo();
                            let old = self.color;
                            self.color = Color32::WHITE;
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            self.paint_pixel(px.0, px.1);
                            self.color = old;
                        }
                    }
                }

                if resp.drag_stopped() {
                    self.last_px_primary = None;
                    self.last_px_secondary = None;
                }
            });

        // ── resize dialog ────────────────────────────
        if self.show_resize {
            egui::Window::new("Resize Canvas")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.add(
                        egui::Slider::new(&mut self.resize_w, 1..=512).text("Width"),
                    );
                    ui.add(
                        egui::Slider::new(&mut self.resize_h, 1..=512).text("Height"),
                    );
                    ui.horizontal(|ui| {
                        if btn(ui, "Apply") {
                            if self.resize_w != self.width
                                || self.resize_h != self.height
                            {
                                self.resize_canvas(self.resize_w, self.resize_h);
                            }
                            self.show_resize = false;
                        }
                        if btn(ui, "Cancel") {
                            self.show_resize = false;
                        }
                    });
                });
        }

        // ── export dialog ───────────────────────────────
        if self.show_export {
            egui::Window::new("Export PNG")
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.export_name)
                                .desired_width(200.0),
                        );
                    });
                    ui.horizontal(|ui| {
                        if btn(ui, "Save") {
                            self.save_png(&self.export_name);
                            self.show_export = false;
                        }
                        if btn(ui, "Cancel") {
                            self.show_export = false;
                        }
                    });
                });
        }
    }
}

// ── main ─────────────────────────────────────────────
fn main() -> eframe::Result {
    let font_data: &'static [u8] = include_bytes!("../font.otf");

    eframe::run_native(
        "Pixesh",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 700.0])
                .with_min_inner_size([400.0, 300.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            // font
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("pixelfont".into(), egui::FontData::from_static(font_data).into());
            for family in fonts.families.values_mut() {
                family.insert(0, "pixelfont".into());
            }
            cc.egui_ctx.set_fonts(fonts);

            // style
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals {
                dark_mode: true,
                override_text_color: Some(TEXT),
                window_fill: PANEL,
                panel_fill: PANEL,
                faint_bg_color: PANEL_LIGHT,
                extreme_bg_color: BG,
                ..Default::default()
            };
            style.spacing.item_spacing = Vec2::new(6.0, 4.0);
            style.spacing.button_padding = Vec2::new(4.0, 2.0);
            cc.egui_ctx.set_style(style);

            Ok(Box::new(PixeshApp::new()))
        }),
    )
}
