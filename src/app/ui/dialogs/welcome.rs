use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::{Document, PixeshApp};
use crate::color::lerp_color;
use crate::constants::*;
use crate::ui::load_icon_texture;

use super::fit::fitted_dialog;

/// Pixel-art button with a drop shadow, hover fade and pressed offset.
fn action_button(
    ui: &egui::Ui,
    p: &egui::Painter,
    rect: Rect,
    id: egui::Id,
    label: &str,
    font_size: f32,
    accent: bool,
) -> bool {
    let resp = ui.interact(rect, id, Sense::click());
    let hover = ui.ctx().animate_bool(resp.id.with("hover"), resp.hovered());
    let down = resp.is_pointer_button_down_on();
    let offset = if down { 2.0 } else { 0.0 };
    let draw = rect.translate(Vec2::new(0.0, offset));

    let base = if accent { PANEL_LIGHT } else { PANEL };
    let mut bg = lerp_color(base, HOVER, hover);
    if down {
        bg = ACCENT;
    }
    let border = lerp_color(if accent { ACCENT } else { BORDER }, ACCENT, hover);

    if offset == 0.0 {
        p.rect_filled(rect.translate(Vec2::new(0.0, 2.0)), 0.0, BORDER);
    }
    p.rect_filled(draw, 0.0, bg);
    p.rect_stroke(
        draw,
        0.0,
        Stroke::new(2.0_f32, border),
        egui::StrokeKind::Outside,
    );
    p.text(
        draw.center(),
        egui::Align2::CENTER_CENTER,
        label,
        egui::FontId::proportional(font_size),
        TEXT,
    );
    resp.clicked()
}

/// Small uppercase section heading with an accent pixel and a trailing rule.
fn section_label(ui: &egui::Ui, p: &egui::Painter, rect: Rect, y: f32, text: &str) {
    let font = egui::FontId::proportional(14.0);
    let gal = ui.fonts(|f| f.layout_no_wrap(text.into(), font.clone(), DIM));
    let x = rect.min.x + 20.0;
    p.rect_filled(
        Rect::from_center_size(Pos2::new(x + 3.0, y), Vec2::splat(6.0)),
        0.0,
        ACCENT,
    );
    p.text(Pos2::new(x + 14.0, y), egui::Align2::LEFT_CENTER, text, font, DIM);
    p.hline(
        (x + 14.0 + gal.size().x + 10.0)..=(rect.max.x - 20.0),
        y,
        Stroke::new(1.0_f32, BORDER),
    );
}

impl PixeshApp {
    pub(crate) fn ui_welcome_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_welcome { return; }
        egui::Area::new("welcome_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(440.0, 500.0);
                fitted_dialog(ui, "welcome_scroll", size, |ui, rect| {
                    let p = ui.painter();
                    let pad = 20.0;
                    let content_w = rect.width() - pad * 2.0;

                    // ── header band ──
                    let header = Rect::from_min_size(
                        rect.min,
                        Vec2::new(rect.width(), 176.0),
                    );
                    p.rect_filled(header, 0.0, BG);
                    p.rect_filled(
                        Rect::from_min_size(rect.min, Vec2::new(rect.width(), 6.0)),
                        0.0,
                        ACCENT,
                    );
                    p.hline(
                        rect.x_range(),
                        header.max.y,
                        Stroke::new(2.0_f32, BORDER),
                    );

                    // ── logo in a framed tile ──
                    let logo_tex = self.logo_tex.get_or_insert_with(|| {
                        load_icon_texture(
                            ui,
                            "logo",
                            include_bytes!("../../../../img/logo.png"),
                        )
                    });
                    let logo_center = Pos2::new(rect.center().x, rect.min.y + 56.0);
                    p.rect_filled(
                        Rect::from_center_size(logo_center, Vec2::splat(92.0)),
                        0.0,
                        ACCENT.gamma_multiply(0.10),
                    );
                    let tile = Rect::from_center_size(logo_center, Vec2::splat(76.0));
                    p.rect_filled(tile, 0.0, PANEL_LIGHT);
                    p.rect_stroke(
                        tile,
                        0.0,
                        Stroke::new(2.0_f32, ACCENT),
                        egui::StrokeKind::Outside,
                    );
                    let lr = Rect::from_center_size(logo_center, Vec2::splat(60.0));
                    p.image(
                        logo_tex.id(),
                        lr,
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );

                    // ── title (two-tone) ──
                    let title_font = egui::FontId::proportional(30.0);
                    let t1 = "Welcome to ";
                    let t2 = "Pixesh";
                    let g1 = ui.fonts(|f| {
                        f.layout_no_wrap(t1.into(), title_font.clone(), TEXT)
                    });
                    let g2 = ui.fonts(|f| {
                        f.layout_no_wrap(t2.into(), title_font.clone(), ACCENT)
                    });
                    let total_w = g1.size().x + g2.size().x;
                    let tx = rect.center().x - total_w * 0.5;
                    let ty = rect.min.y + 104.0;
                    p.text(
                        Pos2::new(tx, ty),
                        egui::Align2::LEFT_TOP,
                        t1,
                        title_font.clone(),
                        TEXT,
                    );
                    p.text(
                        Pos2::new(tx + g1.size().x, ty),
                        egui::Align2::LEFT_TOP,
                        t2,
                        title_font,
                        ACCENT,
                    );

                    // ── subtitle ──
                    let sub = "A tiny pixel art editor";
                    let sub_font = egui::FontId::proportional(17.0);
                    let sgal = ui.fonts(|f| {
                        f.layout_no_wrap(sub.into(), sub_font.clone(), DIM)
                    });
                    p.text(
                        Pos2::new(rect.center().x - sgal.size().x * 0.5, rect.min.y + 146.0),
                        egui::Align2::LEFT_TOP,
                        sub,
                        sub_font,
                        DIM,
                    );

                    // ── primary actions ──
                    section_label(ui, p, rect, rect.min.y + 196.0, "START");
                    let primary_y = rect.min.y + 212.0;
                    let primary_w = (content_w - 10.0) * 0.5;
                    let new_clicked = action_button(
                        ui,
                        p,
                        Rect::from_min_size(
                            Pos2::new(rect.min.x + pad, primary_y),
                            Vec2::new(primary_w, 48.0),
                        ),
                        egui::Id::new("welcome_new"),
                        "New File",
                        20.0,
                        true,
                    );
                    let open_clicked = action_button(
                        ui,
                        p,
                        Rect::from_min_size(
                            Pos2::new(rect.min.x + pad + primary_w + 10.0, primary_y),
                            Vec2::new(primary_w, 48.0),
                        ),
                        egui::Id::new("welcome_open"),
                        "Open...",
                        20.0,
                        false,
                    );

                    // ── canvas size presets ──
                    section_label(ui, p, rect, rect.min.y + 282.0, "CANVAS SIZE");
                    let presets = [
                        "16x16", "32x32", "64x64",
                        "128x128", "512x512", "1280x720",
                    ];
                    let cols = 3;
                    let gap = 8.0;
                    let cell = Vec2::new(
                        (content_w - (cols - 1) as f32 * gap) / cols as f32,
                        42.0,
                    );
                    let grid_y = rect.min.y + 298.0;
                    let mut picked: Option<usize> = None;
                    for (i, label) in presets.iter().enumerate() {
                        let c = (i % cols) as f32;
                        let r = (i / cols) as f32;
                        let cell_rect = Rect::from_min_size(
                            Pos2::new(
                                rect.min.x + pad + c * (cell.x + gap),
                                grid_y + r * (cell.y + gap),
                            ),
                            cell,
                        );
                        if action_button(
                            ui,
                            p,
                            cell_rect,
                            egui::Id::new(("welcome_preset", i)),
                            label,
                            17.0,
                            false,
                        ) {
                            picked = Some(i);
                        }
                    }

                    // ── checkbox: show on startup ──
                    let cb_row = Rect::from_min_size(
                        Pos2::new(rect.min.x + pad, rect.min.y + 408.0),
                        Vec2::new(content_w, 28.0),
                    );
                    let cb_sz = 18.0;
                    let cb_rect = Rect::from_center_size(
                        Pos2::new(cb_row.min.x + cb_sz * 0.5, cb_row.center().y),
                        Vec2::splat(cb_sz),
                    );
                    let cb_resp =
                        ui.interact(cb_rect, egui::Id::new("welcome_cb"), Sense::click());
                    let t_resp =
                        ui.interact(cb_row, egui::Id::new("welcome_cb_row"), Sense::click());
                    if cb_resp.clicked() || t_resp.clicked() {
                        self.welcome_show_again = !self.welcome_show_again;
                    }
                    p.rect_filled(cb_rect, 0.0, PANEL);
                    p.rect_stroke(
                        cb_rect,
                        0.0,
                        Stroke::new(3.0_f32, BORDER),
                        egui::StrokeKind::Outside,
                    );
                    if self.welcome_show_again {
                        p.rect_filled(cb_rect.shrink(4.0), 0.0, ACCENT);
                    }
                    let cb_font = egui::FontId::proportional(17.0);
                    let label_x = cb_row.min.x + cb_sz + 8.0;
                    p.text(
                        Pos2::new(label_x, cb_row.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Show this window on startup",
                        cb_font,
                        TEXT,
                    );

                    // ── Close button ──
                    let close_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + pad, rect.max.y - 50.0),
                        Vec2::new(content_w, 40.0),
                    );
                    let close_clicked = action_button(
                        ui,
                        p,
                        close_rect,
                        egui::Id::new("welcome_close"),
                        "Close",
                        22.0,
                        true,
                    );
                    p.text(
                        Pos2::new(close_rect.max.x - 14.0, close_rect.center().y),
                        egui::Align2::RIGHT_CENTER,
                        "Enter / Esc",
                        egui::FontId::proportional(13.0),
                        DIM,
                    );

                    // ── handle close + persist ──
                    let enter = ctx.input_mut(|i| {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                    });
                    let esc = ctx.input_mut(|i| {
                        i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)
                    });
                    if close_clicked || enter || esc {
                        self.show_welcome = false;
                        return;
                    }

                    // ── handle quick actions ──
                    if new_clicked {
                        self.new_tab(Document::new_sized("Untitled", 16, 16));
                        self.show_welcome = false;
                        return;
                    }
                    if open_clicked {
                        self.open_file_dialog();
                        self.show_welcome = false;
                        return;
                    }
                    if let Some(i) = picked {
                        let label = presets[i];
                        let (w, h) = parse_canvas_preset(label).unwrap_or((16, 16));
                        self.new_tab(Document::new_sized("Untitled", w, h));
                        // Large presets start with the round brush so the
                        // first stroke already looks right on a big canvas.
                        if matches!(label, "512x512" | "1280x720") {
                            self.tool = Tool::Brush;
                            self.brush_shape = BrushShape::Round;
                        }
                        self.show_welcome = false;
                    }
                });
            });
    }

    fn new_tab(&mut self, mut doc: Document) {
        doc.needs_zoom_fit = true;
        self.docs.push(doc);
        self.active_tab = self.docs.len() - 1;
    }

    fn open_file_dialog(&mut self) {
        #[cfg(feature = "rfd")]
        {
            let home = crate::app::config::default_dir();
            if let Some(path) = rfd::FileDialog::new()
                .set_directory(&home)
                .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tga"])
                .pick_file()
            {
                self.open_path(&path);
            }
        }
    }
}

/// Parses a "WxH" canvas preset label from the welcome grid.
fn parse_canvas_preset(label: &str) -> Option<(usize, usize)> {
    let (w, h) = label.split_once('x')?;
    Some((w.parse().ok()?, h.parse().ok()?))
}

#[cfg(test)]
mod tests {
    use super::parse_canvas_preset;

    #[test]
    fn parses_square_and_rectangular_presets() {
        assert_eq!(parse_canvas_preset("16x16"), Some((16, 16)));
        assert_eq!(parse_canvas_preset("512x512"), Some((512, 512)));
        assert_eq!(parse_canvas_preset("1280x720"), Some((1280, 720)));
    }

    #[test]
    fn rejects_non_preset_labels() {
        assert_eq!(parse_canvas_preset("New File"), None);
        assert_eq!(parse_canvas_preset("Open..."), None);
        assert_eq!(parse_canvas_preset("16x"), None);
    }
}
