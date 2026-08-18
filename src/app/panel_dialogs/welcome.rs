use eframe::egui::{self, Color32, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::{config, Document, PixeshApp};
use crate::constants::*;
use crate::ui::load_icon_texture;

impl PixeshApp {
    pub(crate) fn ui_welcome_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_welcome { return; }
        egui::Area::new("welcome_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(480.0, 440.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, 0.0, PANEL);
                p.rect_stroke(rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);

                // ── logo ──
                let logo_tex = self.logo_tex.get_or_insert_with(|| {
                    load_icon_texture(ui, "logo", include_bytes!("../../../logo.png"))
                });
                let logo_sz = Vec2::splat(72.0);
                let lr = Rect::from_center_size(Pos2::new(rect.center().x, rect.min.y + 64.0), logo_sz);
                p.image(logo_tex.id(), lr, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

                // ── title ──
                let title = "Welcome to Pixesh";
                let title_font = egui::FontId::proportional(34.0);
                let tgal = ui.fonts(|f| f.layout_no_wrap(title.into(), title_font.clone(), TEXT));
                p.text(
                    Pos2::new(rect.center().x - tgal.size().x * 0.5, rect.min.y + 104.0),
                    egui::Align2::LEFT_TOP,
                    title,
                    title_font,
                    TEXT,
                );

                // ── subtitle ──
                let sub = "A tiny pixel art editor";
                let sub_font = egui::FontId::proportional(20.0);
                let sgal = ui.fonts(|f| f.layout_no_wrap(sub.into(), sub_font.clone(), DIM));
                p.text(
                    Pos2::new(rect.center().x - sgal.size().x * 0.5, rect.min.y + 150.0),
                    egui::Align2::LEFT_TOP,
                    sub,
                    sub_font,
                    DIM,
                );

                // ── quick actions grid (3 cols) ──
                let grid_y = rect.min.y + 186.0;
                let cols = 3;
                let gap = 10.0;
                let cell = Vec2::new((rect.width() - (cols + 1) as f32 * gap) / cols as f32, 52.0);
                let actions: [(usize, usize); 6] = [
                    (0, 0), (1, 0), (2, 0),
                    (0, 1), (1, 1), (2, 1),
                ];
                let mut picked: Option<usize> = None;
                for (i, (cx, cy)) in actions.iter().enumerate() {
                    let c = *cx as f32;
                    let r = *cy as f32;
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + gap + c * (cell.x + gap), grid_y + r * (cell.y + gap)),
                        cell,
                    );
                    let resp = ui.interact(cell_rect, egui::Id::new(("welcome_action", i)), Sense::click());
                    let bg = if resp.clicked() { ACCENT } else if resp.hovered() { HOVER } else { PANEL_LIGHT };
                    p.rect_filled(cell_rect, 0.0, bg);
                    p.rect_stroke(cell_rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
                    if resp.clicked() {
                        picked = Some(i);
                    }
                }
                let labels = ["New File", "Open...", "16x16", "32x32", "64x64", "128x128"];
                let a_font = egui::FontId::proportional(22.0);
                for (i, (cx, cy)) in actions.iter().enumerate() {
                    let c = *cx as f32;
                    let r = *cy as f32;
                    let cell_rect = Rect::from_min_size(
                        Pos2::new(rect.min.x + gap + c * (cell.x + gap), grid_y + r * (cell.y + gap)),
                        cell,
                    );
                    p.text(cell_rect.center(), egui::Align2::CENTER_CENTER, labels[i], a_font.clone(), TEXT);
                }

                // ── checkbox: show on startup ──
                let cb_row = Rect::from_min_size(
                    Pos2::new(rect.min.x + gap, rect.min.y + 186.0 + cell.y * 2.0 + gap * 2.0 + 14.0),
                    Vec2::new(rect.width() - gap * 2.0, 32.0),
                );
                let cb_sz = 26.0;
                let cb_rect = Rect::from_center_size(
                    Pos2::new(cb_row.min.x + cb_sz * 0.5, cb_row.center().y),
                    Vec2::splat(cb_sz),
                );
                let cb_resp = ui.interact(cb_rect, egui::Id::new("welcome_cb"), Sense::click());
                let t_resp = ui.interact(cb_row, egui::Id::new("welcome_cb_row"), Sense::click());
                if cb_resp.clicked() || t_resp.clicked() {
                    self.welcome_show_again = !self.welcome_show_again;
                }
                p.rect_filled(cb_rect, 0.0, PANEL);
                p.rect_stroke(cb_rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);
                if self.welcome_show_again {
                    p.rect_filled(cb_rect.shrink(5.0), 0.0, ACCENT);
                }
                let cb_font = egui::FontId::proportional(22.0);
                let label_x = cb_row.min.x + cb_sz + 10.0;
                p.text(
                    Pos2::new(label_x, cb_row.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Show this window on startup",
                    cb_font,
                    TEXT,
                );

                // ── Close button ──
                let close_rect = Rect::from_min_size(
                    Pos2::new(rect.min.x + gap, rect.max.y - 54.0),
                    Vec2::new(rect.width() - gap * 2.0, 44.0),
                );
                let close_resp = ui.interact(close_rect, egui::Id::new("welcome_close"), Sense::click());
                let close_bg = if close_resp.clicked() { ACCENT } else if close_resp.hovered() { HOVER } else { PANEL };
                p.rect_filled(close_rect, 0.0, close_bg);
                p.rect_stroke(close_rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);
                p.text(close_rect.center(), egui::Align2::CENTER_CENTER, "Close", egui::FontId::proportional(24.0), TEXT);

                // ── handle close + persist ──
                let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                let esc = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
                if close_resp.clicked() || enter || esc {
                    config::save_welcome_show_again(self.welcome_show_again);
                    self.show_welcome = false;
                    return;
                }

                // ── handle quick action ──
                if let Some(i) = picked {
                    match labels[i] {
                        "New File" => self.new_tab(Document::new_sized("Untitled", 16, 16)),
                        "Open..." => self.open_file_dialog(),
                        label => {
                            let side: usize = label.trim_end_matches("x")
                                .split('x').next().and_then(|s| s.parse().ok())
                                .unwrap_or(16);
                            self.new_tab(Document::new_sized("Untitled", side, side));
                        }
                    }
                    config::save_welcome_show_again(self.welcome_show_again);
                    self.show_welcome = false;
                }
            });
    }

    fn new_tab(&mut self, mut doc: Document) {
        doc.needs_zoom_fit = true;
        self.docs.push(doc);
        self.active_tab = self.docs.len() - 1;
    }

    fn open_file_dialog(&mut self) {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
        if let Some(path) = rfd::FileDialog::new()
            .set_directory(&home)
            .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tga"])
            .pick_file()
        {
            let path_str = path.to_string_lossy().to_string();
            let name = std::path::Path::new(&path_str)
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_else(|| "Untitled".into());
            let mut doc = Document::new(&name);
            doc.load_png(&path_str);
            self.docs.push(doc);
            self.active_tab = self.docs.len() - 1;
        }
    }
}