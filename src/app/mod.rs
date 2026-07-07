pub mod canvas;
pub mod history;
pub mod input;
pub mod io;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::color::*;
use crate::constants::*;
use crate::ui::*;

// ── Layer / Snapshot ─────────────────────────────────
pub(crate) struct Layer {
    pub(crate) name: String,
    pub(crate) pixels: Vec<Color32>,
    pub(crate) visible: bool,
}

pub(crate) struct Snapshot {
    pub(crate) layers: Vec<Vec<Color32>>,
    pub(crate) active: usize,
}

// ── App ──────────────────────────────────────────────
pub struct PixeshApp {
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) color: Color32,
    pub(crate) hsv_h: f32,
    pub(crate) hsv_s: f32,
    pub(crate) hsv_v: f32,
    pub(crate) rgb_r: f32,
    pub(crate) rgb_g: f32,
    pub(crate) rgb_b: f32,
    pub(crate) brush: f32,
    pub(crate) tool: Tool,
    pub(crate) tool_saved: Option<Tool>,
    pub(crate) last_px_primary: Option<(i32, i32)>,
    pub(crate) last_px_secondary: Option<(i32, i32)>,

    pub(crate) grid: bool,
    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) tex: Option<egui::TextureHandle>,
    pub(crate) brush_tex: Option<egui::TextureHandle>,
    pub(crate) eraser_tex: Option<egui::TextureHandle>,
    pub(crate) fill_tex: Option<egui::TextureHandle>,
    pub(crate) drop_tex: Option<egui::TextureHandle>,
    pub(crate) clear_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex_h: f32,

    pub(crate) undo_stack: Vec<Snapshot>,
    pub(crate) redo_stack: Vec<Snapshot>,

    pub(crate) show_resize: bool,
    pub(crate) resize_w: f32,
    pub(crate) resize_h: f32,

    pub(crate) show_export: bool,
    pub(crate) export_name: String,
    pub(crate) export_path: String,
}

impl PixeshApp {
    pub fn new() -> Self {
        Self {
            layers: vec![Layer {
                name: "Background".into(),
                pixels: vec![Color32::TRANSPARENT; 16 * 16],
                visible: true,
            }],
            active_layer: 0,
            width: 16,
            height: 16,
            color: Color32::BLACK,
            hsv_h: 0.0,
            hsv_s: 0.0,
            hsv_v: 0.0,
            rgb_r: 0.0,
            rgb_g: 0.0,
            rgb_b: 0.0,
            brush: 1.0,
            tool: Tool::Brush,
            tool_saved: None,
            last_px_primary: None,
            last_px_secondary: None,
            grid: false,
            zoom: 46.0,
            pan: Vec2::ZERO,
            tex: None,
            brush_tex: None,
            eraser_tex: None,
            fill_tex: None,
            drop_tex: None,
            clear_tex: None,
            sv_tex: None,
            sv_tex_h: -1.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_resize: false,
            resize_w: 64.0,
            resize_h: 64.0,
            show_export: false,
            export_name: "pixesh.png".into(),
            export_path: String::new(),
        }
    }
}

// ── eframe::App ──────────────────────────────────────
impl eframe::App for PixeshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);

        // ── toolbar ──────────────────────────────────
        egui::TopBottomPanel::top("tools")
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
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

                    separator(ui);

                    let brush_tex = self.brush_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/brush.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("brush_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, brush_tex.id(), self.tool == Tool::Brush) { self.tool = Tool::Brush; }
                    let eraser_tex = self.eraser_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/eraser.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("eraser_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, eraser_tex.id(), self.tool == Tool::Eraser) { self.tool = Tool::Eraser; }
                    let fill_tex = self.fill_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/fill.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("fill_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, fill_tex.id(), self.tool == Tool::Fill) { self.tool = Tool::Fill; }
                    let drop_tex = self.drop_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/drop.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("drop_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, drop_tex.id(), self.tool == Tool::Eyedropper) { self.tool = Tool::Eyedropper; }

                    separator(ui);

                    let clear_tex = self.clear_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/clear.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("clear_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, clear_tex.id(), false) {
                        self.push_undo();
                        for layer in &mut self.layers {
                            layer.pixels = vec![Color32::TRANSPARENT; self.width * self.height];
                        }
                    }

                    checkbox(ui, "Grid", &mut self.grid);

                    let display_z = 61.0 - self.zoom;
                    let mut dz = display_z;
                    if slider(ui, "Z", &mut dz, 1.0, 60.0) {
                        self.zoom = (61.0 - dz).clamp(1.0, 60.0);
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

                    let bg = if is_active { HOVER } else { PANEL };
                    ui.painter().rect_filled(rect, 0.0, bg);

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

                // ── HSV picker ───────────────────────────
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
                        let _ = ui.allocate_exact_size(Vec2::new(80.0, (ROW_H + 2.0) * 3.0), Sense::hover());
                    });
                });

                // SV field + H strip
                let avail = ui.available_size();
                let fsize = (avail.x - 24.0).min(avail.y).min(180.0).max(40.0);
                let strip_w = 14.0;
                ui.horizontal(|ui| {
                    // ── SV 2D field ──
                    let (rect, resp) = ui.allocate_exact_size(Vec2::splat(fsize), Sense::click_and_drag());

                    // rebuild texture when H changes
                    if self.sv_tex.is_none() || (self.sv_tex_h - self.hsv_h).abs() > 0.5 {
                        self.sv_tex_h = self.hsv_h;
                        let ts = 128;
                        let h = self.hsv_h;
                        let mut pix = Vec::with_capacity(ts * ts);
                        for y in 0..ts {
                            for x in 0..ts {
                                let s = x as f32 / (ts - 1) as f32 * 255.0;
                                let v = y as f32 / (ts - 1) as f32 * 255.0;
                                let (r, g, b) = hsv_to_rgb(h, s, v);
                                pix.push(Color32::from_rgb(r, g, b));
                            }
                        }
                        let img = ColorImage { size: [ts, ts], pixels: pix };
                        self.sv_tex = Some(ui.ctx().load_texture("sv", img, egui::TextureOptions::LINEAR));
                    }

                    if let Some(tex) = &self.sv_tex {
                        let p = ui.painter();
                        p.image(tex.id(), rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                        p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                        let cx = rect.min.x + (self.hsv_s / 255.0) * rect.width();
                        let cy = rect.min.y + (self.hsv_v / 255.0) * rect.height();
                        let cc = if self.hsv_v > 180.0 { Color32::BLACK } else { Color32::WHITE };
                        p.circle_stroke(Pos2::new(cx, cy), 4.0, Stroke::new(1.5, cc));
                        p.circle_filled(Pos2::new(cx, cy), 2.0, cc);
                    }

                    let pick = resp.dragged_by(egui::PointerButton::Primary)
                        || resp.clicked_by(egui::PointerButton::Primary);
                    if pick {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            let rel = pos - rect.min;
                            self.hsv_s = (rel.x / rect.width() * 255.0).clamp(0.0, 255.0);
                            self.hsv_v = (rel.y / rect.height() * 255.0).clamp(0.0, 255.0);
                            let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                            self.rgb_r = r as f32;
                            self.rgb_g = g as f32;
                            self.rgb_b = b as f32;
                            self.color = Color32::from_rgb(r, g, b);
                        }
                    }

                    // ── H strip ──
                    let (srect, sresp) = ui.allocate_exact_size(Vec2::new(strip_w, fsize), Sense::click_and_drag());

                    let ts = 64;
                    let mut spix = Vec::with_capacity(ts);
                    for y in 0..ts {
                        let hh = y as f32 / (ts - 1) as f32 * 360.0;
                        let (r, g, b) = hsv_to_rgb(hh, 255.0, 255.0);
                        spix.push(Color32::from_rgb(r, g, b));
                    }
                    let simg = ColorImage { size: [1, ts], pixels: spix };
                    let stex = ui.ctx().load_texture("hstrip", simg, egui::TextureOptions::LINEAR);
                    let sp = ui.painter();
                    sp.image(stex.id(), srect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
                    sp.rect_stroke(srect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

                    let hy = srect.min.y + (self.hsv_h / 360.0) * srect.height();
                    sp.hline(srect.x_range(), hy, Stroke::new(2.0, Color32::WHITE));

                    let spick = sresp.dragged_by(egui::PointerButton::Primary)
                        || sresp.clicked_by(egui::PointerButton::Primary);
                    if spick {
                        if let Some(pos) = sresp.interact_pointer_pos() {
                            let rel_y = (pos.y - srect.min.y) / srect.height();
                            self.hsv_h = (rel_y * 360.0).clamp(0.0, 359.99);
                            let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                            self.rgb_r = r as f32;
                            self.rgb_g = g as f32;
                            self.rgb_b = b as f32;
                            self.color = Color32::from_rgb(r, g, b);
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

                    // brush cursor
                    let cursor = resp.interact_pointer_pos()
                        .or_else(|| resp.hover_pos());
                    if let Some(pos) = cursor {
                        if canvas_rect.contains(pos) {
                            let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                            let b = self.brush_i() as i32;
                            let half = (b - 1) / 2;
                            let bx0 = (px - half).max(0) as f32;
                            let by0 = (py - half).max(0) as f32;
                            let bx1 = (px - half + b).min(self.width as i32) as f32;
                            let by1 = (py - half + b).min(self.height as i32) as f32;
                            let cr = Rect::from_min_size(
                                Pos2::new(
                                    canvas_rect.min.x + bx0 * self.zoom,
                                    canvas_rect.min.y + by0 * self.zoom,
                                ),
                                Vec2::new(
                                    (bx1 - bx0) * self.zoom,
                                    (by1 - by0) * self.zoom,
                                ),
                            );
                            p.rect_filled(cr, 0.0, Color32::from_black_alpha(60));
                            p.rect_stroke(cr, 0.0, Stroke::new(1.0, Color32::WHITE.linear_multiply(0.4)), egui::StrokeKind::Inside);
                        }
                    }

                }

                // LMB
                if self.tool == Tool::Eyedropper {
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                                    let c = self.composite()[(py * self.width as i32 + px) as usize];
                                    self.color = c;
                                    self.rgb_r = c.r() as f32;
                                    self.rgb_g = c.g() as f32;
                                    self.rgb_b = c.b() as f32;
                                    let (h, s, v) = rgb_to_hsv(c.r(), c.g(), c.b());
                                    self.hsv_h = h;
                                    self.hsv_s = s;
                                    self.hsv_v = v;
                                }
                            }
                        }
                    }
                } else if self.tool == Tool::Fill {
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                self.push_undo();
                                let (px, py) = self.screen_to_pixel(pos, canvas_rect.min);
                                self.flood_fill(px, py, self.color);
                            }
                        }
                    }
                } else {
                    let paint_color = if self.tool == Tool::Eraser { Color32::TRANSPARENT } else { self.color };
                    // paint on initial press (before drag threshold)
                    if self.last_px_primary.is_none() {
                        let pressed = ctx.input(|i| i.pointer.primary_down());
                        if pressed {
                            if let Some(pos) = resp.interact_pointer_pos() {
                                if canvas_rect.contains(pos) {
                                    let px = self.screen_to_pixel(pos, canvas_rect.min);
                                    self.push_undo();
                                    self.paint_pixel(px.0, px.1, paint_color);
                                    self.last_px_primary = Some(px);
                                }
                            }
                        }
                    }
                    if resp.dragged_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                let px = self.screen_to_pixel(pos, canvas_rect.min);
                                self.paint_pixel(px.0, px.1, paint_color);
                                self.last_px_primary = Some(px);
                            }
                        }
                    }
                    if resp.clicked_by(egui::PointerButton::Primary) {
                        if let Some(pos) = resp.interact_pointer_pos() {
                            if canvas_rect.contains(pos) {
                                self.push_undo();
                                let px = self.screen_to_pixel(pos, canvas_rect.min);
                                self.paint_pixel(px.0, px.1, paint_color);
                            }
                        }
                    }
                }

                // RMB always erases (transparent)
                if resp.dragged_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            if self.last_px_secondary.is_none() {
                                self.push_undo();
                            }
                            self.paint_pixel(px.0, px.1, Color32::TRANSPARENT);
                            self.last_px_secondary = Some(px);
                        }
                    }
                }
                if resp.clicked_by(egui::PointerButton::Secondary) {
                    if let Some(pos) = resp.interact_pointer_pos() {
                        if canvas_rect.contains(pos) {
                            self.push_undo();
                            let px = self.screen_to_pixel(pos, canvas_rect.min);
                            self.paint_pixel(px.0, px.1, Color32::TRANSPARENT);
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
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(28.0));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("W:");
                        let mut w = self.resize_w as i32;
                        ui.add_sized(Vec2::new(100.0, 32.0), egui::DragValue::new(&mut w).range(1..=4096));
                        self.resize_w = w as f32;
                    });
                    ui.horizontal(|ui| {
                        ui.label("H:");
                        let mut h = self.resize_h as i32;
                        ui.add_sized(Vec2::new(100.0, 32.0), egui::DragValue::new(&mut h).range(1..=4096));
                        self.resize_h = h as f32;
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if btn(ui, "Apply") {
                            if self.resize_w as usize != self.width
                                || self.resize_h as usize != self.height
                            {
                                self.resize_canvas(self.resize_w as usize, self.resize_h as usize);
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
                .resizable(false)
                .collapsible(false)
                .frame(egui::Frame::new().fill(PANEL).stroke(Stroke::new(2.0, BORDER)))
                .show(ctx, |ui| {
                    ui.style_mut().override_font_id = Some(egui::FontId::proportional(28.0));
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        ui.label("Folder:");
                        let display = if self.export_path.is_empty() { "." } else { &self.export_path };
                        ui.add_sized(Vec2::new(200.0, 32.0), egui::Label::new(display));
                        if btn(ui, "…") {
                            let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                            if let Some(p) = rfd::FileDialog::new()
                                .set_directory(&home)
                                .pick_folder()
                            {
                                self.export_path = p.to_string_lossy().into();
                            }
                        }
                    });
                    ui.horizontal(|ui| {
                        ui.label("File:");
                        ui.add_sized(
                            Vec2::new(200.0, 32.0),
                            egui::TextEdit::singleline(&mut self.export_name)
                                .font(egui::TextStyle::Body),
                        );
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if btn(ui, "Save") {
                            let path = if self.export_path.is_empty() {
                                self.export_name.clone()
                            } else {
                                format!("{}/{}", self.export_path, self.export_name)
                            };
                            self.save_png(&path);
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
