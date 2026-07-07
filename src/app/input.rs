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
                self.export_name = "pixesh.png".into();
                self.show_export = true;
            }
            // Ctrl+R = изменить размер холста (открыть диалог)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::R) {
                self.resize_w = self.width as f32;
                self.resize_h = self.height as f32;
                self.show_resize = true;
            }
            // Ctrl+L = загрузить PNG (открыть диалог выбора файла)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::L) {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("PNG", &["png"])
                    .pick_file()
                {
                    self.load_png(&path.to_string_lossy());
                }
            }
            // B = кисть, E = ластик, F = заливка (только когда не зажат Alt/Ctrl)
            if !i.modifiers.alt && !i.modifiers.ctrl {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::B) { self.tool = Tool::Brush; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::E) { self.tool = Tool::Eraser; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::F) { self.tool = Tool::Fill; }
            }
            // Escape = закрыть любой открытый диалог
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                self.show_resize = false;
                self.show_export = false;
            }
        });

        // ── зум колесом мыши ──
        // прокрутка в любом месте окна приближает/отдаляет
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let old = self.zoom;
            self.zoom = (self.zoom - scroll * 0.2).clamp(1.0, 60.0);
            // масштабируем панораму чтобы центр не прыгал
            self.pan *= self.zoom / old;
        }

        // ── панорама стрелками ──
        // когда холст не влезает в окно, можно двигать его стрелками
        ctx.input(|i| {
            let speed = if i.modifiers.shift { 80.0 } else { 20.0 };
            if i.key_down(egui::Key::ArrowLeft)  { self.pan.x += speed; }
            if i.key_down(egui::Key::ArrowRight) { self.pan.x -= speed; }
            if i.key_down(egui::Key::ArrowUp)    { self.pan.y += speed; }
            if i.key_down(egui::Key::ArrowDown)  { self.pan.y -= speed; }
        });

        // ── временная пипетка ──
        // пока зажат Alt или Ctrl, инструмент подменяется на Eyedropper
        // когда отпускаешь — возвращается предыдущий
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
