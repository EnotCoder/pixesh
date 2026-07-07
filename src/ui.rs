use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;

// ── btn ──────────────────────────────────────────────
// кастомная текстовая кнопка: фон подсвечивается при наведении/клике
pub fn btn(ui: &mut egui::Ui, label: &str) -> bool {
    // считаем ширину по длине текста
    let label_w = label.len() as f32 * CHAR_W;
    let pad = Vec2::new(8.0, 4.0);
    let size = Vec2::new(label_w + pad.x * 2.0, ROW_H + pad.y * 2.0);
    // выделяем прямоугольник в UI
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    // цвет фона зависит от состояния
    let bg = if resp.clicked() {
        ACCENT
    } else if resp.hovered() {
        HOVER
    } else {
        PANEL
    };
    let p = ui.painter();
    // фон + рамка
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);
    // текст
    p.text(rect.min + pad, egui::Align2::LEFT_TOP, label, egui::FontId::proportional(FONT_SZ), TEXT);

    resp.clicked() // true только один кадр после клика
}

// ── icon_btn ─────────────────────────────────────────
// квадратная кнопка с иконкой (текстурой), active = подсвечена
pub fn icon_btn(ui: &mut egui::Ui, tex_id: egui::TextureId, active: bool) -> bool {
    let size = Vec2::splat(ROW_H + 8.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let bg = if active {
        ACCENT
    } else if resp.hovered() {
        HOVER
    } else {
        PANEL
    };
    let p = ui.painter();
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    // рисуем текстуру на всю кнопку
    let img_rect = rect;
    p.image(tex_id, img_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

    resp.clicked()
}

// ── checkbox ─────────────────────────────────────────
// кастомный чекбокс: квадратик + текст
pub fn checkbox(ui: &mut egui::Ui, label: &str, checked: &mut bool) {
    let cbs = 16.0;
    let total_h = ROW_H.max(cbs) + 4.0;
    let label_w = label.len() as f32 * CHAR_W;
    let total_w = cbs + 8.0 + label_w;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click());

    // квадратик чекбокса слева
    let cb_rect = Rect::from_min_size(
        Pos2::new(rect.min.x, rect.center().y - cbs * 0.5),
        Vec2::splat(cbs),
    );
    let p = ui.painter();
    p.rect_filled(cb_rect, 0.0, PANEL);
    p.rect_stroke(cb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    if *checked {
        let inner = cb_rect.shrink(3.0);
        p.rect_filled(inner, 0.0, ACCENT);
    }

    // обрабатываем клик по квадратику
    let cb_resp = ui.interact(cb_rect, egui::Id::new(label), Sense::click());
    if cb_resp.clicked() {
        *checked = !*checked;
    }

    // клик по тексту тоже переключает
    let label_rect = Rect::from_min_max(
        Pos2::new(cb_rect.max.x + 4.0, rect.min.y),
        Pos2::new(rect.max.x, rect.max.y),
    );
    let lresp = ui.interact(label_rect, egui::Id::new(format!("{}_l", label)), Sense::click());
    if lresp.clicked() {
        *checked = !*checked;
    }

    // текст
    p.text(
        Pos2::new(cb_rect.max.x + 6.0, rect.center().y - ROW_H * 0.5),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(FONT_SZ),
        TEXT,
    );
}

// ── slider ───────────────────────────────────────────
// кастомный ползунок: трек + бегунок + числовое значение
pub fn slider(ui: &mut egui::Ui, label: &str, value: &mut f32, min: f32, max: f32) -> bool {
    let track_w = 80.0;
    let thumb_w = 10.0;
    let label_w = (label.len() as f32 * CHAR_W) + 50.0;
    let total_w = track_w + 8.0 + label_w;
    let total_h = ROW_H + 8.0;

    let mut changed = false;
    let (rect, resp) =
        ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click_and_drag());
    let p = ui.painter();

    // подпись + значение справа
    let label_str = format!("{}{}", label, *value as i32);
    p.text(
        Pos2::new(rect.max.x - label_w, rect.center().y - ROW_H * 0.5),
        egui::Align2::LEFT_TOP,
        &label_str,
        egui::FontId::proportional(FONT_SZ),
        TEXT,
    );

    // трек (серый прямоугольник)
    let ty = rect.center().y - 4.0;
    let track_rect = Rect::from_min_size(
        Pos2::new(rect.min.x, ty),
        Vec2::new(track_w, 8.0),
    );
    p.rect_filled(track_rect, 0.0, Color32::from_gray(30));
    p.rect_stroke(track_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    // бегунок (положение = пропорция между min и max)
    let t = ((*value - min) / (max - min)).clamp(0.0, 1.0);
    let thumb_x = track_rect.min.x + t * (track_w - thumb_w);
    let thumb_rect = Rect::from_min_size(
        Pos2::new(thumb_x, rect.center().y - 7.0),
        Vec2::new(thumb_w, 14.0),
    );
    let thumb_bg = if resp.dragged() || resp.hovered() {
        ACCENT
    } else {
        PANEL_LIGHT
    };
    p.rect_filled(thumb_rect, 0.0, thumb_bg);
    p.rect_stroke(thumb_rect, 0.0, Stroke::new(1.0, BORDER), egui::StrokeKind::Outside);

    // таскаем бегунок
    if resp.dragged_by(egui::PointerButton::Primary) {
        if let Some(pos) = resp.interact_pointer_pos() {
            let t2 = ((pos.x - track_rect.min.x) / track_w).clamp(0.0, 1.0);
            *value = min + t2 * (max - min);
            changed = true;
        }
    }

    changed
}

// ── separator ────────────────────────────────────────
// вертикальная разделительная линия
pub fn separator(ui: &mut egui::Ui) {
    let h = ROW_H + 6.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(2.0, h), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(1.0, BORDER),
    );
}
