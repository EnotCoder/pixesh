use eframe::egui;

use crate::color::*;
use crate::constants::Tool;
use super::PixeshApp;
#[cfg(feature = "rfd")]
use super::Document;

impl PixeshApp {
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context) {
        let text_focused = ctx.memory(|m| m.focused().is_some());
        // гасим подсказку из статус-бара, когда она отжила своё
        let now = ctx.input(|i| i.time);
        if now > self.status_hint_until {
            self.status_hint.clear();
        }
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
            // Ctrl+I = scale dialog
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::I) {
                if !self.dialog_open() {
                    let tab = self.active_tab;
                    self.scale_w = self.docs[tab].width as f32;
                    self.scale_h = self.docs[tab].height as f32;
                    self.show_scale = true;
                }
            }
            // Ctrl+Shift+E = effects panel
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::E) {
                if self.show_effects {
                    self.show_effects = false;
                    if self.tool == Tool::Effects { self.tool = Tool::Brush; }
                } else if !self.dialog_open() {
                    self.tool = Tool::Effects;
                    self.show_effects = true;
                }
            }
            // Ctrl+L = load image (new tab)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::L) {
                #[cfg(feature = "rfd")]
                {
                    let home = crate::app::config::default_dir();
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
                if i.consume_key(egui::Modifiers::NONE, egui::Key::T) { self.tool = Tool::Text; }

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
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Period) {
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
                // [ / ] = previous / next frame
                if i.consume_key(egui::Modifiers::NONE, egui::Key::OpenBracket) {
                    let tab = self.active_tab;
                    let af = self.docs[tab].active_frame;
                    if af > 0 {
                        self.docs[tab].set_active_frame(af - 1);
                    }
                }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::CloseBracket) {
                    let tab = self.active_tab;
                    let af = self.docs[tab].active_frame;
                    let frames = self.docs[tab].frames;
                    if af + 1 < frames {
                        self.docs[tab].set_active_frame(af + 1);
                    }
                }
                // Space = play / pause animation
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Space) {
                    let tab = self.active_tab;
                    let doc = &mut self.docs[tab];
                    doc.playing = !doc.playing;
                    doc.canvas_dirty = true;
                }
            }
            // Delete
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Delete) {
                self.docs[self.active_tab].delete_selection();
            }
            // Enter = подтвердить вставку, иначе обрезать холст по выделению
            if self.docs[self.active_tab].sel.is_some() && !self.dialog_open() && self.renaming_layer.is_none() {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                    let doc = &mut self.docs[self.active_tab];
                    if doc.pasting {
                        // вставка не ложилась на слой — обрезать тут нечего
                        doc.commit_pending_paste();
                    } else {
                        doc.crop_to_selection();
                    }
                }
            }
            // Escape
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.docs[self.active_tab].cancel_pending_paste() {
                    // вставку отменили: слой не тронут, просто убираем блок
                } else if self.docs[self.active_tab].transforming {
                    self.docs[self.active_tab].transforming = false;
                    self.docs[self.active_tab].transform_corner = None;
                    self.docs[self.active_tab].transform_orig_rect = None;
                    self.docs[self.active_tab].canvas_dirty = true;
                    if let Some(saved) = self.tool_saved_shift.take() {
                        self.tool = saved;
                    } else {
                        self.tool = Tool::Select;
                    }
                } else if self.renaming_layer.is_some() {
                    self.renaming_layer = None;
                } else if self.show_text {
                    self.show_text = false;
                    self.text_cursor = None;
                    self.text_buffer.clear();
                } else if self.dialog_open() {
                    self.show_resize = false;
                    self.show_export = false;
                    self.show_panels = false;
                    self.show_settings = false;
                    self.show_scale = false;
                    self.show_quit_dialog = false;
                    self.show_welcome = false;
                    self.show_effects = false;
                    if self.tool == Tool::Effects { self.tool = Tool::Brush; }
                    crate::app::config::save_welcome_show_again(self.welcome_show_again);
                } else {
                    self.docs[self.active_tab].deselect();
                }
            }
            // Ctrl+D = deselect
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::D) {
                self.docs[self.active_tab].deselect();
            }
            // Ctrl+Shift+F = flatten layers
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::F) {
                if !self.dialog_open() {
                    self.docs[self.active_tab].flatten_layers();
                }
            }
            // Y = copy selection, Ctrl+C = то же.
            // Ctrl-перехватчики не забираем, пока открыт диалог или правится
            // имя слоя: там Ctrl+C/Ctrl+V принадлежат текстовому полю.
            let clip_ok = !self.dialog_open() && self.renaming_layer.is_none();
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Y)
                || (clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::C))
            {
                let tab = self.active_tab;
                if !self.docs[tab].copy_selection() {
                    self.status_hint = "nothing selected".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            // P = paste selection, Ctrl+V = то же
            if i.consume_key(egui::Modifiers::NONE, egui::Key::P)
                || (clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::V))
            {
                let tab = self.active_tab;
                if !self.docs[tab].paste_clipboard() {
                    self.status_hint = "clipboard is empty".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            // S = toggle transform (when selection exists)
            if i.consume_key(egui::Modifiers::NONE, egui::Key::S) {
                let tab = self.active_tab;
                let has_sel = self.docs[tab].sel.is_some();
                if has_sel && self.tool != Tool::Transform {
                    // auto-copy selection to buffer if empty
                    if self.docs[tab].sel_buffer.is_none() {
                        if let Some((x0, y0, x1, y1)) = self.docs[tab].sel {
                            let sw = (x1 - x0 + 1) as usize;
                            let sh = (y1 - y0 + 1) as usize;
                            let w = self.docs[tab].width;
                            let mut buf = Vec::with_capacity(sw * sh);
                            for yy in y0..=y1 {
                                for xx in x0..=x1 {
                                    let idx = (yy * w as i32 + xx) as usize;
                                    buf.push(self.docs[tab].layers[self.docs[tab].active_layer].cels[self.docs[tab].active_frame][idx]);
                                }
                            }
                            self.docs[tab].sel_buffer = Some(buf);
                            self.docs[tab].sel_buf_w = sw;
                            self.docs[tab].sel_buf_h = sh;
                        }
                    }
                    self.tool_saved_shift = Some(self.tool);
                    self.tool = Tool::Transform;
                    self.docs[tab].transforming = true;
                    self.docs[tab].transform_orig_rect = self.docs[tab].sel;
                    self.docs[tab].transform_corner = None;
                } else if self.tool == Tool::Transform && self.tool_saved_shift.is_some() {
                    self.tool = self.tool_saved_shift.take().unwrap();
                    self.docs[tab].transforming = false;
                    self.docs[tab].transform_corner = None;
                    self.docs[tab].transform_orig_rect = None;
                    self.docs[tab].canvas_dirty = true;
                }
            }
        });

        // scroll zoom / brush size.
        // Пока открыт диалог, колесо должно прокручивать панель внутри него,
        // а не зумить холст под ней.
        if !self.dialog_open() {
            let scroll = ctx.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 {
                let scroll_norm = scroll.signum();
                if ctx.input(|i| i.modifiers.shift) {
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
        }

        // mobile pinch zoom + two-finger pan
        if self.mobile {
            if let Some(mt) = ctx.input(|i| i.multi_touch()) {
                let tab = self.active_tab;
                let doc = &mut self.docs[tab];
                let old = doc.zoom;
                doc.zoom = (doc.zoom * mt.zoom_delta).clamp(0.1, 60.0);
                doc.pan *= doc.zoom / old;
                doc.pan.x += mt.translation_delta.x * (1.0 / doc.zoom);
                doc.pan.y += mt.translation_delta.y * (1.0 / doc.zoom);
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
        for layer in self.docs[tab].layers.iter().rev() {
            if !layer.visible { continue; }
            let p = layer.cels[self.docs[tab].active_frame][idx];
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
