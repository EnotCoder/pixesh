use std::sync::Arc;

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Vec2};

use crate::constants::*;
use crate::ui::*;
use super::PixeshApp;

impl PixeshApp {
    // верхняя панель инструментов: логотип, слайдер brush, иконки инструментов, grid, zoom
    pub(crate) fn ui_toolbar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("tools")
            .frame(egui::Frame::new().fill(PANEL))
            .show(ctx, |ui| {
                ui.add_space(6.0);
                ui.horizontal(|ui| {
                    ui.add_space(6.0);

                    // ── логотип ──
                    let logo_tex = self.logo_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../logo.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("logo", ci, egui::TextureOptions::NEAREST)
                    });
                    let logo_sz = Vec2::splat((ROW_H + 6.0) * 1.5);
                    let (lr, _) = ui.allocate_exact_size(logo_sz, Sense::hover());
                    ui.painter().image(logo_tex.id(), lr.translate(Vec2::new(0.0, 4.0)), Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)), Color32::WHITE);

                    // ── размер кисти ──
                    ui.add_space(6.0);
                    separator(ui);
                    ui.add_space(6.0);
                    slider(ui, "B", &mut self.brush, 1.0, 10.0);
                    ui.add_space(4.0);
                    separator(ui);
                    ui.add_space(4.0);

                    // ── иконки инструментов ──
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

                    let select_tex = self.select_tex.get_or_insert_with(|| {
                        let img = image::load_from_memory(include_bytes!("../../tex/select.png")).unwrap().into_rgba8();
                        let w = img.width() as usize;
                        let h = img.height() as usize;
                        let raw = img.into_raw();
                        let ci = ColorImage::from_rgba_unmultiplied([w, h], &raw);
                        ui.ctx().load_texture("select_icon", ci, egui::TextureOptions::NEAREST)
                    });
                    if icon_btn(ui, select_tex.id(), self.tool == Tool::Select) { self.tool = Tool::Select; }

                    // ── очистка ──
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
                            layer.pixels = Arc::new(vec![Color32::TRANSPARENT; self.width * self.height]);
                        }
                        self.canvas_dirty = true;
                    }

                    // ── сетка + зум ──
                    ui.add_space(6.0);
                    separator(ui);
                    ui.add_space(6.0);

                    checkbox(ui, "Grid", &mut self.grid);
                    ui.add_space(12.0);

                    let display_z = 61.0 - self.zoom;
                    let mut dz = display_z;
                    if slider(ui, "Z", &mut dz, 1.0, 60.0) {
                        self.zoom = (61.0 - dz).clamp(1.0, 60.0);
                    }
                    ui.add_space(6.0);
                });
                ui.add_space(6.0);
            });
    }
}
