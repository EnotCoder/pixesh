//! Перетаскивание панелей между краями окна.
//!
//! За панель тянут за ручку-«грип» в начале её тела. Сделано именно отдельной
//! ручкой, а не полосой вдоль внутреннего ребра: внутреннее ребро таймлайна
//! занято кнопками Delete/Duplicate/Add, и полоса-приёмник перехватывала бы их
//! нажатия. Ручка ничего не перекрывает, поэтому вёрстку панелей менять не
//! пришлось — только добавить один элемент в начало.
//!
//! Пока панель тащат, по краям окна подсвечиваются зоны-приёмники. Цель
//! выбирается по позиции курсора, а не по наведению на виджет: так подсветка
//! ничего не перехватывает и работает, даже когда курсор над холстом.

use eframe::egui::{self, Color32, CursorIcon, Pos2, Rect, Sense, Vec2};

use crate::app::PixeshApp;
use crate::constants::*;

/// Размер ручки в пикселях.
const GRAB: f32 = 22.0;

/// Насколько близко к краю надо подвести курсор, чтобы сработала зона.
const EDGE_ZONE: f32 = 140.0;

impl PixeshApp {
    /// Ручка в начале панели. Возвращает ничего: перенос целиком держит в
    /// состоянии, зоны рисует ui_panel_drag.
    pub(crate) fn grab_handle(&mut self, ui: &mut egui::Ui, p: PanelId) {
        let (_rect, resp) = ui.allocate_exact_size(Vec2::splat(GRAB), Sense::click_and_drag());
        let painter = ui.painter().clone();
        let r = resp.rect;
        let hot = resp.hovered() || resp.dragged();
        // три пары точек — классический «грип», читается как ручка
        for row in 0..3 {
            for col in 0..2 {
                let c = Pos2::new(
                    r.min.x + 5.0 + col as f32 * 6.0,
                    r.min.y + 5.0 + row as f32 * 6.0,
                );
                painter.rect_filled(
                    Rect::from_center_size(c, Vec2::splat(3.0)),
                    0.0,
                    if hot { ACCENT } else { DIM },
                );
            }
        }
        if hot {
            ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
        }

        if resp.drag_started() {
            self.dragging_panel = Some(p);
            self.drag_from_edge = self.edge_of(p);
        }
        if resp.dragged() {
            if let Some(pos) = ui.input(|i| i.pointer.interact_pos()) {
                self.drop_edge = edge_at(pos, p, ui.ctx().screen_rect());
            }
        }
        if resp.drag_stopped() {
            if let Some(target) = self.drop_edge {
                if target != self.drag_from_edge {
                    self.set_edge(p, target);
                }
            }
            self.cancel_panel_drag();
        }
    }

    /// Зоны-приёмники и подсветка активной. Вызывается после отрисовки холста.
    pub(crate) fn ui_panel_drag(&mut self, ctx: &egui::Context) {
        // на мобильной раскладке таймлайна и статус-бара нет, а диалог должен
        // перехватывать мышь целиком
        if self.mobile || self.dialog_open() {
            self.cancel_panel_drag();
            return;
        }
        let Some(panel) = self.dragging_panel else { return };

        // курсор мог уйти за пределы окна — тогда зон не рисуем
        if let Some(pos) = ctx.input(|i| i.pointer.interact_pos()) {
            self.drop_edge = edge_at(pos, panel, ctx.screen_rect());
        }

        let painter = ctx.layer_painter(egui::LayerId::new(
            egui::Order::Tooltip,
            egui::Id::new("panel_drop_zones"),
        ));
        let screen = ctx.screen_rect();
        for edge in panel.shape_edge().allowed_for() {
            let zone = zone_rect(screen, *edge);
            let active = self.drop_edge == Some(*edge);
            painter.rect_filled(
                zone,
                0.0,
                Color32::from_black_alpha(if active { 90 } else { 40 }),
            );
            painter.rect_stroke(
                zone,
                0.0,
                egui::Stroke::new(if active { 6.0 } else { 3.0 }, ACCENT),
                egui::StrokeKind::Inside,
            );
        }
    }

    /// Перенос не начат или уже завершён — сбрасываем состояние.
    pub(crate) fn cancel_panel_drag(&mut self) {
        self.dragging_panel = None;
        self.drop_edge = None;
    }
}

/// Ближайший край из разрешённых для этой панели. Сравниваем доли экрана, а не
/// абсолютные расстояния: на широком окне до левого края далеко даже когда
/// курсор мысленно «у левого края».
///
/// Расстояние считается до *каждого* края отдельно, а не до ближайшего из двух
/// параллельных: для верхней/нижней пары «до ближайшего» — одно и то же число,
/// и панель, которую тянут к нижнему краю, выдала бы Top.
fn edge_at(pos: Pos2, panel: PanelId, screen: Rect) -> Option<Edge> {
    let (w, h) = (screen.width().max(1.0), screen.height().max(1.0));
    let dist = |e: Edge| -> f32 {
        match e {
            Edge::Top => (pos.y - screen.min.y) / h,
            Edge::Bottom => (screen.max.y - pos.y) / h,
            Edge::Left => (pos.x - screen.min.x) / w,
            Edge::Right => (screen.max.x - pos.x) / w,
        }
    };
    panel
        .shape_edge()
        .allowed_for()
        .iter()
        .map(|e| (dist(*e), *e))
        .min_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(_, e)| e)
}

/// Полоса-приёмник у края окна.
fn zone_rect(screen: Rect, edge: Edge) -> Rect {
    match edge {
        Edge::Top => Rect::from_min_max(
            screen.min,
            Pos2::new(screen.max.x, screen.min.y + EDGE_ZONE),
        ),
        Edge::Bottom => Rect::from_min_max(
            Pos2::new(screen.min.x, screen.max.y - EDGE_ZONE),
            screen.max,
        ),
        Edge::Left => Rect::from_min_max(
            screen.min,
            Pos2::new(screen.min.x + EDGE_ZONE, screen.max.y),
        ),
        Edge::Right => Rect::from_min_max(
            Pos2::new(screen.max.x - EDGE_ZONE, screen.min.y),
            screen.max,
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn screen() -> Rect {
        Rect::from_min_size(Pos2::ZERO, Vec2::new(1000.0, 800.0))
    }

    #[test]
    fn top_left_of_the_window_picks_left_for_the_layers_panel() {
        let e = edge_at(Pos2::new(5.0, 400.0), PanelId::Layers, screen());
        assert_eq!(e, Some(Edge::Left));
    }

    #[test]
    fn bottom_right_picks_bottom_for_the_timeline() {
        let e = edge_at(Pos2::new(995.0, 795.0), PanelId::Timeline, screen());
        assert_eq!(e, Some(Edge::Bottom));
    }

    /// Таймлайн нельзя прижать к боку окна — приёмников там нет вовсе.
    #[test]
    fn horizontal_panels_never_resolve_to_a_side_edge() {
        for p in [PanelId::Toolbar, PanelId::Status, PanelId::Timeline] {
            for pos in [
                Pos2::new(1.0, 1.0),
                Pos2::new(999.0, 799.0),
                Pos2::new(500.0, 400.0),
                Pos2::new(2.0, 700.0),
            ] {
                let e = edge_at(pos, p, screen()).unwrap();
                assert!(
                    !e.is_vertical(),
                    "{p:?} предложил боковой край {e:?} при курсоре {pos:?}"
                );
            }
        }
    }

    /// И наоборот: панель слоёв не уезжает на верх или низ.
    #[test]
    fn the_layers_panel_never_resolves_to_a_horizontal_edge() {
        for pos in [
            Pos2::new(500.0, 1.0),
            Pos2::new(500.0, 799.0),
            Pos2::new(500.0, 400.0),
        ] {
            let e = edge_at(pos, PanelId::Layers, screen()).unwrap();
            assert!(e.is_vertical(), "{e:?} для панели слоёв при {pos:?}");
        }
    }

    /// Зона приёмника должна лежать у своего края и не залезать на соседний.
    #[test]
    fn drop_zones_sit_at_their_own_edges() {
        let s = screen();
        let top = zone_rect(s, Edge::Top);
        assert!((top.min.y - s.min.y).abs() < 0.01);
        assert!((top.max.x - s.max.x).abs() < 0.01);
        assert!(top.height() < s.height() / 2.0, "зона не должна закрывать пол-окна");

        let left = zone_rect(s, Edge::Left);
        assert!((left.min.x - s.min.x).abs() < 0.01);
        assert!((left.max.y - s.max.y).abs() < 0.01);
        assert!(left.width() < s.width() / 2.0);

        let bottom = zone_rect(s, Edge::Bottom);
        assert!((bottom.max.y - s.max.y).abs() < 0.01);
        let right = zone_rect(s, Edge::Right);
        assert!((right.max.x - s.max.x).abs() < 0.01);
    }

    /// Каждой панели доступны ровно два края, и они противоположны.
    #[test]
    fn every_panel_has_exactly_two_opposite_edges() {
        for p in [PanelId::Toolbar, PanelId::Status, PanelId::Timeline, PanelId::Layers] {
            let a = p.shape_edge().allowed_for();
            assert_eq!(a.len(), 2, "{p:?}");
            assert_ne!(a[0], a[1], "{p:?}: края должны быть противоположны");
        }
    }
}