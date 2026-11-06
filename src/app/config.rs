//! Persisting user preferences across application launches.
//!
//! Panel sizes and window geometry are persisted by egui memory mechanism:
//! egui keeps them in `egui::Memory`, while eframe serializes the memory into app.ron.
//! This module manages custom application state that egui does not track.

use crate::constants::{BrushShape, Edge, Tool};

/// Initial directory for native file dialogs.
pub(crate) fn default_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("EXTERNAL_STORAGE")) // Android fallback
        .unwrap_or_else(|_| "/".into())
}

/// Key used in eframe storage (serialized as RON).
#[cfg_attr(not(feature = "persistence"), allow(dead_code))]
pub(crate) const PREFS_KEY: &str = "pixesh_prefs";

/// Preferences that persist across restarts.
///
/// `#[serde(default)]` ensures missing fields in older configuration files
/// fall back to defaults without breaking deserialization.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(crate) struct Prefs {
    pub show_top_panel: bool,
    pub show_right_panel: bool,
    pub show_status_bar: bool,
    pub show_timeline: bool,
    /// Edge assignment for draggable panels.
    pub toolbar_edge: Edge,
    pub layers_edge: Edge,
    pub status_edge: Edge,
    pub timeline_edge: Edge,
    pub welcome_show_again: bool,
    pub arrow_speed: f32,
    pub zoom_speed: f32,
    pub brush: f32,
    pub brush_shape: BrushShape,
    pub tool: Tool,
    /// Current color stored as RGBA.
    pub rgba: [u8; 4],
    pub color_history: Vec<[u8; 4]>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            show_top_panel: true,
            show_right_panel: true,
            show_status_bar: true,
            show_timeline: true,
            toolbar_edge: Edge::Top,
            layers_edge: Edge::Right,
            status_edge: Edge::Bottom,
            timeline_edge: Edge::Bottom,
            welcome_show_again: true,
            arrow_speed: 5.0,
            zoom_speed: 1.0,
            brush: 1.0,
            brush_shape: BrushShape::Square,
            tool: Tool::Brush,
            rgba: [0, 0, 0, 255],
            color_history: Vec::new(),
        }
    }
}

impl Prefs {
    /// Reads settings from eframe storage. Any problem yields defaults: a
    /// corrupt file must not prevent the application from starting.
    pub(crate) fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        #[cfg(feature = "persistence")]
        let mut prefs: Self = storage
            .and_then(|s| eframe::get_value(s, PREFS_KEY))
            .unwrap_or_default();
        // on Android eframe does not provide a data directory
        // (storage_dir -> None), so there is no file there by design:
        // defaults remain
        #[cfg(not(feature = "persistence"))]
        let mut prefs: Self = {
            let _ = storage;
            Self::default()
        };
        prefs.sanitize();
        prefs
    }

    /// Brings values from the file into valid ranges. The file lives in the
    /// application data directory and remains hand-editable, while RON does
    /// not check that `brush` is a positive number. Without this the settings
    /// dialog slider drifts and the brush paints a spot outside the canvas.
    fn sanitize(&mut self) {
        self.arrow_speed = finite_or(self.arrow_speed, Self::default().arrow_speed)
            .clamp(1.0, 100.0);
        self.zoom_speed = finite_or(self.zoom_speed, Self::default().zoom_speed)
            .clamp(0.005, 5.0);
        // the toolbar clamps the brush to the canvas size anyway
        self.brush = finite_or(self.brush, Self::default().brush).clamp(1.0, 512.0);
        if self.color_history.len() > 4 {
            self.color_history.truncate(4);
        }
        // the file is hand-edited, so edges may be invalid (for example, the
        // timeline on a side). Derive allowed edges from the panel's default
        // edge rather than from the corrupted value.
        let d = Self::default();
        self.toolbar_edge = clamp_edge(self.toolbar_edge, d.toolbar_edge);
        self.layers_edge = clamp_edge(self.layers_edge, d.layers_edge);
        self.status_edge = clamp_edge(self.status_edge, d.status_edge);
        self.timeline_edge = clamp_edge(self.timeline_edge, d.timeline_edge);
    }
}

/// Keeps the edge if it is allowed for the panel, otherwise returns the default.
/// `default_edge` identifies which panel this is about: horizontal ones
/// (toolbar, status, timeline) may only move between the top and bottom edges.
pub(crate) fn clamp_edge(loaded: Edge, default_edge: Edge) -> Edge {
    if default_edge.allowed_for().contains(&loaded) {
        loaded
    } else {
        default_edge
    }
}

/// In-memory storage for tests. This way tests go through the real
/// `eframe::get_value` instead of a fake string parser. Kept outside
/// `mod tests` so other test modules can use it too.
#[cfg(all(feature = "persistence", test))]
#[derive(Default)]
pub(crate) struct MemStore(std::cell::RefCell<std::collections::HashMap<String, String>>);

#[cfg(all(feature = "persistence", test))]
impl eframe::Storage for MemStore {
    fn get_string(&self, key: &str) -> Option<String> {
        self.0.borrow().get(key).cloned()
    }
    fn set_string(&mut self, key: &str, value: String) {
        self.0.borrow_mut().insert(key.to_owned(), value);
    }
    fn flush(&mut self) {}
}

/// f32::clamp returns NaN for NaN, so non-finite values are filtered out
/// before clamping, not after.
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}

#[cfg(all(feature = "persistence", test))]
mod tests {
    use super::*;
    use eframe::Storage;

    fn round_trip(p: &Prefs) -> Prefs {
        let mut store = MemStore::default();
        eframe::set_value(&mut store, PREFS_KEY, p);
        Prefs::load(Some(&store))
    }

    #[test]
    fn prefs_survive_a_save_load_cycle() {
        let p = Prefs {
            show_timeline: false,
            show_status_bar: false,
            arrow_speed: 42.5,
            zoom_speed: 2.25,
            brush: 7.0,
            brush_shape: BrushShape::Round,
            tool: Tool::Text,
            rgba: [10, 20, 30, 128],
            color_history: vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ..Default::default()
        };
        let back = round_trip(&p);
        assert!(!back.show_timeline, "закрытая панель осталась закрытой");
        assert!(!back.show_status_bar);
        assert!(back.show_top_panel, "остальные видимости не пострадали");
        assert_eq!(back.arrow_speed, 42.5);
        assert_eq!(back.zoom_speed, 2.25);
        assert_eq!(back.brush, 7.0);
        assert_eq!(back.brush_shape, BrushShape::Round);
        assert_eq!(back.tool, Tool::Text);
        assert_eq!(back.rgba, [10, 20, 30, 128]);
        assert_eq!(back.color_history, vec![[1, 2, 3, 4], [5, 6, 7, 8]]);
    }

    /// Empty storage — an ordinary first launch; defaults are expected.
    #[test]
    fn empty_storage_yields_defaults() {
        let p = Prefs::load(None);
        assert_eq!(p.show_top_panel, true);
        assert_eq!(p.show_timeline, true);
        assert_eq!(p.welcome_show_again, true);
        assert_eq!(p.tool, Tool::Brush);

        let p2 = Prefs::load(Some(&MemStore::default()));
        assert_eq!(p2.tool, Tool::Brush);
    }

    /// A file from a previous version that lacks some fields is read
    /// partially instead of being discarded entirely.
    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let mut store = MemStore::default();
        store.set_string(PREFS_KEY, r#"(show_timeline: false, arrow_speed: 12.0)"#.into());
        let p = Prefs::load(Some(&store));
        assert!(!p.show_timeline, "что было в файле — сохраняется");
        assert_eq!(p.arrow_speed, 12.0);
        assert!(p.show_top_panel, "отсутствующее — из Default");
        assert_eq!(p.tool, Tool::Brush);
    }

    /// An unreadable file must not crash startup.
    #[test]
    fn corrupt_file_yields_defaults() {
        let mut store = MemStore::default();
        store.set_string(PREFS_KEY, "не RON {{{".into());
        let p = Prefs::load(Some(&store));
        assert_eq!(p.show_top_panel, true);
        assert_eq!(p.tool, Tool::Brush);
    }

    /// RON does not validate ranges, and the file lives in the application
    /// data directory and remains hand-editable.
    #[test]
    fn out_of_range_and_nan_values_are_sanitized() {
        let mut store = MemStore::default();
        store.set_string(
            PREFS_KEY,
            r#"(arrow_speed: 1e30, zoom_speed: -3.0, brush: 0.0, color_history: [])"#.into(),
        );
        let p = Prefs::load(Some(&store));
        assert!(p.arrow_speed.is_finite() && p.arrow_speed <= 100.0 && p.arrow_speed >= 1.0);
        assert!(p.zoom_speed >= 0.005 && p.zoom_speed <= 5.0);
        assert!(p.brush >= 1.0);
    }

    #[test]
    fn history_is_capped_like_in_the_eyedropper() {
        let mut store = MemStore::default();
        let long: Vec<[u8; 4]> = (0..99).map(|i| [i, i, i, 255]).collect();
        eframe::set_value(
            &mut store,
            PREFS_KEY,
            &Prefs { color_history: long, ..Default::default() },
        );
        assert_eq!(Prefs::load(Some(&store)).color_history.len(), 4);
    }
}