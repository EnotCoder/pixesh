use eframe::egui;

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
            // Ctrl+Y = повтор
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Y) {
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
                if i.consume_key(egui::Modifiers::NONE, egui::Key::B) { self.tool = Tool::Fill; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::E) { self.tool = Tool::Eraser; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::P) { self.tool = Tool::Brush; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::R) { self.tool = Tool::Select; }
                // F = открыть диалог размера кисти
                if i.consume_key(egui::Modifiers::NONE, egui::Key::F) {
                    if !self.dialog_open() {
                        self.show_brush = true;
                    }
                }
            }
            // Delete = удалить выделение
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                self.delete_selection();
            }
            // Escape = закрыть диалоги / сбросить выделение
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.sel.is_some() {
                    self.deselect();
                } else {
                    self.show_resize = false;
                    self.show_export = false;
                    self.show_brush = false;
                }
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
                self.zoom = (self.zoom - scroll * 0.2).clamp(1.0, 60.0);
                self.pan *= self.zoom / old;
            }
        }

        // ── панорама стрелками (блокируется если открыт диалог) ──
        if !self.show_resize && !self.show_export && !self.show_brush {
            ctx.input(|i| {
                let speed = if i.modifiers.shift { 80.0 } else { 20.0 };
                if i.key_down(egui::Key::ArrowLeft)  { self.pan.x += speed; }
                if i.key_down(egui::Key::ArrowRight) { self.pan.x -= speed; }
                if i.key_down(egui::Key::ArrowUp)    { self.pan.y += speed; }
                if i.key_down(egui::Key::ArrowDown)  { self.pan.y -= speed; }
            });
        }

        // ── временная пипетка ──
        ctx.input(|i| {
            let held = i.modifiers.alt || i.modifiers.ctrl;
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
