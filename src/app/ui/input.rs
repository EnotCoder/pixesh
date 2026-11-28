use eframe::egui;

use crate::color::*;
use crate::constants::{self, Tool};
use crate::app::PixeshApp;
use crate::app::lua_config::{Action, Guard};
use crate::app::state::color::format_rgb_input;

impl PixeshApp {
    pub(crate) fn handle_input(&mut self, ctx: &egui::Context) {
        let text_focused = ctx.memory(|m| m.focused().is_some());
        // clear the status-bar hint once it has expired
        let now = ctx.input(|i| i.time);
        if now > self.status_hint_until {
            self.status_hint.clear();
        }

        // All bindings come from the Lua config. `consume_key` ignores extra
        // Shift/Alt, so bindings are sorted by modifier count (most specific
        // first): Ctrl+Shift+L wins over Ctrl+L and consumes the key. The
        // bundled "Ctrl+=" / "Ctrl+Shift+=" pair covers layouts where '+' is
        // produced with Shift.
        let bindings = crate::app::lua_config::bindings();
        ctx.input_mut(|i| {
            for (action, combo) in &bindings {
                if !self.guard_allows(action.guard(), i.modifiers, text_focused) {
                    continue;
                }
                if i.consume_key(combo.modifiers, combo.key) {
                    self.apply_action(*action, now);
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
                    self.brush = (self.brush + scroll_norm * constants::brush_step()).clamp(1.0, max);
                } else {
                    let tab = self.active_tab;
                    let doc = &mut self.docs[tab];
                    let old = doc.zoom;
                    doc.zoom = (doc.zoom
                        * (1.0 + scroll_norm * self.zoom_speed * constants::zoom_step()))
                        .clamp(constants::zoom_min(), constants::zoom_max());
                    doc.pan *= doc.zoom / old;
                }
            }
        }

        // arrow pan
        if !self.dialog_open() {
            let tab = self.active_tab;
            ctx.input(|i| {
                let fast = if i.modifiers.shift { constants::pan_shift_multiplier() } else { 1.0 };
                let speed = self.arrow_speed * fast;
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

    /// Scope of a binding, derived from the action (not configurable).
    fn guard_allows(&self, guard: Guard, modifiers: egui::Modifiers, text_focused: bool) -> bool {
        let clean = !self.dialog_open() && self.renaming_layer.is_none();
        match guard {
            Guard::Global => true,
            Guard::Clean => clean,
            Guard::DialogFree => !self.dialog_open(),
            Guard::Bare => !modifiers.ctrl && !modifiers.alt && !modifiers.mac_cmd && !text_focused,
            Guard::BareClean => clean
                && !modifiers.ctrl && !modifiers.alt && !modifiers.mac_cmd && !text_focused,
        }
    }

    /// Perform the action bound to the pressed combination.
    pub(crate) fn apply_action(&mut self, action: Action, now: f64) {
        match action {
            Action::Undo => {
                self.docs[self.active_tab].undo();
            }
            Action::Redo => {
                self.docs[self.active_tab].redo();
            }
            Action::BrushUp | Action::BrushDown => {
                let tab = self.active_tab;
                let max = self.docs[tab].width.max(self.docs[tab].height) as f32;
                let delta = constants::brush_step()
                    * if action == Action::BrushUp { 1.0 } else { -1.0 };
                self.brush = (self.brush + delta).clamp(1.0, max);
            }
            Action::AddLayer => {
                let tab = self.active_tab;
                self.docs[tab].add_layer();
            }
            Action::RemoveLayer => {
                let tab = self.active_tab;
                if self.docs[tab].layers.len() > 1 {
                    let al = self.docs[tab].active_layer;
                    self.docs[tab].remove_layer(al);
                } else {
                    self.hint("cannot remove the last layer");
                }
            }
            Action::Export => {
                let tab = self.active_tab;
                if self.docs[tab].export_name.is_empty() {
                    self.docs[tab].export_name = "pixesh.png".into();
                }
                self.show_export = true;
            }
            Action::Resize => {
                let tab = self.active_tab;
                self.resize_w = self.docs[tab].width as f32;
                self.resize_h = self.docs[tab].height as f32;
                self.show_resize = true;
            }
            Action::Panels => self.show_panels = !self.show_panels,
            Action::Settings => self.show_settings = !self.show_settings,
            Action::Scale => {
                let tab = self.active_tab;
                self.scale_w = self.docs[tab].width as f32;
                self.scale_h = self.docs[tab].height as f32;
                self.show_scale = true;
            }
            Action::Effects => {
                if self.show_effects {
                    self.show_effects = false;
                    if self.tool == Tool::Effects { self.tool = Tool::Brush; }
                } else if !self.dialog_open() {
                    self.tool = Tool::Effects;
                    self.show_effects = true;
                }
            }
            Action::NewTab => self.new_blank_tab(),
            Action::ColorInput => {
                self.color_input_buf = format_rgb_input(self.color);
                self.color_input_error = None;
                self.show_color_input = true;
            }
            Action::OpenFile => {
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
            Action::NextTab => {
                let n = self.docs.len();
                if n > 1 {
                    self.active_tab = (self.active_tab + 1) % n;
                }
            }
            Action::ToolBrush => self.tool = Tool::Brush,
            Action::ToolEraser => self.tool = Tool::Eraser,
            Action::ToolFill => self.tool = Tool::Fill,
            Action::ToolSelect => self.tool = Tool::Select,
            Action::ToolMove => self.tool = Tool::Move,
            Action::ToolText => self.tool = Tool::Text,
            Action::ToggleGrid => {
                let tab = self.active_tab;
                self.docs[tab].grid = !self.docs[tab].grid;
            }
            Action::HsvValueUp => self.bump_hsv(0.0, 0.0, constants::hsv_step()),
            Action::HsvValueDown => self.bump_hsv(0.0, 0.0, -constants::hsv_step()),
            Action::HsvSatDown => self.bump_hsv(0.0, -constants::hsv_step(), 0.0),
            Action::HsvSatUp => self.bump_hsv(0.0, constants::hsv_step(), 0.0),
            Action::HsvHueDown => self.bump_hsv(-constants::hsv_step(), 0.0, 0.0),
            Action::HsvHueUp => self.bump_hsv(constants::hsv_step(), 0.0, 0.0),
            Action::AlphaDown => self.bump_alpha(-constants::alpha_step()),
            Action::AlphaUp => self.bump_alpha(constants::alpha_step()),
            Action::Palette1 => self.pick_preset(0),
            Action::Palette2 => self.pick_preset(1),
            Action::Palette3 => self.pick_preset(2),
            Action::Palette4 => self.pick_preset(3),
            Action::Palette5 => self.pick_preset(4),
            Action::Palette6 => self.pick_preset(5),
            Action::Palette7 => self.pick_preset(6),
            Action::Palette8 => self.pick_preset(7),
            Action::Palette9 => self.pick_preset(8),
            Action::FramePrev => {
                let tab = self.active_tab;
                let af = self.docs[tab].active_frame;
                if af > 0 {
                    self.docs[tab].set_active_frame(af - 1);
                }
            }
            Action::FrameNext => {
                let tab = self.active_tab;
                let af = self.docs[tab].active_frame;
                let frames = self.docs[tab].frames;
                if af + 1 < frames {
                    self.docs[tab].set_active_frame(af + 1);
                }
            }
            Action::TogglePlay => {
                let tab = self.active_tab;
                let doc = &mut self.docs[tab];
                doc.playing = !doc.playing;
                doc.canvas_dirty = true;
            }
            Action::Delete => {
                self.docs[self.active_tab].delete_selection();
            }
            Action::CommitOrCrop => {
                if self.docs[self.active_tab].sel.is_some() {
                    let doc = &mut self.docs[self.active_tab];
                    if doc.pasting {
                        doc.commit_pending_paste();
                    } else {
                        doc.crop_to_selection();
                    }
                }
            }
            Action::Escape => {
                if self.show_clear_confirm {
                    self.show_clear_confirm = false;
                } else if self.show_color_input {
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
                    self.show_clear_confirm = false;
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
            Action::Deselect => {
                self.docs[self.active_tab].deselect();
            }
            Action::CloseTab => {
                let tab = self.active_tab;
                self.close_tab(tab);
            }
            Action::SelectAll => {
                self.docs[self.active_tab].select_all();
            }
            Action::Flatten => {
                self.docs[self.active_tab].flatten_layers();
            }
            Action::Copy | Action::CopyClipboard => {
                let tab = self.active_tab;
                if let Some(clip) = self.docs[tab].copy_selection() {
                    self.clipboard = clip;
                } else {
                    self.status_hint = "nothing selected".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            Action::Paste | Action::PasteClipboard => {
                let tab = self.active_tab;
                if !self.docs[tab].paste_clipboard(&self.clipboard) {
                    self.status_hint = "clipboard is empty".into();
                    self.status_hint_until = now + 2.0;
                }
            }
            Action::Transform => {
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
            Action::ClearAll => {
                self.show_clear_confirm = true;
            }
            Action::ReloadConfig => {
                match crate::app::lua_config::reload() {
                    Ok(msg) => self.hint(msg),
                    Err(msg) => self.hint(format!("config error: {msg}")),
                }
            }
        }
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
    fn copied_pixels_survive_tab_switch_and_paste_without_committing_source() {
        let mut app = PixeshApp::new();
        app.show_welcome = false;
        app.new_blank_tab();
        app.active_tab = 0;

        // A floating paste in tab 0: the layer itself is still untouched.
        {
            let doc = &mut app.docs[0];
            doc.sel = Some((1, 1, 1, 1));
            doc.sel_buffer = Some(vec![eframe::egui::Color32::RED]);
            doc.sel_buf_w = 1;
            doc.sel_buf_h = 1;
            doc.pasting = true;
        }
        let source_before = app.docs[0].layers[0].cels[0].clone();

        press(&mut app, Key::C, Modifiers::CTRL);
        press(&mut app, Key::Tab, Modifiers::CTRL);
        assert_eq!(app.active_tab, 1, "Ctrl+Tab должен переключить холст");
        press(&mut app, Key::V, Modifiers::CTRL);

        assert!(app.docs[0].pasting, "копирование не коммитит вставку в источнике");
        assert_eq!(app.docs[0].layers[0].cels[0], source_before);
        assert!(app.docs[1].pasting, "вставка в другом холсте тоже плавающая");
        assert_eq!(
            app.docs[1].sel_buffer.as_ref().unwrap(),
            &vec![eframe::egui::Color32::RED],
            "скопированные пиксели доступны во второй вкладке"
        );

        assert_eq!(app.docs[1].sel, Some((7, 7, 7, 7)), "блок в центре 16×16");
        app.docs[1].commit_pending_paste();
        assert_eq!(
            app.docs[1].layers[0].cels[0][119], // row 7, col 7 of a 16x16 canvas
            eframe::egui::Color32::RED,
            "вставка легла в центр второго холста"
        );
        assert_eq!(app.docs[0].layers[0].cels[0], source_before);
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

    #[test]
    fn lua_config_can_replace_a_default_shortcut() {
        let cfg = crate::app::lua_config::parse_for_test(
            "keys = { tool_brush = 'V' }",
        ).unwrap();
        crate::app::lua_config::with_test_config(cfg, || {
            let mut app = PixeshApp::new();
            app.show_welcome = false;

            app.tool = Tool::Eraser;
            press(&mut app, Key::V, Modifiers::NONE);
            assert_eq!(app.tool, Tool::Brush, "V must select the brush");

            app.tool = Tool::Text;
            press(&mut app, Key::B, Modifiers::NONE);
            assert_eq!(app.tool, Tool::Text, "the old B binding is gone");
        });
    }

    #[test]
    fn lua_config_values_reach_the_runtime_constants() {
        let cfg = crate::app::lua_config::parse_for_test(
            "colors = { palette = { '#01020304' } } constants = { hsv_step = 9 }",
        ).unwrap();
        crate::app::lua_config::with_test_config(cfg, || {
            assert_eq!(crate::constants::num_palette()[0], [1, 2, 3, 4]);
            assert_eq!(crate::constants::hsv_step(), 9.0);
        });
    }
}
