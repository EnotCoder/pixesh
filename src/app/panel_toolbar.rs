use eframe::egui::{self, Color32, ColorImage, Sense, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

impl PixeshApp {
    pub(crate) fn ui_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tools")
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    let title = "Pixesh";
                    let title_w = title.len() as f32 * CHAR_W;
                    let (tr, _) = ui.allocate_exact_size(
                        Vec2::new(title_w + 8.0, ROW_H + 4.0),
                        Sense::hover(),
                    );
                    ui.painter().text(
                        tr.min + Vec2::new(4.0, 2.0),
                        egui::Align2::LEFT_TOP,
                        title,
                        egui::FontId::proportional(FONT_SZ),
                        ACCENT,
                    );

                    separator(ui);
                    slider(ui, "B", &mut self.brush, 1.0, 10.0);
                    separator(ui);

                    let brush_tex = self.brush_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/brush.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("brush_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, brush_tex.id(), self.tool == Tool::Brush) { self.tool = Tool::Brush; }

                    let eraser_tex = self.eraser_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/eraser.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("eraser_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, eraser_tex.id(), self.tool == Tool::Eraser) { self.tool = Tool::Eraser; }

                    let fill_tex = self.fill_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/fill.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("fill_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, fill_tex.id(), self.tool == Tool::Fill) { self.tool = Tool::Fill; }

                    let drop_tex = self.drop_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/drop.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("drop_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, drop_tex.id(), self.tool == Tool::Eyedropper) { self.tool = Tool::Eyedropper; }

                    separator(ui);

                    let clear_tex = self.clear_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/clear.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("clear_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, clear_tex.id(), false) {
                        self.push_undo();
                        for layer in &mut self.layers {
                            layer.pixels = vec![Color32::TRANSPARENT; self.width * self.height];
                        }
                    }

                    checkbox(ui, "Grid", &mut self.grid);

                    let display_z = 61.0 - self.zoom;
                    let mut dz = display_z;
                    if slider(ui, "Z", &mut dz, 1.0, 60.0) {
                        self.zoom = (61.0 - dz).clamp(1.0, 60.0);
                    }
                });
                ui.add_space(4.0);
            });
    }
}
