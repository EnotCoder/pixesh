pub mod anim;
pub mod canvas;
pub mod config;
pub mod history;
pub mod input;
pub mod io;
pub mod panel_canvas;
pub mod panel_dialogs;
pub mod panel_layers;
    pub mod panel_status;
    pub mod panel_timeline;
    pub mod panel_toolbar;
pub mod text;
pub mod tools;

use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Stroke, Vec2};

use crate::constants::*;
use crate::ui::btn_min_w;

// ── Layer / Snapshot ─────────────────────────────────
pub(crate) struct Layer {
    pub(crate) name: String,
    // one pixel buffer ("cel") per animation frame
    pub(crate) cels: Vec<Arc<Vec<Color32>>>,
    pub(crate) visible: bool,
}

pub(crate) struct SnapshotLayer {
    pub(crate) name: String,
    pub(crate) cels: Vec<Arc<Vec<Color32>>>,
    pub(crate) visible: bool,
}

pub(crate) struct Snapshot {
    pub(crate) layers: Vec<SnapshotLayer>,
    pub(crate) active: usize,
    pub(crate) frames: usize,
    pub(crate) active_frame: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) sel: Option<(i32, i32, i32, i32)>,
    pub(crate) sel_buffer: Option<Vec<Color32>>,
    pub(crate) sel_buf_w: usize,
    pub(crate) sel_buf_h: usize,
    pub(crate) pasting: bool,
    pub(crate) clipboard: Option<Vec<Color32>>,
    pub(crate) clip_w: usize,
    pub(crate) clip_h: usize,
}

// ── Document: per-image state ────────────────────────
pub(crate) struct Document {
    pub(crate) name: String,
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) frames: usize,
    pub(crate) active_frame: usize,
    pub(crate) playing: bool,
    pub(crate) fps: f32,
    pub(crate) last_play: f64,

    pub(crate) undo_stack: Vec<Snapshot>,
    pub(crate) redo_stack: Vec<Snapshot>,

    pub(crate) canvas_dirty: bool,
    pub(crate) display_cell: i32,
    pub(crate) display_buf: Vec<Color32>,
    pub(crate) tex: Option<egui::TextureHandle>,

    pub(crate) sel: Option<(i32, i32, i32, i32)>,
    pub(crate) sel_start: Option<(i32, i32)>,
    pub(crate) sel_end: Option<(i32, i32)>,
    pub(crate) sel_move_origin: Option<(i32, i32)>,
    pub(crate) sel_move_current: Option<(i32, i32)>,
    pub(crate) sel_buffer: Option<Vec<Color32>>,
    pub(crate) sel_buf_w: usize,
    pub(crate) sel_buf_h: usize,
    pub(crate) sel_tex: Option<egui::TextureHandle>,

    pub(crate) canvas_move_origin: Option<(i32, i32)>,
    pub(crate) canvas_move_current: Option<(i32, i32)>,

    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) grid: bool,

    pub(crate) last_px_primary: Option<(i32, i32)>,
    pub(crate) last_px_secondary: Option<(i32, i32)>,
    pub(crate) mid_pan_pos: Option<Pos2>,

    pub(crate) unsaved: bool,
    pub(crate) export_path: String,
    pub(crate) export_name: String,
    pub(crate) needs_zoom_fit: bool,

    pub(crate) clipboard: Option<Vec<Color32>>,
    pub(crate) clip_w: usize,
    pub(crate) clip_h: usize,
    pub(crate) pasting: bool,

    pub(crate) transforming: bool,
    pub(crate) transform_orig_rect: Option<(i32, i32, i32, i32)>,
    pub(crate) transform_corner: Option<usize>,
}

impl Document {
    pub(crate) fn new_sized(name: &str, w: usize, h: usize) -> Self {
        let mut doc = Document::new(name);
        doc.width = w;
        doc.height = h;
        doc.layers[0].cels = vec![Arc::new(vec![Color32::TRANSPARENT; w * h])];
        doc
    }

    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            layers: vec![Layer {
                name: "Background".into(),
                cels: vec![Arc::new(vec![Color32::TRANSPARENT; 16 * 16])],
                visible: true,
            }],
            active_layer: 0,
            width: 16,
            height: 16,
            frames: 1,
            active_frame: 0,
            playing: false,
            fps: 12.0,
            last_play: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            canvas_dirty: true,
            display_cell: 0,
            display_buf: Vec::new(),
            tex: None,
            sel: None,
            sel_start: None,
            sel_end: None,
            sel_move_origin: None,
            sel_move_current: None,
            sel_buffer: None,
            sel_buf_w: 0,
            sel_buf_h: 0,
            sel_tex: None,
            canvas_move_origin: None,
            canvas_move_current: None,
            zoom: 46.0,
            pan: Vec2::ZERO,
            grid: false,
            last_px_primary: None,
            last_px_secondary: None,
            mid_pan_pos: None,
            unsaved: false,
            export_path: String::new(),
            export_name: "pixesh.png".into(),
            needs_zoom_fit: true,
            clipboard: None,
            clip_w: 0,
            clip_h: 0,
            pasting: false,
            transforming: false,
            transform_orig_rect: None,
            transform_corner: None,
        }
    }
}

// ── App ──────────────────────────────────────────────
pub struct PixeshApp {
    pub(crate) docs: Vec<Document>,
    pub(crate) active_tab: usize,

    pub(crate) color: Color32,
    pub(crate) hsv_h: f32,
    pub(crate) hsv_s: f32,
    pub(crate) hsv_v: f32,
    pub(crate) rgb_r: f32,
    pub(crate) rgb_g: f32,
    pub(crate) rgb_b: f32,
    pub(crate) rgb_a: f32,
    pub(crate) brush: f32,
    pub(crate) brush_shape: BrushShape,
    pub(crate) tool: Tool,
    pub(crate) tool_saved: Option<Tool>,
    pub(crate) tool_saved_shift: Option<Tool>,

    pub(crate) brush_tex: Option<egui::TextureHandle>,
    pub(crate) eraser_tex: Option<egui::TextureHandle>,
    pub(crate) fill_tex: Option<egui::TextureHandle>,
    pub(crate) drop_tex: Option<egui::TextureHandle>,
    pub(crate) clear_tex: Option<egui::TextureHandle>,
    pub(crate) logo_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex_h: f32,
    pub(crate) select_tex: Option<egui::TextureHandle>,
    pub(crate) move_tex: Option<egui::TextureHandle>,
    pub(crate) h_tex: Option<egui::TextureHandle>,
    pub(crate) mirror_h_tex: Option<egui::TextureHandle>,
    pub(crate) mirror_v_tex: Option<egui::TextureHandle>,
    pub(crate) plus_layer_tex: Option<egui::TextureHandle>,
    pub(crate) minus_layer_tex: Option<egui::TextureHandle>,
    pub(crate) clone_layer_tex: Option<egui::TextureHandle>,
    pub(crate) set_all_tex: Option<egui::TextureHandle>,
    pub(crate) tab_close_tex: Option<egui::TextureHandle>,
    pub(crate) tab_plus_tex: Option<egui::TextureHandle>,

    pub(crate) show_resize: bool,
    pub(crate) resize_w: f32,
    pub(crate) resize_h: f32,
    pub(crate) show_scale: bool,
    pub(crate) scale_w: f32,
    pub(crate) scale_h: f32,
    pub(crate) show_export: bool,
    pub(crate) export_scale: i32,
    pub(crate) export_bg: ExportBg,
    pub(crate) export_layers: bool,
    pub(crate) export_sheet: bool,
    pub(crate) show_panels: bool,
    pub(crate) show_settings: bool,
    pub(crate) show_top_panel: bool,
    pub(crate) show_right_panel: bool,
    pub(crate) show_status_bar: bool,
    pub(crate) show_timeline: bool,
    pub(crate) show_quit_dialog: bool,
    pub(crate) show_welcome: bool,
    pub(crate) welcome_show_again: bool,

    pub(crate) arrow_speed: f32,
    pub(crate) zoom_speed: f32,

    pub(crate) color_history: Vec<Color32>,
    pub(crate) renaming_layer: Option<usize>,
    pub(crate) rename_buf: String,

    pub(crate) show_text: bool,
    pub(crate) text_cursor: Option<(i32, i32)>,
    pub(crate) text_buffer: String,
    pub(crate) text_scale: i32,

    pub(crate) cursor_px: Option<(i32, i32)>,
    pub(crate) close_handled: bool,
}

impl PixeshApp {
    pub fn new() -> Self {
        let welcome_show_again = config::load_welcome_show_again();
        Self {
            docs: vec![Document::new("Untitled")],
            active_tab: 0,
            color: Color32::BLACK,
            hsv_h: 0.0, hsv_s: 0.0, hsv_v: 0.0,
            rgb_r: 0.0, rgb_g: 0.0, rgb_b: 0.0, rgb_a: 255.0,
            brush: 1.0,
            brush_shape: BrushShape::Square,
            tool: Tool::Brush,
            tool_saved: None,
            tool_saved_shift: None,
            brush_tex: None, eraser_tex: None, fill_tex: None,
            drop_tex: None, clear_tex: None, logo_tex: None,
            sv_tex: None, sv_tex_h: -1.0,
            select_tex: None, move_tex: None, h_tex: None,
            mirror_h_tex: None, mirror_v_tex: None,
            plus_layer_tex: None, minus_layer_tex: None,
            clone_layer_tex: None, set_all_tex: None,
            tab_close_tex: None, tab_plus_tex: None,
            show_resize: false, resize_w: 64.0, resize_h: 64.0,
            show_scale: false, scale_w: 64.0, scale_h: 64.0,
            show_export: false, export_scale: 1, export_bg: ExportBg::Transparent,
            export_layers: false, export_sheet: false,
            show_panels: false, show_settings: false,
            show_top_panel: true, show_right_panel: true, show_status_bar: true, show_timeline: true,
            show_quit_dialog: false,
            welcome_show_again,
            show_welcome: welcome_show_again,
            arrow_speed: 5.0,             zoom_speed: 1.0,
            color_history: Vec::new(),
            renaming_layer: None, rename_buf: String::new(),
            show_text: false,
            text_cursor: None,
            text_buffer: String::new(),
            text_scale: 2,
            cursor_px: None,
            close_handled: false,
        }
    }
}

impl PixeshApp {
    pub(crate) fn dialog_open(&self) -> bool {
        self.show_resize || self.show_export
            || self.show_panels || self.show_settings || self.show_scale
            || self.show_quit_dialog || self.show_text || self.show_welcome
    }

    pub(crate) fn any_unsaved(&self) -> bool {
        self.docs.iter().any(|d| d.unsaved)
    }

    pub(crate) fn update_playback(&mut self, ctx: &egui::Context) {
        let t = ctx.input(|i| i.time);
        for doc in &mut self.docs {
            if doc.playing {
                let interval = 1.0 / (doc.fps as f64).max(0.1);
                if doc.last_play == 0.0 {
                    doc.last_play = t;
                } else if t - doc.last_play >= interval {
                    doc.last_play = t;
                    let next = (doc.active_frame + 1) % doc.frames;
                    doc.set_active_frame(next);
                    ctx.request_repaint();
                }
            } else {
                doc.last_play = 0.0;
            }
        }
    }

    pub(crate) fn close_tab(&mut self, idx: usize) {
        if self.docs.len() <= 1 {
            self.docs[0] = Document::new("Untitled");
            self.active_tab = 0;
            return;
        }
        self.docs.remove(idx);
        if self.active_tab >= self.docs.len() {
            self.active_tab = self.docs.len() - 1;
        } else if self.active_tab > idx {
            self.active_tab -= 1;
        }
    }
}

// ── eframe::App ──────────────────────────────────────
impl eframe::App for PixeshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        self.update_playback(ctx);
        if self.show_top_panel { self.ui_toolbar(ctx); }
        if self.show_right_panel { self.ui_layers(ctx); }
        if self.show_status_bar { self.ui_status(ctx); }
        if self.show_timeline { self.ui_timeline(ctx); }
        self.ui_canvas(ctx);
        self.ui_dialogs(ctx);

        if ctx.input(|i| i.viewport().close_requested()) && self.any_unsaved() && !self.show_quit_dialog && !self.close_handled {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_quit_dialog = true;
        }
        if !ctx.input(|i| i.viewport().close_requested()) {
            self.close_handled = false;
        }
        if self.show_quit_dialog {
            egui::Area::new("quit_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(260.0);
                    
                    let t = ui.ctx().animate_bool(ui.make_persistent_id("quit_anim"), self.show_quit_dialog);
                    let size = size * (0.8 + 0.2 * t);
                    
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let p = ui.painter();
                    
                    let _alpha = (t * 255.0) as u8;
                    let bg = PANEL.gamma_multiply(t);
                    let brd = BORDER.gamma_multiply(t);
                    
                    p.rect_filled(rect, 0.0, bg);
                    p.rect_stroke(rect, 0.0, Stroke::new(4.0, brd), egui::StrokeKind::Outside);
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::top_down(egui::Align::Center))
                            .max_rect(rect),
                    );
                    child.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(24.0),
                    );
                    child.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    child.add_space(24.0);
                    child.label(egui::RichText::new("Unsaved changes!").size(26.0).color(TEXT));
                    child.add_space(12.0);
                    child.label(egui::RichText::new("Quit anyway?").size(22.0).color(TEXT));
                    child.add_space(child.available_height() - 44.0);
                    let spacing = child.style().spacing.item_spacing.x;
                    let half = (child.available_width() - spacing) / 2.0;
                    child.horizontal(|ui| {
                        if btn_min_w(ui, "Quit", half) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if btn_min_w(ui, "Cancel", half) {
                            self.show_quit_dialog = false;
                            self.close_handled = true;
                        }
                    });
                });
        }
    }
}
