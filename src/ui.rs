use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::color::lerp_color;

/// Apply the dark pixel-art style. Called at startup and after a Lua reload so
/// egui's own widgets follow the same theme as the custom-painted ones.
pub(crate) fn apply_style(ctx: &egui::Context) {
    let mut style = (*ctx.style()).clone();
    style.visuals = egui::Visuals {
        dark_mode: true,
        override_text_color: Some(crate::constants::text()),
        window_fill: crate::constants::panel(),
        panel_fill: crate::constants::panel(),
        faint_bg_color: crate::constants::panel_light(),
        extreme_bg_color: crate::constants::bg(),
        ..Default::default()
    };
    style.spacing.item_spacing = egui::Vec2::new(10.0, 6.0);
    style.spacing.button_padding = egui::Vec2::new(10.0, 6.0);
    // Pixel-art style: zero corner radius everywhere
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
    style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
    style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
    style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
    style.visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 1.0 };
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::proportional(crate::constants::font_sz()),
    );
    ctx.set_style(style);
}

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
        .unwrap_or(egui::FontId::proportional(crate::constants::font_sz()));
    let font_sz = font_id.size;
    let label_w = label.len() as f32 * crate::constants::char_w() * (font_sz / crate::constants::font_sz());
    let pad = Vec2::new(14.0, 8.0);
    let w = label_w + pad.x * 2.0;
    let size = Vec2::new(w.max(min_w), font_sz + pad.y * 2.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let t_hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());
    let t_active = ui.ctx().animate_bool(resp.id.with("active"), resp.is_pointer_button_down_on());

    let mut bg = crate::constants::panel();
    bg = lerp_color(bg, crate::constants::hover(), t_hover);
    bg = lerp_color(bg, crate::constants::accent(), t_active);

    let offset = if resp.is_pointer_button_down_on() { 2.0 } else { 0.0 };
    let draw_rect = rect.translate(Vec2::new(0.0, offset));

    let p = ui.painter();
    if offset == 0.0 {
        p.rect_filled(rect.translate(Vec2::new(0.0, 2.0)), 0.0, crate::constants::border());
    }
    p.rect_filled(draw_rect, 0.0, bg);
    p.rect_stroke(draw_rect, 0.0, Stroke::new(4.0_f32, crate::constants::border()), egui::StrokeKind::Inside);
    p.text(draw_rect.center(), egui::Align2::CENTER_CENTER, label, font_id, crate::constants::text());

    resp.clicked()
}

// ── toggle button (segmented selection) ──────────────
pub fn toggle_btn(ui: &mut egui::Ui, label: &str, active: bool) -> bool {
    let font_id = ui.style().text_styles.get(&egui::TextStyle::Button)
        .cloned()
        .unwrap_or(egui::FontId::proportional(crate::constants::font_sz()));
    let font_sz = font_id.size;
    let label_w = label.len() as f32 * crate::constants::char_w() * (font_sz / crate::constants::font_sz());
    let pad_x = 14.0;
    let h = crate::constants::row_h() + 16.0; // Standard button height matching other buttons
    let w = (label_w + pad_x * 2.0).max(80.0); // Minimum width for symmetry
    let size = Vec2::new(w, h);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let t_hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());
    let t_active = ui.ctx().animate_bool(resp.id.with("active"), active);

    let mut bg = crate::constants::panel();
    bg = lerp_color(bg, crate::constants::hover(), t_hover);
    bg = lerp_color(bg, crate::constants::accent(), t_active);

    let is_down = resp.is_pointer_button_down_on();
    let offset = if is_down { 2.0 } else { 0.0 };
    let draw_rect = rect.translate(Vec2::new(0.0, offset));

    let p = ui.painter();
    if offset == 0.0 {
        p.rect_filled(rect.translate(Vec2::new(0.0, 2.0)), 0.0, crate::constants::border());
    }
    p.rect_filled(draw_rect, 0.0, bg);
    p.rect_stroke(draw_rect, 0.0, Stroke::new(4.0_f32, crate::constants::border()), egui::StrokeKind::Inside);
    p.text(draw_rect.center(), egui::Align2::CENTER_CENTER, label, font_id, crate::constants::text());

    resp.clicked()
}

// ── icon_btn with tooltip ────────────────────────────
pub fn icon_btn_tip(ui: &mut egui::Ui, tex_id: egui::TextureId, active: bool, tip: &str) -> bool {
    let size = Vec2::splat(crate::constants::row_h() + 16.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());

    let t_hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());
    let t_active = ui.ctx().animate_bool(resp.id.with("active"), active);

    let mut bg = crate::constants::panel();
    bg = lerp_color(bg, crate::constants::hover(), t_hover);
    bg = lerp_color(bg, crate::constants::accent(), t_active);

    let is_down = resp.is_pointer_button_down_on();
    let offset = if is_down { 2.0 } else { 0.0 };
    let draw_rect = rect.translate(Vec2::new(0.0, offset));

    let p = ui.painter();
    if offset == 0.0 {
        p.rect_filled(rect.translate(Vec2::new(0.0, 2.0)), 0.0, crate::constants::border());
    }
    p.rect_filled(draw_rect, 0.0, bg);
    p.image(tex_id, draw_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);
    p.rect_stroke(draw_rect, 0.0, Stroke::new(4.0_f32, crate::constants::border()), egui::StrokeKind::Inside);

    let resp = resp.on_hover_text(tip);
    resp.clicked()
}

// ── separator ────────────────────────────────────────
pub fn separator(ui: &mut egui::Ui) {
    let h = crate::constants::row_h() + 16.0;
    let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, h), Sense::hover());
    ui.painter().vline(
        rect.center().x,
        rect.y_range(),
        Stroke::new(4.0_f32, crate::constants::border()),
    );
}

/// Checkbox row in pixel-art style: crisp rectangular border, accent fill when checked.
/// `font` is a scale multiplier against base 19 px, allowing reuse in dialogs and panels.
pub fn check_row(ui: &mut egui::Ui, label: &str, value: bool, font: f32, mut set: impl FnMut(bool)) {
    let sz = 19.0 * font;
    let row_h = sz + 12.0;
    let cbs = 18.0;
    let font_id = egui::FontId::proportional(sz);
    // Label width is measured upfront: the checkbox and label are centered as visible content
    let text_w = ui
        .fonts(|f| f.layout_no_wrap(label.to_owned(), font_id.clone(), crate::constants::text()).size().x);

    let w = ui.available_width().min(170.0);
    let (row, _) = ui.allocate_exact_size(Vec2::new(w, row_h), Sense::hover());

    // Center visible content (checkbox + spacing + text) within the row
    let content_w = cbs + 8.0 + text_w;
    let cb_rect = Rect::from_min_size(
        Pos2::new(
            row.center().x - content_w / 2.0,
            row.center().y - cbs * 0.5,
        ),
        Vec2::splat(cbs),
    );
    let p = ui.painter();
    p.text(
        Pos2::new(cb_rect.max.x + 8.0, row.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        font_id,
        crate::constants::text(),
    );
    p.rect_filled(cb_rect, 0.0, crate::constants::panel());
    p.rect_stroke(cb_rect, 0.0, Stroke::new(3.0_f32, crate::constants::border()), egui::StrokeKind::Outside);
    if value {
        p.rect_filled(cb_rect.shrink(4.0), 0.0, crate::constants::accent());
    }
    // Neutral id prefix so multiple dialogs can share check_row safely
    if ui.interact(row, egui::Id::new(("check_row", label)), Sense::click()).clicked()
        || ui.interact(cb_rect, egui::Id::new(("check_row_inner", label)), Sense::click()).clicked()
    {
        set(!value);
    }
}
