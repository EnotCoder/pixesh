use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;

// ── btn ──────────────────────────────────────────────
// кастомная текстовая кнопка: фон подсвечивается при наведении/клике
pub fn btn(ui: &mut egui::Ui, label: &str) -> bool {
    btn_min_w(ui, label, 0.0)
}

pub fn btn_min_w(ui: &mut egui::Ui, label: &str, min_w: f32) -> bool {
    let font_id = ui.style().text_styles.get(&egui::TextStyle::Button)
        .cloned()
        .unwrap_or(egui::FontId::proportional(FONT_SZ));
    let font_sz = font_id.size;
    let label_w = label.len() as f32 * CHAR_W * (font_sz / FONT_SZ);
    let pad = Vec2::new(14.0, 8.0);
    let w = label_w + pad.x * 2.0;
    let size = Vec2::new(w.max(min_w), font_sz + pad.y * 2.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let bg = if resp.clicked() {
        ACCENT
    } else if resp.hovered() {
        HOVER
    } else {
        PANEL
    };
    let p = ui.painter();
    p.rect_filled(rect, 4.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
    p.text(rect.center(), egui::Align2::CENTER_CENTER, label, font_id, TEXT);

    resp.clicked()
}

// ── icon_btn ─────────────────────────────────────────
// квадратная кнопка с иконкой (текстурой), active = подсвечена
pub fn icon_btn(ui: &mut egui::Ui, tex_id: egui::TextureId, active: bool) -> bool {
    let size = Vec2::splat(ROW_H + 16.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let bg = if active {
        ACCENT
    } else if resp.hovered() {
        HOVER
    } else {
        PANEL
    };
    let p = ui.painter();
    p.rect_filled(rect, 4.0, bg);

    let img_rect = rect;
    p.image(tex_id, img_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

    p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);

    resp.clicked()
}

// ── checkbox ─────────────────────────────────────────
// кастомный чекбокс: квадратик + текст
pub fn checkbox(ui: &mut egui::Ui, label: &str, checked: &mut bool) {
    let cbs = 18.0;
    let total_h = (ROW_H + 8.0).max(cbs + 8.0);
    let label_w = label.len() as f32 * CHAR_W;
    let total_w = cbs + 12.0 + label_w;

    let (rect, _) = ui.allocate_exact_size(Vec2::new(total_w, total_h), Sense::click());

    let cb_rect = Rect::from_min_size(
        Pos2::new(rect.min.x + 4.0, rect.center().y - cbs * 0.5),
        Vec2::splat(cbs),
    );
    let p = ui.painter();
    p.rect_filled(cb_rect, 3.0, PANEL);
    p.rect_stroke(cb_rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);

    if *checked {
        let inner = cb_rect.shrink(4.0);
        p.rect_filled(inner, 2.0, ACCENT);
    }

    let cb_resp = ui.interact(cb_rect, egui::Id::new(label), Sense::click());
    if cb_resp.clicked() {
        *checked = !*checked;
    }

    let label_rect = Rect::from_min_max(
        Pos2::new(cb_rect.max.x + 6.0, rect.min.y),
        Pos2::new(rect.max.x, rect.max.y),
    );
    let lresp = ui.interact(label_rect, egui::Id::new(format!("{}_l", label)), Sense::click());
    if lresp.clicked() {
        *checked = !*checked;
    }

    p.text(
        Pos2::new(cb_rect.max.x + 8.0, rect.center().y - ROW_H * 0.5),
        egui::Align2::LEFT_TOP,
        label,
        egui::FontId::proportional(FONT_SZ),
        TEXT,
    );
}

// ── separator ────────────────────────────────────────
// вертикальная разделительная линия
pub fn separator(ui: &mut egui::Ui) {
    let h = ROW_H + 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, h), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(2.0, BORDER),
    );
}
