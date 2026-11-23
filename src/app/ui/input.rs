use eframe::egui;

use crate::color::*;
use crate::constants::Tool;
use crate::app::PixeshApp;
use crate::app::state::color::format_rgb_input;

impl PixeshApp {
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context) {
        let text_focused = ctx.memory(|m| m.focused().is_some());
        // clear the status-bar hint once it has expired
        let now = ctx.input(|i| i.time);
        if now > self.status_hint_until {
            self.status_hint.clear();
        }
        ctx.input_mut(|i| {
            // Shared guard for shortcuts that must not steal text input.
            let clip_ok = !self.dialog_open() && self.renaming_layer.is_none();
            // Ctrl+Z = undo
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::Z) {
                self.docs[self.active_tab].undo();
            }
            // Ctrl+Shift+Z = redo
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::Z) {
                self.docs[self.active_tab].redo();
            }
            // Ctrl++ / Ctrl+- = brush size. "+" is Shift+= on most layouts,
            // so Equals is accepted too (modifier matching ignores Shift).
            if clip_ok {
                let tab = self.active_tab;
                let max = self.docs[tab].width.max(self.docs[tab].height) as f32;
                if i.consume_key(egui::Modifiers::CTRL, egui::Key::Plus)
                    || i.consume_key(egui::Modifiers::CTRL, egui::Key::Equals)
                {
                    self.brush = (self.brush + 1.0).clamp(1.0, max);
                }
                if i.consume_key(egui::Modifiers::CTRL, egui::Key::Minus) {
                    self.brush = (self.brush - 1.0).clamp(1.0, max);
                }
            }
            // Ctrl+Shift+L = remove the active layer. Checked before Ctrl+L:
            // consume_key ignores extra Shift when matching, so the generic
            // shortcut would otherwise fire on Ctrl+Shift+L as well.
            // Both layer shortcuts act directly on the document — they never
            // commit a floating paste or a transform in progress.
            if clip_ok && i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::L) {
                let tab = self.active_tab;
                if self.docs[tab].layers.len() > 1 {
                    let al = self.docs[tab].active_layer;
                    self.docs[tab].remove_layer(al);
                } else {
                    self.hint("cannot remove the last layer");
                }
            }
            // Ctrl+L = add a new layer above the active one
            if clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::L) {
                let tab = self.active_tab;
                self.docs[tab].add_layer();
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
            // Ctrl+N = new blank tab; same behaviour as the toolbar "+" button.
            if clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::N) {
                self.new_blank_tab();
            }
            // Ctrl+K = live RGB/hex entry. The dialog is modal, so it cannot be
            // opened on top of another dialog or while renaming a layer.
            if clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::K) {
                self.color_input_buf = format_rgb_input(self.color);
                self.color_input_error = None;
                self.show_color_input = true;
            }
            // Ctrl+O = load image (new tab)
            if i.consume_key(egui::Modifiers::CTRL, egui::Key::O) {
                #[cfg(feature = "rfd")]
                {
                    let home = crate::app::config::default_dir();
                    if let Some(path) = rfd::FileDialog::new()
                        .set_directory(&home)
                        .add_filter("Images", &["png", "jpg", "jpeg", "gif", "bmp", "webp", "tiff", "tga"])
                        .pick_file()
                    {
                        self.open_path(&path);
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
                // colour: W/S value, A/D saturation, I/K hue
                if i.consume_key(egui::Modifiers::NONE, egui::Key::W) { self.bump_hsv(0.0, 0.0, 5.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::S) { self.bump_hsv(0.0, 0.0, -5.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::A) { self.bump_hsv(0.0, -5.0, 0.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::D) { self.bump_hsv(0.0, 5.0, 0.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::I) { self.bump_hsv(-5.0, 0.0, 0.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::K) { self.bump_hsv(5.0, 0.0, 0.0); }
                // , / . = opacity
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Comma) { self.bump_alpha(-5.0); }
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Period) { self.bump_alpha(5.0); }
                // 1–9 = preset colours from NUM_PALETTE.
                // The index is called idx rather than i: the ctx.input_mut(|i| …)
                // closure already uses the name i, and shadowing would break
                // the surrounding handlers.
                for (idx, key) in [
                    egui::Key::Num1,
                    egui::Key::Num2,
                    egui::Key::Num3,
                    egui::Key::Num4,
                    egui::Key::Num5,
                    egui::Key::Num6,
                    egui::Key::Num7,
                    egui::Key::Num8,
                    egui::Key::Num9,
                ]
                .into_iter()
                .enumerate()
                {
                    if i.consume_key(egui::Modifiers::NONE, key) {
                        self.pick_preset(idx);
                    }
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
            // Enter = commit the paste, otherwise crop the canvas to the selection
            if self.docs[self.active_tab].sel.is_some() && !self.dialog_open() && self.renaming_layer.is_none() {
                if i.consume_key(egui::Modifiers::NONE, egui::Key::Enter) {
                    let doc = &mut self.docs[self.active_tab];
                    if doc.pasting {
                        // the paste never landed on the layer — nothing to crop here
                        doc.commit_pending_paste();
                    } else {
                        doc.crop_to_selection();
                    }
                }
            }
            // Escape
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                if self.show_color_input {
                    // The text field owns Escape while the colour dialog is up.
                    self.show_color_input = false;
                    self.color_input_buf.clear();
                    self.color_input_error = None;
                } else if self.docs[self.active_tab].cancel_pending_paste() {
                    // paste cancelled: the layer was not touched, just remove the block
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
                    self.show_color_input = false;
                    self.color_input_buf.clear();
                    self.color_input_error = None;
                    self.show_effects = false;
                    if self.tool == Tool::Effects { self.tool = Tool::Brush; }
                } else {
                    self.docs[self.active_tab].deselect();
                }
            }
            // Ctrl+Shift+D = deselect. Checked before Ctrl+D: consume_key
            // ignores extra Shift, so the close-tab shortcut would also match.
            if clip_ok && i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::D) {
                self.docs[self.active_tab].deselect();
            }
            // Ctrl+D = close the active tab (same as the tab bar close button).
            if clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::D) {
                let tab = self.active_tab;
                self.close_tab(tab);
            }
            // Ctrl+A = select the whole canvas, like in Photoshop.
            // The guard is shared with Ctrl+C/Ctrl+V for a reason: the egui
            // TextEdit field itself uses Ctrl+A to select all text, and
            // without this check we would take it away from the field.
            if clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::A) {
                self.docs[self.active_tab].select_all();
            }
            // Ctrl+Shift+F = flatten layers
            if i.consume_key(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::F) {
                if !self.dialog_open() {
                    self.docs[self.active_tab].flatten_layers();
                }
            }
            // Y = copy selection, Ctrl+C = same.
            if i.consume_key(egui::Modifiers::NONE, egui::Key::Y)
                || (clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::C))
            {
                let tab = self.active_tab;
                if !self.docs[tab].copy_selection() {
                    self.status_hint = "nothing selected".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            // P = paste selection, Ctrl+V = same
            if i.consume_key(egui::Modifiers::NONE, egui::Key::P)
                || (clip_ok && i.consume_key(egui::Modifiers::CTRL, egui::Key::V))
            {
                let tab = self.active_tab;
                if !self.docs[tab].paste_clipboard() {
                    self.status_hint = "clipboard is empty".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            // X = toggle transform (when selection exists).
            // text_focused is needed: otherwise "x" typed into a text field or
            // a DragValue would switch the tool out from under the user.
            if i.consume_key(egui::Modifiers::NONE, egui::Key::X) && !text_focused {
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
        // While a dialog is open, the wheel should scroll the panel inside it
        // rather than zoom the canvas underneath. The same applies to scrollable
        // panels (layers, timeline, ...): only zoom when the cursor is actually
        // over the canvas.
        let over_canvas = ctx
            .pointer_hover_pos()
            .is_some_and(|p| self.canvas_rect.contains(p));
        if !self.dialog_open() && over_canvas {
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
            self.push_history(c);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{Event, Key, Modifiers, RawInput};

    /// Feed one key press through a real egui Context, then run the app's
    /// input handler the same way `update()` does.
    fn press(app: &mut PixeshApp, key: Key, modifiers: Modifiers) {
        let ctx = egui::Context::default();
        let _ = ctx.run(RawInput::default(), |_| {});
        let _ = ctx.run(
            RawInput {
                events: vec![Event::Key {
                    key,
                    physical_key: None,
                    pressed: true,
                    repeat: false,
                    modifiers,
                }],
                ..Default::default()
            },
            |_| {},
        );
        app.handle_input(&ctx);
    }

    #[test]
    fn ctrl_plus_and_minus_change_brush_size() {
        let mut app = PixeshApp::new();
        app.show_welcome = false; // first-run dialog would block shortcuts
        app.brush = 5.0;
        press(&mut app, Key::Plus, Modifiers::CTRL);
        assert_eq!(app.brush, 6.0);
        press(&mut app, Key::Equals, Modifiers::CTRL);
        assert_eq!(app.brush, 7.0);
        press(&mut app, Key::Minus, Modifiers::CTRL);
        assert_eq!(app.brush, 6.0);
    }

    #[test]
    fn ctrl_shift_l_removes_a_layer_and_ctrl_l_adds_one() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        let n = app.docs[0].layers.len();
        app.docs[0].add_layer();

        // consume_key ignores the extra Shift, so the more specific
        // shortcut must win when both bindings match.
        press(&mut app, Key::L, Modifiers::CTRL | Modifiers::SHIFT);
        assert_eq!(app.docs[0].layers.len(), n, "Ctrl+Shift+L удалил слой");
        press(&mut app, Key::L, Modifiers::CTRL);
        assert_eq!(app.docs[0].layers.len(), n + 1, "Ctrl+L добавил слой");
    }

    #[test]
    fn layer_shortcuts_do_not_commit_a_pending_paste() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        let doc = &mut app.docs[0];
        doc.sel = Some((0, 0, 0, 0));
        doc.sel_buffer = Some(vec![eframe::egui::Color32::RED]);
        doc.sel_buf_w = 1;
        doc.pasting = true;

        press(&mut app, Key::L, Modifiers::CTRL);
        assert!(app.docs[0].pasting, "Ctrl+L не должен коммитить вставку");
        press(&mut app, Key::L, Modifiers::CTRL | Modifiers::SHIFT);
        assert!(app.docs[0].pasting, "Ctrl+Shift+L не должен коммитить вставку");
    }

    #[test]
    fn brush_size_is_clamped_to_the_canvas_side() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        app.brush = 1.0;
        press(&mut app, Key::Minus, Modifiers::CTRL);
        assert_eq!(app.brush, 1.0);
    }

    #[test]
    fn ctrl_n_creates_a_tab_and_ctrl_d_closes_it() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        assert_eq!(app.docs.len(), 1);

        press(&mut app, Key::N, Modifiers::CTRL);
        assert_eq!(app.docs.len(), 2);
        assert_eq!(app.active_tab, 1);
        assert_eq!(app.docs[1].name, "Untitled 1");

        press(&mut app, Key::D, Modifiers::CTRL);
        assert_eq!(app.docs.len(), 1);
        assert_eq!(app.active_tab, 0);
    }

    #[test]
    fn ctrl_shift_d_deselects_instead_of_closing_a_tab() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        app.new_blank_tab();
        app.docs[app.active_tab].sel = Some((0, 0, 1, 1));

        press(&mut app, Key::D, Modifiers::CTRL | Modifiers::SHIFT);
        assert!(app.docs[1].sel.is_none(), "Ctrl+Shift+D должен снимать выделение");
        assert_eq!(app.docs.len(), 2, "Ctrl+Shift+D не должен закрывать вкладку");
    }

    #[test]
    fn ctrl_k_opens_live_color_input_without_history_change() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        app.set_color(eframe::egui::Color32::from_rgb(12, 34, 56));
        app.color_history.push(eframe::egui::Color32::RED);

        press(&mut app, Key::K, Modifiers::CTRL);
        assert!(app.show_color_input);
        assert_eq!(app.color_input_buf, "12, 34, 56");
        assert!(app.color_input_error.is_none());
        assert_eq!(app.color_history, vec![eframe::egui::Color32::RED]);
    }
}
