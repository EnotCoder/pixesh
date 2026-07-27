use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;

pub(crate) fn load_icon_texture(ui: &egui::Ui, name: &str, bytes: &[u8]) -> egui::TextureHandle {
    let img = match image::load_from_memory(bytes) {
        Ok(img) => img.into_rgba8(),
        Err(_) => return {
            let fallback = ColorImage::from_rgba_unmultiplied([1, 1], &[255, 0, 255, 255]);
            ui.ctx().load_texture(name, fallback, egui::TextureOptions::NEAREST)
        },
    };
    let w = img.width() as usize;
    let h = img.height() as usize;
    let raw = img.into_raw();
    let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
    ui.ctx().load_texture(name, ci, egui::TextureOptions::NEAREST)
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
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);
    p.text(rect.center(), egui::Align2::CENTER_CENTER, label, font_id, TEXT);

    resp.clicked()
}

// ── icon_btn with tooltip ────────────────────────────
pub fn icon_btn_tip(ui: &mut egui::Ui, tex_id: egui::TextureId, active: bool, tip: &str) -> bool {
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
    p.rect_filled(rect, 0.0, bg);
    p.image(tex_id, rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
    p.rect_stroke(rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Inside);

    let resp = resp.on_hover_text(tip);
    resp.clicked()
}

// ── separator ────────────────────────────────────────
pub fn separator(ui: &mut egui::Ui) {
    let h = ROW_H + 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, h), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(4.0, BORDER),
    );
}
