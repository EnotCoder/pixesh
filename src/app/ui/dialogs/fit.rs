//! Fitting fixed-size dialogs into small windows.

use eframe::egui::{self, Rect, Sense, Stroke, Vec2};

use crate::constants::*;

/// Margin kept free around a dialog, split between both sides.
const DIALOG_MARGIN: f32 = 16.0;

/// Largest frame a dialog of `designed` size can use inside the window.
fn fitted_size(ctx: &egui::Context, designed: Vec2) -> Vec2 {
    let screen = ctx.screen_rect().size();
    let avail = (screen - Vec2::splat(DIALOG_MARGIN)).max(Vec2::splat(64.0));
    Vec2::new(designed.x.min(avail.x), designed.y.min(avail.y))
}

/// Draws a dialog that keeps its designed content size but fits into the
/// window. The background and border are drawn on the fitted frame; `body`
/// receives the (possibly larger) content rect and runs inside a scroll area
/// when the window is too small to show everything at once.
///
/// `body` must draw all dialog content relative to the given rect and use the
/// provided `ui` for painting/interaction, so it stays clipped and scrollable.
pub(crate) fn fitted_dialog(
    ui: &mut egui::Ui,
    id_salt: &str,
    designed: Vec2,
    body: impl FnOnce(&mut egui::Ui, Rect),
) {
    let viewport = fitted_size(ui.ctx(), designed);
    let (frame, _) = ui.allocate_exact_size(viewport, Sense::hover());

    let p = ui.painter();
    p.rect_filled(frame, 0.0, PANEL);
    p.rect_stroke(
        frame,
        0.0,
        Stroke::new(4.0_f32, BORDER),
        egui::StrokeKind::Outside,
    );

    let need_x = viewport.x + 0.5 < designed.x;
    let need_y = viewport.y + 0.5 < designed.y;

    // When everything fits, the content is exactly the frame.
    if !need_x && !need_y {
        body(ui, frame);
        return;
    }

    // Content keeps its designed size; when it does not fit, the area below
    // clips it and lets the user scroll to the rest instead of cutting it off.
    let draw = move |ui: &mut egui::Ui| {
        let (content, _) = ui.allocate_exact_size(designed, Sense::hover());
        body(ui, content);
    };

    let mut area = ui.new_child(egui::UiBuilder::new().max_rect(frame));
    if need_x && need_y {
        egui::ScrollArea::both()
            .id_salt(id_salt)
            .auto_shrink([false, false])
            .show(&mut area, draw);
    } else if need_y {
        egui::ScrollArea::vertical()
            .id_salt(id_salt)
            .auto_shrink([false, false])
            .show(&mut area, draw);
    } else {
        egui::ScrollArea::horizontal()
            .id_salt(id_salt)
            .auto_shrink([false, false])
            .show(&mut area, draw);
    }
}
