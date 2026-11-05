use eframe::egui::{Color32, Id};

// Available editing tools
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Tool { Brush, Eraser, Fill, Eyedropper, Select, Move, Text, Transform, Effects }

// Background type for PNG export
#[derive(PartialEq, Clone, Copy)]
pub enum ExportBg { Transparent, White, Black, Checker }

// Brush shape / mask
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BrushShape { Round, Square }

// ── Panel layout ──

/// Which edge of the window a panel occupies. Panels can be dragged between edges,
/// but not to any edge: for example, the timeline strip does not make sense on a side edge.
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Edge { Top, Bottom, Left, Right }

impl Edge {
    pub fn is_vertical(self) -> bool {
        matches!(self, Edge::Left | Edge::Right)
    }

    /// Render group ordering.
    /// The top panel (Toolbar) is rendered first to stretch across full window width.
    /// Side panels (Layers) are rendered next, before bottom panels (Status, Timeline),
    /// so that Status and Timeline do not clip under or overlap the Layers side panel.
    pub fn draw_group(self) -> usize {
        match self {
            Edge::Top => 0,
            Edge::Left => 1,
            Edge::Right => 2,
            Edge::Bottom => 3,
        }
    }

    /// Edges to which a panel is allowed to be dragged. Toolbar, status bar, and
    /// timeline are horizontal strips, while the layers panel is a vertical list.
    pub fn allowed_for(self) -> &'static [Edge] {
        const H: &[Edge] = &[Edge::Top, Edge::Bottom];
        const V: &[Edge] = &[Edge::Left, Edge::Right];
        if self.is_vertical() { V } else { H }
    }
}

/// Identifies draggable panels. The enum variant order defines the canonical
/// draw order within the same edge — it remains stable so panels on the same edge
/// do not jump around when moved.
#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum PanelId { Toolbar, Status, Timeline, Layers }

/// Ordering of a panel within an edge. Used with `sort_by_key`.
pub(crate) fn p_order(p: PanelId) -> usize {
    match p {
        PanelId::Toolbar => 0,
        PanelId::Status => 1,
        PanelId::Timeline => 2,
        PanelId::Layers => 3,
    }
}

impl PanelId {
    /// Default edge. Panels on the same edge are rendered in this order:
    /// the first rendered one sits closest to the outer window edge.
    pub fn default_edge(self) -> Edge {
        match self {
            PanelId::Toolbar => Edge::Top,
            PanelId::Status => Edge::Bottom,
            PanelId::Timeline => Edge::Bottom,
            PanelId::Layers => Edge::Right,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PanelId::Toolbar => "Toolbar",
            PanelId::Status => "Status",
            PanelId::Timeline => "Timeline",
            PanelId::Layers => "Layers",
        }
    }

    /// Inherent shape edge used to determine whether the panel is horizontal or vertical.
    pub fn shape_edge(self) -> Edge {
        self.default_edge()
    }

    /// Stable Id for egui. The edge is intentionally included in the Id:
    /// egui stores PanelState by Id, reading width for side panels and height
    /// for top/bottom panels. Distinguishing by edge prevents size corruption
    /// when dragging panels across edges.
    pub fn ui_id(self, edge: Edge) -> Id {
        Id::new((self.name(), format!("{:?}", edge)))
    }
}

// ── Theme colors ──
pub const BG: Color32 = Color32::from_rgb(24, 24, 24);          // Darkest (canvas background)
pub const PANEL: Color32 = Color32::from_rgb(32, 32, 32);       // Panels background
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(44, 44, 44); // Lighter panel elements
pub const BORDER: Color32 = Color32::from_rgb(80, 80, 80);      // Widget borders
pub const TEXT: Color32 = Color32::from_rgb(220, 220, 220);      // Text color
pub const ACCENT: Color32 = Color32::from_rgb(200, 120, 60);     // Accent / selection
pub const HOVER: Color32 = Color32::from_rgb(60, 60, 60);       // Hover state
pub const DIM: Color32 = Color32::from_rgb(120, 120, 120);      // Dimmed text

// ── Numeric key palette ──
/// Preset colors mapped to keys 1–9. Index plus one corresponds to the key.
/// Full opacity (alpha 255) is preserved; opacity is modified independently.
///
/// Order: RGB channels, then combinations, then neutrals — putting both colors
/// and stroke/fill tones within immediate reach.
pub const NUM_PALETTE: [[u8; 4]; 9] = [
    [255, 0, 0, 255],     // 1 Red
    [0, 255, 0, 255],     // 2 Green
    [0, 0, 255, 255],     // 3 Blue
    [255, 255, 0, 255],   // 4 Yellow
    [0, 255, 255, 255],   // 5 Cyan
    [255, 0, 255, 255],   // 6 Magenta
    [255, 255, 255, 255], // 7 White
    [128, 128, 128, 255], // 8 Gray
    [0, 0, 0, 255],       // 9 Black
];

// ── Dimensions ──
pub const FONT_SZ: f32 = 20.0;    // Font size in pixels
pub const CHAR_W: f32 = 11.0;     // Approximate character width for layout calculations
pub const ROW_H: f32 = 22.0;      // Row height in pixels
pub const PANEL_PAD: f32 = 12.0;   // Panel padding
pub const CHECKER_PX: f32 = 8.0;  // Checkerboard square size on screen for transparent background

/// Maximum canvas side dimension in pixels (matches DragValue limit in dialogs).
pub const MAX_SIDE: i32 = 4096;

/// Returns a (W, H) pair where the edited side is matched to the other side
/// preserving the aspect ratio of the original canvas.
///
/// Anchored against `orig_w`/`orig_h` rather than current `w`/`h` to prevent drift.
pub fn linked_side(orig_w: i32, orig_h: i32, w: i32, h: i32, edited_w: bool) -> (i32, i32) {
    let clamp = |v: i32| v.clamp(1, MAX_SIDE);
    if edited_w {
        if orig_w <= 0 { return (clamp(w), clamp(h)); }
        (clamp(w), clamp(((w as f64 * orig_h as f64 / orig_w as f64).round()) as i32))
    } else {
        if orig_h <= 0 { return (clamp(w), clamp(h)); }
        (clamp(((h as f64 * orig_w as f64 / orig_h as f64).round()) as i32), clamp(h))
    }
}

#[cfg(test)]
mod linked_tests {
    use super::*;

    #[test]
    fn width_edit_keeps_aspect() {
        // 100x50 -> width 200, height must become 100
        assert_eq!(linked_side(100, 50, 200, 50, true), (200, 100));
    }

    #[test]
    fn height_edit_keeps_aspect() {
        // 100x50: ratio is 2, so H=200 is 4x and width must be 400
        assert_eq!(linked_side(100, 50, 100, 200, false), (400, 200));
        // H=100 is exactly double
        assert_eq!(linked_side(100, 50, 100, 100, false), (200, 100));
    }

    #[test]
    fn square_canvas_is_unchanged() {
        assert_eq!(linked_side(64, 64, 128, 64, true), (128, 128));
        assert_eq!(linked_side(64, 64, 64, 32, false), (32, 32));
    }

    /// Key property: successive edits do not drift from the original aspect ratio.
    /// Previously calculating from displayed values caused 101 instead of 100 on second step.
    #[test]
    fn successive_edits_do_not_drift() {
        let (ow, oh) = (64, 48);
        let (w, h) = linked_side(ow, oh, 100, 48, true);
        assert_eq!((w, h), (100, 75));
        let (w, h) = linked_side(ow, oh, w, 76, false);
        assert_eq!((w, h), (101, 76), "width calculated from anchor, not from 100");
        let (w, h) = linked_side(ow, oh, 202, h, true);
        assert_eq!((w, h), (202, 152));
    }

    /// Result ratio matches original ratio within one pixel rounding.
    /// Cases hitting MAX_SIDE 4096 are skipped here and tested in clamp test.
    #[test]
    fn result_stays_within_one_pixel_of_the_original_ratio() {
        for (ow, oh) in [(64, 48), (100, 50), (37, 91), (16, 16), (5, 3)] {
            for target in [1, 7, 33, 64, 200, 4096] {
                let expect = target as f64 * oh as f64 / ow as f64;
                if !(1.0..=MAX_SIDE as f64).contains(&expect.round()) { continue; }
                let (w, h) = linked_side(ow, oh, target, oh, true);
                assert_eq!(w, target, "{ow}x{oh}, width must stay untouched");
                assert!(
                    (h as f64 - expect).abs() <= 0.5,
                    "{ow}x{oh} -> {w}x{h}, expected ~{expect}"
                );
            }
        }
    }

    #[test]
    fn values_are_clamped_to_the_dialog_range() {
        assert_eq!(linked_side(1, 4096, 4096, 4096, true).1, 4096);
        // tiny height and huge width must not zero out the height
        let (_, h) = linked_side(4096, 1, 4096, 1, true);
        assert_eq!(h, 1);
        let (w, _) = linked_side(1, 4096, 1, 4096, false);
        assert_eq!(w, 1);
        // narrow canvas at large width clamps to MAX_SIDE
        assert_eq!(linked_side(37, 91, 4096, 91, true).1, MAX_SIDE);
    }

    /// Degenerate anchor avoids division by zero.
    #[test]
    fn degenerate_anchor_passes_values_through() {
        assert_eq!(linked_side(0, 10, 5, 10, true), (5, 10));
        assert_eq!(linked_side(10, 0, 10, 5, false), (10, 5));
    }
}
