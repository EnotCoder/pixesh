use crate::app::config;
use crate::constants::*;

use super::PixeshApp;

impl PixeshApp {
    /// The edge the panel is currently on.
    pub(crate) fn edge_of(&self, p: PanelId) -> Edge {
        match p {
            PanelId::Toolbar => self.toolbar_edge,
            PanelId::Layers => self.layers_edge,
            PanelId::Status => self.status_edge,
            PanelId::Timeline => self.timeline_edge,
        }
    }

    pub(crate) fn set_edge(&mut self, p: PanelId, e: Edge) {
        let e = config::clamp_edge(e, p.default_edge());
        match p {
            PanelId::Toolbar => self.toolbar_edge = e,
            PanelId::Layers => self.layers_edge = e,
            PanelId::Status => self.status_edge = e,
            PanelId::Timeline => self.timeline_edge = e,
        }
    }

    pub(crate) fn panel_visible(&self, p: PanelId) -> bool {
        match p {
            PanelId::Toolbar => self.show_top_panel,
            PanelId::Layers => self.show_right_panel,
            PanelId::Status => self.show_status_bar,
            PanelId::Timeline => self.show_timeline,
        }
    }

    /// Panel draw order: grouped by edge, and within an edge by canonical
    /// order. Only visible panels are returned.
    ///
    /// Kept separate and free of egui so the behaviour can be covered by a
    /// test: the key property here is that all horizontal strips come before
    /// side panels. Otherwise a top panel drawn after a side panel would not
    /// get the full window width: egui carves panels out of the remaining
    /// rectangle strictly in call order.
    pub(crate) fn draw_order(&self) -> Vec<PanelId> {
        let mut out: Vec<PanelId> = [PanelId::Toolbar, PanelId::Status, PanelId::Timeline, PanelId::Layers]
            .into_iter()
            .filter(|p| self.panel_visible(*p))
            .collect();
        out.sort_by_key(|p| (self.edge_of(*p).draw_group(), p_order(*p)));
        out
    }
}

// ── panel layout ─────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    /// The key property of the draw order: the top strip (Toolbar) comes before
    /// side panels, and side panels (Layers) come before bottom strips (Status,
    /// Timeline), so the bottom panels do not collide with the layers panel
    /// (do not run underneath it).
    #[test]
    fn layers_panel_drawn_before_bottom_panels() {
        let mut app = PixeshApp::new();
        // standard arrangement: toolbar on top, layers on the right, status and
        // timeline at the bottom
        app.toolbar_edge = Edge::Top;
        app.layers_edge = Edge::Right;
        app.status_edge = Edge::Bottom;
        app.timeline_edge = Edge::Bottom;

        let order = app.draw_order();
        let layers_pos = order
            .iter()
            .position(|p| *p == PanelId::Layers)
            .unwrap();
        let status_pos = order
            .iter()
            .position(|p| *p == PanelId::Status)
            .unwrap();
        let timeline_pos = order
            .iter()
            .position(|p| *p == PanelId::Timeline)
            .unwrap();
        assert!(
            layers_pos < status_pos && layers_pos < timeline_pos,
            "панель слоёв должна рисоваться раньше нижних панелей: {order:?}"
        );
    }

    /// The relative order of panels does not depend on which edge they are on:
    /// it is determined by the edge group (Top -> Left/Right -> Bottom), and
    /// within one edge by the canonical order.
    #[test]
    fn order_inside_an_edge_is_stable() {
        // all three horizontal panels on top, layers on the left:
        // Top (Toolbar, Status, Timeline) -> Left (Layers)
        let mut app = PixeshApp::new();
        app.toolbar_edge = Edge::Top;
        app.status_edge = Edge::Top;
        app.timeline_edge = Edge::Top;
        app.layers_edge = Edge::Left;
        assert_eq!(
            app.draw_order(),
            vec![PanelId::Toolbar, PanelId::Status, PanelId::Timeline, PanelId::Layers],
            "верх + лево"
        );

        // the same panels, but Toolbar, Status, Timeline at the bottom, layers
        // on the right:
        // Right (Layers) -> Bottom (Toolbar, Status, Timeline)
        let mut app = PixeshApp::new();
        app.toolbar_edge = Edge::Bottom;
        app.status_edge = Edge::Bottom;
        app.timeline_edge = Edge::Bottom;
        app.layers_edge = Edge::Right;
        assert_eq!(
            app.draw_order(),
            vec![PanelId::Layers, PanelId::Toolbar, PanelId::Status, PanelId::Timeline],
            "низ + право"
        );

        // move the timeline to the top:
        // Top (Toolbar, Timeline) -> Right (Layers) -> Bottom (Status)
        let mut app = PixeshApp::new();
        app.timeline_edge = Edge::Top;
        app.status_edge = Edge::Bottom;
        assert_eq!(
            app.draw_order(),
            vec![PanelId::Toolbar, PanelId::Timeline, PanelId::Layers, PanelId::Status],
            "таймлайн наверху, статус внизу"
        );
    }

    #[test]
    fn hidden_panels_are_not_drawn() {
        let mut app = PixeshApp::new();
        app.show_timeline = false;
        app.show_status_bar = false;
        let order = app.draw_order();
        assert!(!order.contains(&PanelId::Timeline));
        assert!(!order.contains(&PanelId::Status));
        assert_eq!(order, vec![PanelId::Toolbar, PanelId::Layers]);
    }

    /// A move must not take a panel to a forbidden edge, even if it was forced
    /// in by hand (for example by editing the settings file).
    #[test]
    fn set_edge_refuses_forbidden_edges() {
        let mut app = PixeshApp::new();
        app.set_edge(PanelId::Timeline, Edge::Left);
        assert_ne!(app.timeline_edge, Edge::Left, "таймлайн нельзя в бок");
        app.set_edge(PanelId::Layers, Edge::Top);
        assert_ne!(app.layers_edge, Edge::Top, "слои нельзя наверх");
        app.set_edge(PanelId::Timeline, Edge::Top);
        assert_eq!(app.timeline_edge, Edge::Top, "разрешённый край принимаем");
        app.set_edge(PanelId::Layers, Edge::Left);
        assert_eq!(app.layers_edge, Edge::Left);
    }

    #[test]
    fn panel_ids_differ_per_edge_so_sizes_do_not_mix() {
        // key point: PanelState lives in egui::Memory keyed by Id, and for a
        // side panel it is read as rect.width(), while for a horizontal panel
        // it is read as rect.height(). Different Ids per edge — otherwise a
        // panel moved from the bottom to a side would read the old height as
        // its width
        assert_ne!(
            PanelId::Timeline.ui_id(Edge::Bottom),
            PanelId::Timeline.ui_id(Edge::Top)
        );
        assert_ne!(
            PanelId::Layers.ui_id(Edge::Left),
            PanelId::Layers.ui_id(Edge::Right)
        );
        // and the Ids are unique between panels as well
        assert_ne!(
            PanelId::Timeline.ui_id(Edge::Top),
            PanelId::Toolbar.ui_id(Edge::Top)
        );
    }

    #[test]
    fn edges_survive_a_restart() {
        let mut first = PixeshApp::new();
        first.set_edge(PanelId::Timeline, Edge::Top);
        first.set_edge(PanelId::Toolbar, Edge::Bottom);
        first.set_edge(PanelId::Layers, Edge::Left);

        let mut store = config::MemStore::default();
        eframe::set_value(&mut store, config::PREFS_KEY, &first.prefs());
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert_eq!(second.timeline_edge, Edge::Top);
        assert_eq!(second.toolbar_edge, Edge::Bottom);
        assert_eq!(second.layers_edge, Edge::Left);
        assert_eq!(second.status_edge, Edge::Bottom);
    }

    /// A settings file from an older version without edge fields is read as
    /// the original layout, not as "panels got mixed up".
    #[test]
    fn layout_without_edge_fields_keeps_the_original_arrangement() {
        let mut store = config::MemStore::default();
        eframe::set_value(
            &mut store,
            config::PREFS_KEY,
            &config::Prefs { show_timeline: false, ..Default::default() },
        );
        let mut app = PixeshApp::new();
        app.load_prefs(Some(&store));

        assert!(!app.show_timeline);
        assert_eq!(app.toolbar_edge, Edge::Top);
        assert_eq!(app.layers_edge, Edge::Right);
        assert_eq!(app.status_edge, Edge::Bottom);
        assert_eq!(app.timeline_edge, Edge::Bottom);
    }

    #[test]
    fn a_forbidden_edge_in_the_file_is_repaired() {
        let mut store = config::MemStore::default();
        eframe::set_value(
            &mut store,
            config::PREFS_KEY,
            &config::Prefs {
                timeline_edge: Edge::Left, // garbage in the file
                layers_edge: Edge::Top,     // garbage in the file
                ..Default::default()
            },
        );
        let mut app = PixeshApp::new();
        app.load_prefs(Some(&store));
        assert_eq!(app.timeline_edge, Edge::Bottom, "вернули таймлайн вниз");
        assert_eq!(app.layers_edge, Edge::Right, "вернули слои вправо");
    }
}
