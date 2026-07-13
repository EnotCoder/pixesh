use eframe::egui;

use crate::color::*;
use crate::constants::Tool;
use super::PixeshApp;

// ── handle_input ─────────────────────────────────────
// вызывается каждый кадр из update() — обрабатывает клавиатуру, зум, панораму, пипетку
impl PixeshApp {
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context) {
        // ── хоткеи ──
        ctx.input_mut(|i| {
            // Ctrl+Z = отмена
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.undo();
            }
            // Ctrl+Shift+Z = повтор
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z) {
                self.redo();
            }
            // Ctrl+S = сохранить (открыть диалог экспорта)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                if !self.dialog_open() {
                    if self.export_name.is_empty() {
                        self.export_name = "pixesh.png".into();
                    }
                    self.show_export = true;
                }
            }
            // Ctrl+R = изменить размер холста (открыть диалог)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::R) {
                if !self.dialog_open() {
                    self.resize_w = self.width as f32;
                    self.resize_h = self.height as f32;
                    self.show_resize = true;
                }
            }
            // Ctrl+W = управление панелями
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::W) {
                if !self.dialog_open() {
                    self.show_panels = !self.show_panels;
                }
            }
            // Ctrl+H = настройки
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::H) {
                if !self.dialog_open() {
                    self.show_settings = !self.show_settings;
                }
            }
            // Ctrl+B = диалог размера кисти
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::B) {
                if !self.dialog_open() {
                    self.show_brush = true;
                }
            }
            // Ctrl+L = загрузить PNG (открыть диалог выбора файла)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::L) {
                let home = std::env::var("HOME").unwrap_or_else(|_| "/".into());
                if let Some(path) = rfd::FileDialog::new()
                    .set_directory(&home)
                    .add_filter("PNG", &["png"])
                    .pick_file()
                {
                    self.load_png(&path.to_string_lossy());
                }
            }
            // B = кисть, E = ластик, F = заливка, R = выделение (только когда не зажат Alt/Ctrl)
            if !i.modifiers.alt && !i.modifiers.ctrl {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::B) { self.tool = Tool::Brush; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::E) { self.tool = Tool::Eraser; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::F) { self.tool = Tool::Fill; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::R) { self.tool = Tool::Select; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::M) { self.tool = Tool::Move; }
                // G = переключить сетку
                if i.consume_key(egui::Modifiers::NONE, egui::Key::G) {
                    self.grid = !self.grid;
                }
                // W = темнее на 5
                if i.consume_key(egui::Modifiers::NONE, egui::Key::W) {
                    self.hsv_v = (self.hsv_v - 5.0).clamp(0.0, 255.0);
                    let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                    self.rgb_r = r as f32;
                    self.rgb_g = g as f32;
                    self.rgb_b = b as f32;
                    self.color = egui::Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                }
                // S = ярче на 5
                if i.consume_key(egui::Modifiers::NONE, egui::Key::S) {
                    self.hsv_v = (self.hsv_v + 5.0).clamp(0.0, 255.0);
                    let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                    self.rgb_r = r as f32;
                    self.rgb_g = g as f32;
                    self.rgb_b = b as f32;
                    self.color = egui::Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                }
                // A = прозрачнее на 5
                if i.consume_key(egui::Modifiers::NONE, egui::Key::A) {
                    self.rgb_a = (self.rgb_a - 5.0).clamp(0.0, 255.0);
                    self.color = egui::Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);
                }
                // D = непрозрачнее на 5
                if i.consume_key(egui::Modifiers::NONE, egui::Key::D) {
                    self.rgb_a = (self.rgb_a + 5.0).clamp(0.0, 255.0);
                    self.color = egui::Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);
                }
            }
            // Delete = удалить выделение
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                self.delete_selection();
            }
            // Escape = сначала закрыть диалоги, потом снять выделение
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.dialog_open() {
                    self.show_resize = false;
                    self.show_export = false;
                    self.show_brush = false;
                    self.show_panels = false;
                    self.show_settings = false;
                } else {
                    self.deselect();
                }
            }
            // Ctrl+D = снять выделение
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::D) {
                self.deselect();
            }
        });

        // ── зум колесом мыши / размер кисти (Shift) ──
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            if self.show_brush || ctx.input(|i| i.modifiers.shift) {
                // диалог открыт или Shift = размер кисти (без зума)
                let max = self.width.max(self.height) as f32;
                self.brush = (self.brush + scroll.signum()).clamp(1.0, max);
            } else {
                let old = self.zoom;
                self.zoom = (self.zoom + scroll * self.zoom_speed).clamp(1.0, 60.0);
                self.pan *= self.zoom / old;
            }
        }

        // ── панорама стрелками (блокируется если открыт диалог) ──
        if !self.dialog_open() {
            ctx.input(|i| {
                let speed = if i.modifiers.shift { self.arrow_speed * 4.0 } else { self.arrow_speed };
                if i.key_down(egui::Key::ArrowLeft)  { self.pan.x += speed; }
                if i.key_down(egui::Key::ArrowRight) { self.pan.x -= speed; }
                if i.key_down(egui::Key::ArrowUp)    { self.pan.y += speed; }
                if i.key_down(egui::Key::ArrowDown)  { self.pan.y -= speed; }
            });
        }

        // ── временная пипетка (Alt) ──
        ctx.input(|i| {
            let held = i.modifiers.alt;
            if held && self.tool_saved.is_none() {
                self.tool_saved = Some(self.tool);
                self.tool = Tool::Eyedropper;
            } else if !held {
                if let Some(saved) = self.tool_saved.take() {
                    self.tool = saved;
                }
            }
        });
    }
}
