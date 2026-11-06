use eframe::egui::Color32;

use crate::app::config;
use crate::constants::{Edge, Tool};

use super::PixeshApp;

impl PixeshApp {
    /// Default preferences. Without the persistence feature there is no file,
    /// so Preferences simply becomes a set of initial field values.
    pub(super) fn initial_prefs() -> config::Prefs {
        config::Prefs::default()
    }

    /// Snapshot of the persisted state. The list is deliberately short: only
    /// things that make sense to remember between launches belong here.
    /// Dialog flags (show_resize/show_export/show_settings/show_panels/
    /// show_text/show_effects/show_quit_dialog) do NOT go here — otherwise a
    /// dialog would pop up on startup and a "closed window" would reopen itself.
    #[cfg_attr(not(feature = "persistence"), allow(dead_code))]
    pub(crate) fn prefs(&self) -> config::Prefs {
        config::Prefs {
            show_top_panel: self.show_top_panel,
            show_right_panel: self.show_right_panel,
            show_status_bar: self.show_status_bar,
            show_timeline: self.show_timeline,
            toolbar_edge: self.toolbar_edge,
            layers_edge: self.layers_edge,
            status_edge: self.status_edge,
            timeline_edge: self.timeline_edge,
            welcome_show_again: self.welcome_show_again,
            arrow_speed: self.arrow_speed,
            zoom_speed: self.zoom_speed,
            brush: self.brush,
            brush_shape: self.brush_shape,
            tool: self.tool,
            rgba: [
                self.rgb_r.clamp(0.0, 255.0) as u8,
                self.rgb_g.clamp(0.0, 255.0) as u8,
                self.rgb_b.clamp(0.0, 255.0) as u8,
                self.rgb_a.clamp(0.0, 255.0) as u8,
            ],
            color_history: self
                .color_history
                .iter()
                .map(|c| [c.r(), c.g(), c.b(), c.a()])
                .collect(),
        }
    }

    /// Restores state from the file. eframe 0.31 has no `App::restore`
    /// method — the storage has to be read here, in the creation closure.
    pub(crate) fn load_prefs(&mut self, storage: Option<&dyn eframe::Storage>) {
        let prefs = config::Prefs::load(storage);

        self.show_top_panel = prefs.show_top_panel;
        self.show_right_panel = prefs.show_right_panel;
        self.show_status_bar = prefs.show_status_bar;
        self.show_timeline = prefs.show_timeline;
        self.toolbar_edge = config::clamp_edge(prefs.toolbar_edge, Edge::Top);
        self.layers_edge = config::clamp_edge(prefs.layers_edge, Edge::Right);
        self.status_edge = config::clamp_edge(prefs.status_edge, Edge::Bottom);
        self.timeline_edge = config::clamp_edge(prefs.timeline_edge, Edge::Bottom);
        self.welcome_show_again = prefs.welcome_show_again;
        self.arrow_speed = prefs.arrow_speed;
        self.zoom_speed = prefs.zoom_speed;
        self.brush = prefs.brush;
        self.brush_shape = prefs.brush_shape;
        self.tool = prefs.tool;
        self.set_color(Color32::from_rgba_unmultiplied(
            prefs.rgba[0], prefs.rgba[1], prefs.rgba[2], prefs.rgba[3],
        ));
        self.color_history = prefs
            .color_history
            .iter()
            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .collect();

        // dialogs must all start closed
        self.show_resize = false;
        self.show_scale = false;
        self.show_export = false;
        self.show_panels = false;
        self.show_settings = false;
        self.show_quit_dialog = false;
        self.show_effects = false;
        self.show_text = false;
        self.show_welcome = self.welcome_show_again;
        if self.tool == Tool::Effects { self.tool = Tool::Brush; }
        self.renaming_layer = None;
    }
}
