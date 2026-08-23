use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

fn thumb_color(doc: &super::Document, f: usize) -> ColorImage {
    let w = doc.width;
    let h = doc.height;
    let flat = doc.composite_at(f);
    let mut pix = Vec::with_capacity(w * h);
    for y in 0..h {
        for x in 0..w {
            let c = flat[y * w + x];
            if c.a() == 0 {
                let cb = if (x + y) % 2 == 0 { Color32::from_gray(200) } else { Color32::from_gray(180) };
                pix.push(cb);
            } else {
                pix.push(c);
            }
        }
    }
    ColorImage { size: [w, h], pixels: pix }
}

impl PixeshApp {
    pub(crate) fn ui_timeline(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::bottom("timeline")
            .frame(egui::Frame::new().fill(PANEL))
            .show_separator_line(false)
            .show(ctx, |ui| {
                let i = self.active_tab;
                let doc = &mut self.docs[i];

                ui.horizontal(|ui| {
                    ui.add_space(8.0);

                    // header
                    ui.painter().text(
                        Pos2::new(ui.cursor().min.x, ui.cursor().min.y + 2.0),
                        egui::Align2::LEFT_TOP,
                        "Frames",
                        egui::FontId::proportional(FONT_SZ * 1.3),
                        TEXT,
                    );
                    ui.add_space("Frames".len() as f32 * CHAR_W * 1.3 + 8.0);

                    separator(ui);
                    ui.add_space(8.0);

                    // play / pause
                    let play_label = if doc.playing { "Pause" } else { "Play" };
                    if toggle_btn(ui, play_label, doc.playing) {
                        doc.playing = !doc.playing;
                        doc.canvas_dirty = true;
                    }

                    if btn_min_w(ui, "Stop", 56.0) {
                        doc.playing = false;
                        doc.set_active_frame(0);
                    }

                    ui.add_space(10.0);

                    // fps
                    ui.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(FONT_SZ),
                    );
                    ui.label(egui::RichText::new("FPS").size(FONT_SZ).color(TEXT));
                    ui.add_sized(
                        Vec2::new(56.0, 30.0),
                        egui::DragValue::new(&mut doc.fps)
                            .range(1.0..=60.0)
                            .speed(1.0)
                            .suffix(""),
                    );

                    ui.add_space(10.0);
                    separator(ui);
                    ui.add_space(8.0);

                    // frame counter
                    ui.label(
                        egui::RichText::new(format!("{}/{}", doc.active_frame + 1, doc.frames))
                            .size(FONT_SZ)
                            .color(DIM),
                    );

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if btn_min_w(ui, "Del", 48.0) {
                            doc.delete_frame();
                        }
                        ui.add_space(4.0);
                        if btn_min_w(ui, "Dup", 48.0) {
                            doc.duplicate_frame();
                        }
                        ui.add_space(4.0);
                        if btn_min_w(ui, "Add", 48.0) {
                            doc.add_frame_blank();
                        }
                        ui.add_space(4.0);
                        if btn_min_w(ui, "<", 32.0) {
                            if doc.active_frame > 0 {
                                doc.move_frame(doc.active_frame, doc.active_frame - 1);
                            }
                        }
                        ui.add_space(4.0);
                        if btn_min_w(ui, ">", 32.0) {
                            if doc.active_frame + 1 < doc.frames {
                                doc.move_frame(doc.active_frame, doc.active_frame + 1);
                            }
                        }
                    });
                });

                ui.add_space(6.0);

                // frame thumbnails
                egui::ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add_space(8.0);
                        let th = 60.0;
                        for f in 0..doc.frames {
                            let cell = Vec2::new(th, th + 18.0);
                            let (rect, resp) = ui.allocate_exact_size(cell, Sense::click());
                            let is_active = f == doc.active_frame;

                            let p = ui.painter();
                            p.rect_filled(rect, 0.0, PANEL_LIGHT);

                            let img_rect = Rect::from_min_size(rect.min, Vec2::new(th, th));
                            let img = thumb_color(doc, f);
                            let tex = ui.ctx().load_texture(
                                format!("tl_{}_{}", i, f),
                                img,
                                egui::TextureOptions::NEAREST,
                            );
                            p.image(tex.id(), img_rect, Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

                            // frame number
                            p.text(
                                Pos2::new(rect.min.x + 4.0, rect.min.y + th + 2.0),
                                egui::Align2::LEFT_TOP,
                                &format!("{}", f + 1),
                                egui::FontId::proportional(FONT_SZ),
                                if is_active { ACCENT } else { DIM },
                            );

                            // border
                            let bw = if is_active { 4.0 } else { 2.0 };
                            let bc = if is_active { ACCENT } else { BORDER };
                            p.rect_stroke(rect, 0.0, Stroke::new(bw, bc), egui::StrokeKind::Outside);

                            if resp.clicked() {
                                doc.set_active_frame(f);
                            }
                        }
                        ui.add_space(8.0);
                    });
                });

                // separator line on top of panel
                let panel_left = ui.max_rect().left();
                let panel_right = ui.max_rect().right();
                let panel_top = ui.max_rect().top();
                ui.painter().hline(panel_left..=panel_right, panel_top, Stroke::new(8.0, BORDER));
            });
    }
}
