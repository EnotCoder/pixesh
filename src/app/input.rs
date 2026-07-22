use eframe::egui;

use crate::color::*;
use crate::constants::Tool;
use super::{PixeshApp, Document};

impl PixeshApp {
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context) {
        let text_focused = ctx.memory(|m| m.focused().is_some());
        ctx.input_mut(|i| {
            // Ctrl+Z = undo
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.docs[self.active_tab].undo();
            }
            // Ctrl+Shift+Z = redo
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z) {
                self.docs[self.active_tab].redo();
            }
            // Ctrl+S = export dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::S) {
                if !self.dialog_open() {
                    let tab = self.active_tab;
                    if self.docs[tab].export_name.is_empty() {
                        self.docs[tab].export_name = "pixesh.png".into();
                    }
                    self.show_export = true;
                }
            }
            // Ctrl+R = resize dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::R) {
                if !self.dialog_open() {
                    let tab = self.active_tab;
                    self.resize_w = self.docs[tab].width as f32;
                    self.resize_h = self.docs[tab].height as f32;
                    self.show_resize = true;
                }
            }
            // Ctrl+W = panels dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::W) {
                if !self.dialog_open() {
                    self.show_panels = !self.show_panels;
                }
            }
            // Ctrl+H = settings
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::H) {
                if !self.dialog_open() {
                    self.show_settings = !self.show_settings;
                }
            }
            // Ctrl+B = brush size dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::B) {
                if !self.dialog_open() {
                    self.show_brush = true;
                }
            }
            // Ctrl+I = scale dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::I) {
                if !self.dialog_open() {
                    let tab = self.active_tab;
                    self.scale_w = self.docs[tab].width as f32;
                    self.scale_h = self.docs[tab].height as f32;
                    self.show_scale = true;
                }
            }
            // Ctrl+L = load image (new tab)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::L) {
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
            // Ctrl+Tab = next tab
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Tab) {
                let n = self.docs.len();
                if n > 1 {
                    self.active_tab = (self.active_tab + 1) % n;
                }
            }
            // tool keys
            if !i.modifiers.alt && !i.modifiers.ctrl && !text_focused {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::B) { self.tool = Tool::Brush; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::E) { self.tool = Tool::Eraser; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::F) { self.tool = Tool::Fill; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::R) { self.tool = Tool::Select; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::M) { self.tool = Tool::Move; }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::G) {
                    self.docs[self.active_tab].grid = !self.docs[self.active_tab].grid;
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::W) {
                    self.hsv_v = (self.hsv_v - 5.0).clamp(0.0, 255.0);
                    let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                    self.rgb_r = r as f32;
                    self.rgb_g = g as f32;
                    self.rgb_b = b as f32;
                    self.color = egui::Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::S) {
                    self.hsv_v = (self.hsv_v + 5.0).clamp(0.0, 255.0);
                    let (r, g, b) = hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
                    self.rgb_r = r as f32;
                    self.rgb_g = g as f32;
                    self.rgb_b = b as f32;
                    self.color = egui::Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::A) {
                    self.rgb_a = (self.rgb_a - 5.0).clamp(0.0, 255.0);
                    self.color = egui::Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::D) {
                    self.rgb_a = (self.rgb_a + 5.0).clamp(0.0, 255.0);
                    self.color = egui::Color32::from_rgba_unmultiplied(self.rgb_r as u8, self.rgb_g as u8, self.rgb_b as u8, self.rgb_a as u8);
                }
            }
            // Delete
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                self.docs[self.active_tab].delete_selection();
            }
            // Enter = crop to selection
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                if self.docs[self.active_tab].sel.is_some() && !self.dialog_open() {
                    self.docs[self.active_tab].crop_to_selection();
                }
            }
            // Escape
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.dialog_open() {
                    self.show_resize = false;
                    self.show_export = false;
                    self.show_brush = false;
                    self.show_panels = false;
                    self.show_settings = false;
                    self.show_scale = false;
                    self.show_quit_dialog = false;
                } else {
                    self.docs[self.active_tab].deselect();
                }
            }
            // Ctrl+D = deselect
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::D) {
                self.docs[self.active_tab].deselect();
            }
        });

        // scroll zoom / brush size
        let scroll = ctx.input(|i| i.raw_scroll_delta.y);
        if scroll != 0.0 {
            let scroll_norm = scroll.signum();
            if self.show_brush || ctx.input(|i| i.modifiers.shift) {
                let tab = self.active_tab;
                let max = self.docs[tab].width.max(self.docs[tab].height) as f32;
                self.brush = (self.brush + scroll_norm).clamp(1.0, max);
            } else {
                let tab = self.active_tab;
                let doc = &mut self.docs[tab];
                let old = doc.zoom;
                doc.zoom = (doc.zoom * (1.0 + scroll_norm * self.zoom_speed * 0.1)).clamp(0.1, 60.0);
                doc.pan *= doc.zoom / old;
            }
        }

        // arrow pan
        if !self.dialog_open() {
            let tab = self.active_tab;
            ctx.input(|i| {
                let speed = if i.modifiers.shift { self.arrow_speed * 4.0 } else { self.arrow_speed };
                if i.key_down(egui::Key::ArrowLeft)  { self.docs[tab].pan.x += speed; }
                if i.key_down(egui::Key::ArrowRight) { self.docs[tab].pan.x -= speed; }
                if i.key_down(egui::Key::ArrowUp)    { self.docs[tab].pan.y += speed; }
                if i.key_down(egui::Key::ArrowDown)  { self.docs[tab].pan.y -= speed; }
            });
        }

        // temporary eyedropper (Alt)
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

    pub(crate) fn handle_eyedropper(&mut self, px: i32, py: i32) {
        let tab = self.active_tab;
        let w = self.docs[tab].width as i32;
        let h = self.docs[tab].height as i32;
        if px < 0 || px >= w || py < 0 || py >= h { return; }
        let idx = (py * w + px) as usize;
        let mut c = egui::Color32::TRANSPARENT;
        for layer in &self.docs[tab].layers {
            if !layer.visible { continue; }
            let p = layer.pixels[idx];
            if p != egui::Color32::TRANSPARENT {
                c = p;
                break;
            }
        }
        self.color = c;
        self.rgb_r = c.r() as f32;
        self.rgb_g = c.g() as f32;
        self.rgb_b = c.b() as f32;
        self.rgb_a = c.a() as f32;
        let (h_, s, v) = rgb_to_hsv(c.r(), c.g(), c.b());
        self.hsv_h = h_;
        self.hsv_s = s;
        self.hsv_v = v;
        if c != egui::Color32::TRANSPARENT {
            self.color_history.retain(|&x| x != c);
            self.color_history.push(c);
            if self.color_history.len() > 4 {
                self.color_history.remove(0);
            }
        }
    }
}
