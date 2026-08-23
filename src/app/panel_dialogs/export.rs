use eframe::egui::{self, Rect, Sense, Stroke, Vec2};
use crate::app::PixeshApp;
use crate::constants::*;
use crate::ui::*;

impl PixeshApp {
    pub(crate) fn ui_export_dialog(&mut self, ctx: &egui::Context) {
        if !self.show_export { return; }
        let i = self.active_tab;
        egui::Area::new("export_dialog".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let size = Vec2::new(380.0, 484.0);
                let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                let pad = 10.0;
                let btn_h = 44.0;

                let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                let display = if self.docs[i].export_path.is_empty() || self.docs[i].export_path == home {
                    "home".to_string()
                } else {
                    std::path::Path::new(&self.docs[i].export_path)
                        .file_name()
                        .map(|n| n.to_string_lossy().into_owned())
                        .unwrap_or_else(|| self.docs[i].export_path.clone())
                };
                let display = if display.chars().count() > 14 {
                    format!("{}...", display.chars().take(14).collect::<String>())
                } else {
                    display
                };
                let folder_text = format!("Folder: {}", display);

                let body_font = egui::FontId::proportional(28.0);
                let title_font = egui::FontId::proportional(32.0);
                let small_font = egui::FontId::proportional(22.0);
                let toggle_font = egui::FontId::proportional(18.0);

                {
                    let p = ui.painter();
                    p.rect_filled(rect, 0.0, PANEL);
                    p.rect_stroke(rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);

                    let title = "Export PNG";
                    let title_galley = ui.fonts(|f| f.layout_no_wrap(title.into(), title_font.clone(), TEXT));
                    let title_x = rect.center().x - title_galley.size().x * 0.5;
                    p.text(
                        egui::pos2(title_x, rect.min.y + 12.0),
                        egui::Align2::LEFT_TOP,
                        title,
                        title_font,
                        TEXT,
                    );

                    p.text(
                        egui::pos2(rect.min.x + pad, rect.min.y + 56.0 + 8.0),
                        egui::Align2::LEFT_TOP,
                        &folder_text,
                        body_font.clone(),
                        TEXT,
                    );

                    let file_row_y = rect.min.y + 116.0 + 8.0;
                    p.text(
                        egui::pos2(rect.min.x + pad, file_row_y),
                        egui::Align2::LEFT_TOP,
                        "File:",
                        body_font.clone(),
                        TEXT,
                    );

                    p.text(
                        egui::pos2(rect.min.x + pad, rect.min.y + 172.0),
                        egui::Align2::LEFT_TOP,
                        "Scale:",
                        small_font.clone(),
                        TEXT,
                    );
                    p.text(
                        egui::pos2(rect.min.x + pad, rect.min.y + 250.0),
                        egui::Align2::LEFT_TOP,
                        "Background:",
                        small_font.clone(),
                        TEXT,
                    );
                }

                let file_edit_x = rect.min.x + pad + 80.0;
                let file_edit_w = rect.max.x - pad - file_edit_x;
                let file_edit_rect = egui::Rect::from_min_size(
                    egui::pos2(file_edit_x, rect.min.y + 116.0),
                    Vec2::new(file_edit_w, btn_h),
                );
                ui.put(file_edit_rect, egui::TextEdit::singleline(&mut self.docs[i].export_name).desired_width(file_edit_w));

                let dot_btn_rect = egui::Rect::from_min_size(
                    egui::pos2(rect.max.x - pad - btn_h, rect.min.y + 56.0),
                    Vec2::splat(btn_h),
                );
                let dot_resp = ui.interact(dot_btn_rect, egui::Id::new("export_dot"), egui::Sense::click());
                if dot_resp.clicked() {
                    if let Some(path) = rfd::FileDialog::new().set_directory(&home).pick_folder() {
                        self.docs[i].export_path = path.to_string_lossy().into();
                    }
                }

                {
                    let p = ui.painter();
                    let dot_bg = if dot_resp.clicked() { ACCENT } else if dot_resp.hovered() { HOVER } else { PANEL };
                    p.rect_filled(dot_btn_rect, 0.0, dot_bg);
                    p.rect_stroke(dot_btn_rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);
                    p.text(dot_btn_rect.center(), egui::Align2::CENTER_CENTER, "...", body_font.clone(), TEXT);
                }

                // scale buttons
                let mut new_scale = self.export_scale;
                {
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + pad, rect.min.y + 192.0),
                        Vec2::new(rect.width() - pad * 2.0, btn_h),
                    );
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::left_to_right(egui::Align::Center))
                            .max_rect(btn_rect),
                    );
                    row_ui.style_mut().text_styles.insert(egui::TextStyle::Button, toggle_font.clone());
                    for s in [1, 2, 4, 8] {
                        if toggle_btn(&mut row_ui, &format!("x{}", s), self.export_scale == s) {
                            new_scale = s;
                        }
                    }
                }
                self.export_scale = new_scale;

                // background buttons
                let mut new_bg = self.export_bg;
                {
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + pad, rect.min.y + 270.0),
                        Vec2::new(rect.width() - pad * 2.0, btn_h),
                    );
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::left_to_right(egui::Align::Center))
                            .max_rect(btn_rect),
                    );
                    row_ui.style_mut().text_styles.insert(egui::TextStyle::Button, toggle_font.clone());
                    for (on, bg) in [("None", ExportBg::Transparent), ("White", ExportBg::White), ("Black", ExportBg::Black), ("Check", ExportBg::Checker)] {
                        if toggle_btn(&mut row_ui, on, self.export_bg == bg) {
                            new_bg = bg;
                        }
                    }
                }
                self.export_bg = new_bg;

                // frames mode (current vs all frames as sprite sheet)
                let mut new_sheet = self.export_sheet;
                {
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x + pad, rect.min.y + 318.0),
                        Vec2::new(rect.width() - pad * 2.0, btn_h),
                    );
                    let mut row_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::left_to_right(egui::Align::Center))
                            .max_rect(btn_rect),
                    );
                    row_ui.style_mut().text_styles.insert(egui::TextStyle::Button, toggle_font.clone());
                    let sheet_on = self.docs[i].frames > 1 && self.export_sheet;
                    if toggle_btn(&mut row_ui, "Current", !sheet_on) {
                        new_sheet = false;
                    }
                    if self.docs[i].frames > 1 && toggle_btn(&mut row_ui, "Sheet", sheet_on) {
                        new_sheet = true;
                    }
                }
                self.export_sheet = new_sheet;

                // layers checkbox
                {
                    let cb_row = Rect::from_min_size(
                        egui::pos2(rect.min.x + pad, rect.min.y + 366.0),
                        Vec2::new(rect.width() - pad * 2.0, 32.0),
                    );
                    let cb_sz = 26.0;
                    let cb_rect = Rect::from_center_size(
                        egui::pos2(cb_row.min.x + cb_sz * 0.5, cb_row.center().y),
                        Vec2::splat(cb_sz),
                    );
                    let cb_resp = ui.interact(cb_rect, egui::Id::new("export_layers_cb"), Sense::click());
                    let row_resp = ui.interact(cb_row, egui::Id::new("export_layers_cb_row"), Sense::click());
                    if cb_resp.clicked() || row_resp.clicked() {
                        self.export_layers = !self.export_layers;
                    }
                    let p = ui.painter();
                    p.rect_filled(cb_rect, 0.0, PANEL);
                    p.rect_stroke(cb_rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);
                    if self.export_layers {
                        p.rect_filled(cb_rect.shrink(5.0), 0.0, ACCENT);
                    }
                    p.text(
                        egui::pos2(cb_row.min.x + cb_sz + 10.0, cb_row.center().y),
                        egui::Align2::LEFT_CENTER,
                        "Each layer",
                        small_font.clone(),
                        TEXT,
                    );
                }

                // size preview
                {
                    let s = self.export_scale.max(1) as usize;
                    let (ow, oh) = if self.export_sheet && self.docs[i].frames > 1 {
                        (self.docs[i].width * self.docs[i].frames, self.docs[i].height)
                    } else {
                        (self.docs[i].width, self.docs[i].height)
                    };
                    let out = format!("Out: {}x{} px", ow * s, oh * s);
                    let p = ui.painter();
                    p.text(
                        egui::pos2(rect.min.x + pad, rect.min.y + 406.0),
                        egui::Align2::LEFT_TOP,
                        out,
                        small_font.clone(),
                        DIM,
                    );
                }

                let mut save_clicked = false;
                {
                    let btn_y = rect.max.y - btn_h;
                    let btn_rect = egui::Rect::from_min_size(
                        egui::pos2(rect.min.x, btn_y),
                        Vec2::new(rect.width(), btn_h),
                    );
                    let mut btn_ui = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::left_to_right(egui::Align::Center))
                            .max_rect(btn_rect),
                    );
                    btn_ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    let spacing = btn_ui.style().spacing.item_spacing.x;
                    let half_w = (btn_ui.available_width() - spacing) / 2.0;
                    if btn_min_w(&mut btn_ui, "Save", half_w) {
                        save_clicked = true;
                    }
                    if btn_min_w(&mut btn_ui, "Cancel", half_w) {
                        self.show_export = false;
                    }
                }

                let enter = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter));
                if enter || save_clicked {
                    let dir = if self.docs[i].export_path.is_empty() { home.clone() } else { self.docs[i].export_path.clone() };
                    let scale = self.export_scale.max(1) as u32;
                    let base = if let Some(p) = self.docs[i].export_name.rfind('.') {
                        self.docs[i].export_name[..p].to_string()
                    } else {
                        self.docs[i].export_name.clone()
                    };
                    
                    if self.export_sheet && self.docs[i].frames > 1 {
                        let path = format!("{}/{}.png", dir, base);
                        let _ = self.docs[i].save_png_sheet(&path, scale, self.export_bg);
                    } else if self.docs[i].frames > 1 {
                        // Auto-export to multiple PNGs if animation exists and not sheet
                        let _ = self.docs[i].save_frame_pngs(&dir, &base, scale, self.export_bg);
                    } else {
                        let path = format!("{}/{}.png", dir, base);
                        let _ = self.docs[i].save_png(&path, scale, self.export_bg);
                        if self.export_layers {
                            let _ = self.docs[i].save_layer_pngs(&dir, scale, self.export_bg);
                        }
                    }
                    
                    self.docs[i].unsaved = false;
                    self.show_export = false;
                }
            });
    }
}