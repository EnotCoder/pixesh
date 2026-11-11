//! Application state and behaviour.

pub(crate) mod actions;
pub(crate) mod color;
pub(crate) mod layout;
pub(crate) mod prefs;
pub(crate) mod tabs;
pub(crate) mod update;

#[cfg(all(test, feature = "persistence"))]
mod tests;

use eframe::egui::{self, Color32};

use crate::app::document::Document;
use crate::app::effects;
use crate::constants::*;

// ── App ──────────────────────────────────────────────
pub struct PixeshApp {
    pub(crate) docs: Vec<Document>,
    pub(crate) active_tab: usize,
    /// Last tab scrolled into view, so the tab bar only auto-scrolls when the
    /// active tab actually changes instead of fighting manual scrolling.
    pub(crate) tab_bar_last_active: Option<usize>,
    /// Visible canvas area from the previous frame. Wheel zoom only applies
    /// here, so scrolling a panel does not also zoom the canvas.
    pub(crate) canvas_rect: egui::Rect,

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
    pub(crate) round_brush_tex: Option<egui::TextureHandle>,
    pub(crate) square_brush_tex: Option<egui::TextureHandle>,
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
    /// Keep the aspect ratio: entering W also changes H and vice versa.
    /// Disabled by default, otherwise editing one side would silently
    /// overwrite the other.
    pub(crate) scale_lock_ratio: bool,
    pub(crate) resize_lock_ratio: bool,
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
    /// Falling-snow effect over the canvas.
    pub(crate) show_snow: bool,

    /// Window edge for each draggable panel
    pub(crate) toolbar_edge: Edge,
    pub(crate) layers_edge: Edge,
    pub(crate) status_edge: Edge,
    pub(crate) timeline_edge: Edge,

    // ── dragging panels between edges ──
    /// Edge the cursor is currently hovering over during a move
    pub(crate) drop_edge: Option<Edge>,
    /// Panel currently being dragged
    pub(crate) dragging_panel: Option<PanelId>,
    /// Where the drag started — the move is cancelled if dropped in the same place
    pub(crate) drag_from_edge: Edge,
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

    /// Transient status-bar hint (clipboard, selection, etc.)
    pub(crate) status_hint: String,
    pub(crate) status_hint_until: f64,
    /// Time of the last frame — lets hint() expire a hint without access to ctx
    pub(crate) last_time: f64,

    // ── effects panel (ports from Pixelorama) ──
    pub(crate) show_effects: bool,
    pub(crate) effect: effects::EffectParams,
    /// how many layers/frames the effect applies to
    pub(crate) effect_all_layers: bool,
    pub(crate) effect_all_frames: bool,
    /// preview cache: parameter signature + texture
    pub(crate) effect_prev_sig: String,
    pub(crate) effect_prev_tex: Option<egui::TextureHandle>,
    pub(crate) effect_prev_px: Vec<Color32>,
    pub(crate) effect_prev_size: (usize, usize),
    pub(crate) effect_prev_time: f64,

    pub(crate) cursor_px: Option<(i32, i32)>,
    pub(crate) close_handled: bool,

    pub(crate) logo_easter_egg: f32,

    pub(crate) snow: crate::app::snow::Snowfall,
}

impl PixeshApp {
    pub fn new() -> Self {
        // preference defaults live in one place — Prefs — otherwise "what was
        // there on first launch" and "what is in the file" drift apart
        let prefs = Self::initial_prefs();
        Self {
            docs: vec![Document::new_logo("Untitled")],
            active_tab: 0,
            tab_bar_last_active: None,
            canvas_rect: egui::Rect::NOTHING,
            color: Color32::BLACK,
            hsv_h: 0.0, hsv_s: 0.0, hsv_v: 0.0,
            rgb_r: 0.0, rgb_g: 0.0, rgb_b: 0.0, rgb_a: 255.0,
            brush: prefs.brush,
            brush_shape: prefs.brush_shape,
            tool: prefs.tool,
            tool_saved: None,
            tool_saved_shift: None,
            brush_tex: None, eraser_tex: None, fill_tex: None,
            drop_tex: None, clear_tex: None, logo_tex: None,
            round_brush_tex: None, square_brush_tex: None,
            sv_tex: None, sv_tex_h: -1.0,
            select_tex: None, move_tex: None, h_tex: None,
            mirror_h_tex: None, mirror_v_tex: None,
            plus_layer_tex: None, minus_layer_tex: None,
            clone_layer_tex: None, set_all_tex: None,
            tab_close_tex: None, tab_plus_tex: None,
            show_resize: false, resize_w: 64.0, resize_h: 64.0,
            show_scale: false, scale_w: 64.0, scale_h: 64.0,
            scale_lock_ratio: false, resize_lock_ratio: false,
            show_export: false, export_scale: 1, export_bg: ExportBg::Transparent,
            export_layers: false, export_sheet: false,
            show_panels: false, show_settings: false,
            show_top_panel: prefs.show_top_panel,
            show_right_panel: prefs.show_right_panel,
            show_status_bar: prefs.show_status_bar,
            show_timeline: prefs.show_timeline,
            show_snow: prefs.show_snow,
            toolbar_edge: prefs.toolbar_edge,
            layers_edge: prefs.layers_edge,
            status_edge: prefs.status_edge,
            timeline_edge: prefs.timeline_edge,
            drop_edge: None,
            dragging_panel: None,
            drag_from_edge: Edge::Top,
            show_quit_dialog: false,
            welcome_show_again: prefs.welcome_show_again,
            show_welcome: prefs.welcome_show_again,
            arrow_speed: prefs.arrow_speed,
            zoom_speed: prefs.zoom_speed,
            color_history: Vec::new(),
            renaming_layer: None, rename_buf: String::new(),
            status_hint: String::new(),
            status_hint_until: 0.0,
            last_time: 0.0,
            show_text: false,
            text_cursor: None,
            text_buffer: String::new(),
            text_scale: 2,
            show_effects: false,
            effect: effects::EffectParams::new(effects::EffectKind::OffsetScale),
            effect_all_layers: false,
            effect_all_frames: false,
            effect_prev_sig: String::new(),
            effect_prev_tex: None,
            effect_prev_px: Vec::new(),
            effect_prev_size: (0, 0),
            effect_prev_time: 0.0,
            cursor_px: None,
            close_handled: false,
            logo_easter_egg: 0.0,
            snow: crate::app::snow::Snowfall::default(),
        }
    }
}
