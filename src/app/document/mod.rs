//! Document data model and per-image operations.

pub(crate) mod anim;
pub(crate) mod canvas;
pub(crate) mod history;
pub(crate) mod io;
pub(crate) mod text;
pub(crate) mod tools;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Vec2};

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

/// Default canvas side. Matches the size of `img/pixel_logo.png` so the
/// mascot lands on the canvas one-to-one, without any downscaling.
pub(crate) const MASCOT_SIDE: usize = 32;

/// Owl pixels from `img/pixel_logo.png`, resized to the startup canvas size.
///
/// The source is exactly the same size as the canvas, so `resize` is just a
/// copy here. The nearest filter is deliberate: if the sizes ever diverge,
/// nearest keeps the owl's pixel-art look, whereas smoothing on a noticeable
/// downscale would turn the mascot into a mottled blob.
///
/// `None` if the image fails to decode: the canvas simply stays empty and the
/// application does not crash.
fn logo_pixels(w: usize, h: usize) -> Option<Vec<Color32>> {
    let img = image::load_from_memory(include_bytes!("../../../img/pixel_logo.png"))
        .ok()?
        .to_rgba8();
    let small = image::imageops::resize(
        &img,
        w as u32,
        h as u32,
        image::imageops::FilterType::Nearest,
    );
    Some(
        small
            .pixels()
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect(),
    )
}

impl Document {
    pub(crate) fn new_sized(name: &str, w: usize, h: usize) -> Self {
        let mut doc = Document::new(name);
        doc.width = w;
        doc.height = h;
        doc.layers[0].cels = vec![Arc::new(vec![Color32::TRANSPARENT; w * h])];
        doc
    }

    /// Default document: a clean 32×32 canvas filled with the owl from
    /// `img/pixel_logo.png`. This exists so that on startup and when the last
    /// tab is closed the canvas does not look like an empty grey square.
    pub(crate) fn new_logo(name: &str) -> Self {
        let mut doc = Document::new_sized(name, MASCOT_SIDE, MASCOT_SIDE);
        // The length is checked in place: if the sizes ever diverge from the
        // image size, the document stays empty instead of panicking on an index.
        if let Some(px) = logo_pixels(doc.width, doc.height)
            .filter(|px| px.len() == doc.width * doc.height)
        {
            doc.layers[0].cels[0] = Arc::new(px);
        }
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
