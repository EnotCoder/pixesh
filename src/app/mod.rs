pub mod canvas;
pub mod history;
pub mod input;
pub mod io;
pub mod panel_canvas;
pub mod panel_dialogs;
pub mod panel_layers;
pub mod panel_toolbar;

use std::sync::Arc;

use eframe::egui::{self, Color32, Vec2};

use crate::constants::Tool;

// ── Layer / Snapshot ─────────────────────────────────
pub(crate) struct Layer {
    pub(crate) name: String,
    pub(crate) pixels: Arc<Vec<Color32>>,
    pub(crate) visible: bool,
}

pub(crate) struct Snapshot {
    pub(crate) layers: Vec<Arc<Vec<Color32>>>,
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

    pub(crate) canvas_dirty: bool,

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
                pixels: Arc::new(vec![Color32::TRANSPARENT; 16 * 16]),
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
            canvas_dirty: true,
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
        self.ui_toolbar(ctx);
        self.ui_layers(ctx);
        self.ui_canvas(ctx);
        self.ui_dialogs(ctx);
    }
}
