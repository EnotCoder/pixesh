//! User-editable Lua configuration for colours, constants and key bindings.
//!
//! The configuration is a plain Lua 5.4 file with three global tables:
//! `colors`, `constants` and `keys`. Any table (or field) may be omitted —
//! missing values keep their bundled defaults. See `config/config.lua` for the
//! documented template; that file is copied to
//! `~/.config/pixesh/config.lua` on the first launch.
//!
//! Parsing happens once at startup and on `Ctrl+Shift+R`. All runtime readers
//! go through [`get`] so a reload is visible to every widget immediately.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{OnceLock, RwLock};

use eframe::egui::{Color32, Key, Modifiers};
use mlua::{Lua, Table, Value};

/// Template shipped with the application and used as the default configuration.
pub(crate) const TEMPLATE: &str = include_str!("../../config/config.lua");

/// Parsing can fail either because Lua rejected the file or because a value is
/// invalid; both are reported as text to the user.
type ConfigResult<T> = Result<T, Box<dyn std::error::Error>>;

fn err(msg: impl Into<String>) -> Box<dyn std::error::Error> {
    let text: String = msg.into();
    text.into()
}

// ── Values ────────────────────────────────────────────

#[derive(Clone, Copy, Debug)]
pub(crate) struct Colors {
    pub bg: Color32,
    pub panel: Color32,
    pub panel_light: Color32,
    pub border: Color32,
    pub text: Color32,
    pub accent: Color32,
    pub hover: Color32,
    pub dim: Color32,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Numbers {
    pub font_size: f32,
    pub char_width: f32,
    pub row_height: f32,
    pub panel_padding: f32,
    pub checker_px: f32,
    pub max_side: i32,
    pub hsv_step: f32,
    pub alpha_step: f32,
    pub brush_step: f32,
    pub zoom_min: f32,
    pub zoom_max: f32,
    pub zoom_step: f32,
    pub pan_shift_multiplier: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct KeyCombo {
    pub modifiers: Modifiers,
    pub key: Key,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum Action {
    Undo, Redo,
    BrushUp, BrushDown,
    AddLayer, RemoveLayer,
    Export, Resize, Panels, Settings, Scale, Effects,
    NewTab, ColorInput, OpenFile, NextTab,
    ToolBrush, ToolEraser, ToolFill, ToolSelect, ToolMove, ToolText,
    ToggleGrid,
    HsvValueUp, HsvValueDown,
    HsvSatDown, HsvSatUp,
    HsvHueDown, HsvHueUp,
    AlphaDown, AlphaUp,
    Palette1, Palette2, Palette3, Palette4, Palette5,
    Palette6, Palette7, Palette8, Palette9,
    FramePrev, FrameNext, TogglePlay,
    Delete, CommitOrCrop, Escape,
    Deselect, CloseTab, SelectAll, Flatten,
    Copy, CopyClipboard, Paste, PasteClipboard,
    Transform, ClearAll, ReloadConfig,
}

impl Action {
    /// Lua name used in the `keys` table.
    fn name(self) -> &'static str {
        match self {
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::BrushUp => "brush_up",
            Self::BrushDown => "brush_down",
            Self::AddLayer => "add_layer",
            Self::RemoveLayer => "remove_layer",
            Self::Export => "export",
            Self::Resize => "resize",
            Self::Panels => "panels",
            Self::Settings => "settings",
            Self::Scale => "scale",
            Self::Effects => "effects",
            Self::NewTab => "new_tab",
            Self::ColorInput => "color_input",
            Self::OpenFile => "open_file",
            Self::NextTab => "next_tab",
            Self::ToolBrush => "tool_brush",
            Self::ToolEraser => "tool_eraser",
            Self::ToolFill => "tool_fill",
            Self::ToolSelect => "tool_select",
            Self::ToolMove => "tool_move",
            Self::ToolText => "tool_text",
            Self::ToggleGrid => "toggle_grid",
            Self::HsvValueUp => "hsv_value_up",
            Self::HsvValueDown => "hsv_value_down",
            Self::HsvSatDown => "hsv_sat_down",
            Self::HsvSatUp => "hsv_sat_up",
            Self::HsvHueDown => "hsv_hue_down",
            Self::HsvHueUp => "hsv_hue_up",
            Self::AlphaDown => "alpha_down",
            Self::AlphaUp => "alpha_up",
            Self::Palette1 => "palette_1",
            Self::Palette2 => "palette_2",
            Self::Palette3 => "palette_3",
            Self::Palette4 => "palette_4",
            Self::Palette5 => "palette_5",
            Self::Palette6 => "palette_6",
            Self::Palette7 => "palette_7",
            Self::Palette8 => "palette_8",
            Self::Palette9 => "palette_9",
            Self::FramePrev => "frame_prev",
            Self::FrameNext => "frame_next",
            Self::TogglePlay => "toggle_play",
            Self::Delete => "delete",
            Self::CommitOrCrop => "commit_or_crop",
            Self::Escape => "escape",
            Self::Deselect => "deselect",
            Self::CloseTab => "close_tab",
            Self::SelectAll => "select_all",
            Self::Flatten => "flatten",
            Self::Copy => "copy",
            Self::CopyClipboard => "copy_clipboard",
            Self::Paste => "paste",
            Self::PasteClipboard => "paste_clipboard",
            Self::Transform => "transform",
            Self::ClearAll => "clear_all",
            Self::ReloadConfig => "reload_config",
        }
    }

    fn from_name(name: &str) -> Option<Self> {
        ALL.iter().copied().find(|a| a.name() == name)
    }

    /// When a shortcut may fire. This is part of the application logic, not of
    /// the config: the Lua file only chooses the key combination.
    pub(crate) fn guard(self) -> Guard {
        match self {
            Self::Undo | Self::Redo | Self::Effects | Self::NextTab
            | Self::Delete | Self::Escape
            | Self::Copy | Self::Paste => Guard::Global,
            Self::BrushUp | Self::BrushDown | Self::AddLayer | Self::RemoveLayer
            | Self::NewTab | Self::ColorInput | Self::Deselect | Self::CloseTab
            | Self::SelectAll | Self::CommitOrCrop | Self::CopyClipboard
            | Self::PasteClipboard | Self::ReloadConfig => Guard::Clean,
            Self::Export | Self::Resize | Self::Panels | Self::Settings
            | Self::Scale | Self::OpenFile | Self::Flatten => Guard::DialogFree,
            Self::ToolBrush | Self::ToolEraser | Self::ToolFill | Self::ToolSelect
            | Self::ToolMove | Self::ToolText | Self::ToggleGrid
            | Self::HsvValueUp | Self::HsvValueDown | Self::HsvSatDown | Self::HsvSatUp
            | Self::HsvHueDown | Self::HsvHueUp | Self::AlphaDown | Self::AlphaUp
            | Self::Palette1 | Self::Palette2 | Self::Palette3 | Self::Palette4
            | Self::Palette5 | Self::Palette6 | Self::Palette7 | Self::Palette8
            | Self::Palette9 | Self::FramePrev | Self::FrameNext | Self::TogglePlay => Guard::Bare,
            Self::Transform | Self::ClearAll => Guard::BareClean,
        }
    }
}

const ALL: &[Action] = &[
    Action::Undo, Action::Redo, Action::BrushUp, Action::BrushDown,
    Action::AddLayer, Action::RemoveLayer, Action::Export, Action::Resize,
    Action::Panels, Action::Settings, Action::Scale, Action::Effects,
    Action::NewTab, Action::ColorInput, Action::OpenFile, Action::NextTab,
    Action::ToolBrush, Action::ToolEraser, Action::ToolFill, Action::ToolSelect,
    Action::ToolMove, Action::ToolText, Action::ToggleGrid,
    Action::HsvValueUp, Action::HsvValueDown, Action::HsvSatDown, Action::HsvSatUp,
    Action::HsvHueDown, Action::HsvHueUp, Action::AlphaDown, Action::AlphaUp,
    Action::Palette1, Action::Palette2, Action::Palette3, Action::Palette4,
    Action::Palette5, Action::Palette6, Action::Palette7, Action::Palette8,
    Action::Palette9, Action::FramePrev, Action::FrameNext, Action::TogglePlay,
    Action::Delete, Action::CommitOrCrop, Action::Escape, Action::Deselect,
    Action::CloseTab, Action::SelectAll, Action::Flatten, Action::Copy,
    Action::CopyClipboard, Action::Paste, Action::PasteClipboard,
    Action::Transform, Action::ClearAll, Action::ReloadConfig,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Guard {
    /// Works everywhere, including dialogs (kept for the historical shortcuts).
    Global,
    /// No dialog, no layer rename.
    Clean,
    /// No dialog (dialog-opening shortcuts).
    DialogFree,
    /// Bare key: no Ctrl/Alt and no text field focused.
    Bare,
    /// Bare key plus `Clean` (destructive/transform shortcuts).
    BareClean,
}

#[derive(Clone, Debug)]
pub(crate) struct Config {
    pub colors: Colors,
    pub palette: [[u8; 4]; 9],
    pub numbers: Numbers,
    keys: HashMap<Action, Vec<KeyCombo>>,
}

impl Default for Config {
    fn default() -> Self {
        Self::parse(TEMPLATE, &baseline()).expect("bundled config/config.lua must be valid")
    }
}

/// Values used when a key is absent from both the user file and the template.
fn baseline() -> Config {
    Config {
        colors: Colors {
            bg: Color32::from_rgb(24, 24, 24),
            panel: Color32::from_rgb(32, 32, 32),
            panel_light: Color32::from_rgb(44, 44, 44),
            border: Color32::from_rgb(80, 80, 80),
            text: Color32::from_rgb(220, 220, 220),
            accent: Color32::from_rgb(200, 120, 60),
            hover: Color32::from_rgb(60, 60, 60),
            dim: Color32::from_rgb(120, 120, 120),
        },
        palette: [
            [255, 0, 0, 255], [0, 255, 0, 255], [0, 0, 255, 255],
            [255, 255, 0, 255], [0, 255, 255, 255], [255, 0, 255, 255],
            [255, 255, 255, 255], [128, 128, 128, 255], [0, 0, 0, 255],
        ],
        numbers: Numbers {
            font_size: 20.0,
            char_width: 11.0,
            row_height: 22.0,
            panel_padding: 12.0,
            checker_px: 8.0,
            max_side: 4096,
            hsv_step: 5.0,
            alpha_step: 5.0,
            brush_step: 1.0,
            zoom_min: 0.1,
            zoom_max: 60.0,
            zoom_step: 0.1,
            pan_shift_multiplier: 4.0,
        },
        keys: HashMap::new(),
    }
}

// ── Parsing ───────────────────────────────────────────

impl Config {
    /// Parse a Lua source string on top of `defaults`. Unknown fields are
    /// ignored, malformed values are reported with the field name.
    pub(crate) fn parse(src: &str, defaults: &Config) -> ConfigResult<Config> {
        let lua = Lua::new();
        lua.load(src).exec().map_err(|e| err(format!("Lua syntax error: {e}")))?;
        let mut cfg = defaults.clone();

        if let Some(t) = optional_table(&lua, "colors")? {
            let c = &mut cfg.colors;
            c.bg = color_field(&t, "bg", c.bg)?;
            c.panel = color_field(&t, "panel", c.panel)?;
            c.panel_light = color_field(&t, "panel_light", c.panel_light)?;
            c.border = color_field(&t, "border", c.border)?;
            c.text = color_field(&t, "text", c.text)?;
            c.accent = color_field(&t, "accent", c.accent)?;
            c.hover = color_field(&t, "hover", c.hover)?;
            c.dim = color_field(&t, "dim", c.dim)?;

            if let Value::Table(p) = t.get::<Value>("palette")? {
                for i in 1..=cfg.palette.len() {
                    if let Value::Nil = p.raw_get::<Value>(i)? { break; }
                    cfg.palette[i - 1] = parse_rgba(&p.raw_get::<Value>(i)?)?;
                }
            }
        }

        if let Some(t) = optional_table(&lua, "constants")? {
            let n = &mut cfg.numbers;
            n.font_size = number_field(&t, "font_size", n.font_size, 1.0, 200.0)?;
            n.char_width = number_field(&t, "char_width", n.char_width, 0.1, 100.0)?;
            n.row_height = number_field(&t, "row_height", n.row_height, 1.0, 200.0)?;
            n.panel_padding = number_field(&t, "panel_padding", n.panel_padding, 0.0, 200.0)?;
            n.checker_px = number_field(&t, "checker_px", n.checker_px, 1.0, 128.0)?;
            n.max_side = int_field(&t, "max_side", n.max_side, 16, 16384)?;
            n.hsv_step = number_field(&t, "hsv_step", n.hsv_step, 0.1, 180.0)?;
            n.alpha_step = number_field(&t, "alpha_step", n.alpha_step, 0.1, 255.0)?;
            n.brush_step = number_field(&t, "brush_step", n.brush_step, 0.1, 512.0)?;
            n.zoom_min = number_field(&t, "zoom_min", n.zoom_min, 0.01, 10.0)?;
            n.zoom_max = number_field(&t, "zoom_max", n.zoom_max, 1.0, 500.0)?;
            n.zoom_step = number_field(&t, "zoom_step", n.zoom_step, 0.001, 10.0)?;
            n.pan_shift_multiplier = number_field(&t, "pan_shift_multiplier", n.pan_shift_multiplier, 1.0, 20.0)?;
        }
        if cfg.numbers.zoom_min >= cfg.numbers.zoom_max {
            return Err("constants.zoom_min must be smaller than zoom_max".into());
        }

        if let Some(t) = optional_table(&lua, "keys")? {
            for pair in t.pairs::<String, Value>() {
                let (name, value) = pair.map_err(|e| err(format!("keys: {e}")))?;
                let action = Action::from_name(&name)
                    .ok_or_else(|| err(format!("keys: unknown action '{name}'")))?;
                // `undo = false` disables an inherited default binding.
                if matches!(value, Value::Boolean(false)) {
                    cfg.keys.remove(&action);
                    continue;
                }
                let combos = match value {
                    Value::String(s) => vec![parse_combo(&s.to_string_lossy())?],
                    Value::Table(list) => {
                        let mut out = Vec::new();
                        for i in 1..=list.raw_len() {
                            let v = list.raw_get::<Value>(i)?;
                            match v {
                                Value::String(s) => out.push(parse_combo(&s.to_string_lossy())?),
                                Value::Nil => break,
                                _ => return Err(err(format!("keys.{name}: expected a string of keys"))),
                            }
                        }
                        if out.is_empty() {
                            return Err(err(format!("keys.{name}: empty key list")));
                        }
                        out
                    }
                    _ => return Err(err(format!("keys.{name}: expected a key string or a list"))),
                };
                cfg.keys.insert(action, combos);
            }
        }

        Ok(cfg)
    }

    /// Flat list used by the input handler. More specific combos come first so
    /// a conflict between two bindings is resolved consistently.
    pub(crate) fn bindings(&self) -> Vec<(Action, KeyCombo)> {
        let mut out: Vec<(Action, KeyCombo)> = self
            .keys
            .iter()
            .flat_map(|(&action, combos)| combos.iter().map(move |&c| (action, c)))
            .collect();
        out.sort_by(|a, b| {
            let weight = |c: KeyCombo| {
                [c.modifiers.ctrl, c.modifiers.shift, c.modifiers.alt, c.modifiers.mac_cmd]
                    .iter().filter(|v| **v).count()
            };
            weight(b.1).cmp(&weight(a.1))
                .then_with(|| (a.0.name()).cmp(b.0.name()))
        });
        out
    }
}

fn optional_table(lua: &Lua, name: &str) -> ConfigResult<Option<Table>> {
    match lua.globals().get::<Value>(name)? {
        Value::Nil => Ok(None),
        Value::Table(t) => Ok(Some(t)),
        _ => Err(err(format!("'{name}' must be a table"))),
    }
}

fn color_field(t: &Table, key: &str, fallback: Color32) -> ConfigResult<Color32> {
    match t.get::<Value>(key)? {
        Value::Nil => Ok(fallback),
        v => parse_color(&v).map_err(|e| err(format!("colors.{key}: {e}"))),
    }
}

fn number_field(t: &Table, key: &str, fallback: f32, min: f32, max: f32) -> ConfigResult<f32> {
    let v = match t.get::<Value>(key)? {
        Value::Nil => return Ok(fallback),
        v => v,
    };
    let number = match v {
        Value::Integer(i) => i as f64,
        Value::Number(f) => f,
        _ => return Err(err(format!("constants.{key}: expected a number"))),
    };
    if !number.is_finite() || !(min as f64..=max as f64).contains(&number) {
        return Err(err(format!("constants.{key}: expected a number in {min}..={max}")));
    }
    Ok(number as f32)
}

fn int_field(t: &Table, key: &str, fallback: i32, min: i32, max: i32) -> ConfigResult<i32> {
    let v = match t.get::<Value>(key)? {
        Value::Nil => return Ok(fallback),
        v => v,
    };
    let number = match v {
        Value::Integer(i) => i,
        Value::Number(f) if f.is_finite() && f.fract() == 0.0 => f as i64,
        _ => return Err(err(format!("constants.{key}: expected an integer"))),
    };
    if !(min as i64..=max as i64).contains(&number) {
        return Err(err(format!("constants.{key}: expected an integer in {min}..={max}")));
    }
    Ok(number as i32)
}

fn parse_color(v: &Value) -> ConfigResult<Color32> {
    let [r, g, b, a] = parse_rgba(v)?;
    Ok(Color32::from_rgba_unmultiplied(r, g, b, a))
}

/// Raw RGBA components, kept unpremultiplied for the number-key palette.
fn parse_rgba(v: &Value) -> ConfigResult<[u8; 4]> {
    match v {
        Value::String(s) => parse_hex_rgba(&s.to_string_lossy()),
        Value::Table(t) => Ok([
            t.raw_get::<u8>(1)?, t.raw_get::<u8>(2)?, t.raw_get::<u8>(3)?,
            t.raw_get::<Option<u8>>(4)?.unwrap_or(255),
        ]),
        _ => Err("expected '#RRGGBB[AA]' or {r, g, b[, a]}".into()),
    }
}

fn parse_hex_rgba(text: &str) -> ConfigResult<[u8; 4]> {
    let hex = text.trim().trim_start_matches('#');
    let digits = |i: usize| -> ConfigResult<u8> {
        hex.as_bytes().get(i).and_then(|b| (*b as char).to_digit(16))
            .map(|d| d as u8)
            .ok_or_else(|| err(format!("invalid colour '{text}'")))
    };
    let (r, g, b, a) = match hex.len() {
        3 => (digits(0)? * 17, digits(1)? * 17, digits(2)? * 17, 255),
        4 => (digits(0)? * 17, digits(1)? * 17, digits(2)? * 17, digits(3)? * 17),
        6 => (digits(0)? * 16 + digits(1)?, digits(2)? * 16 + digits(3)?,
              digits(4)? * 16 + digits(5)?, 255),
        8 => (digits(0)? * 16 + digits(1)?, digits(2)? * 16 + digits(3)?,
              digits(4)? * 16 + digits(5)?, digits(6)? * 16 + digits(7)?),
        _ => return Err(err(format!("invalid colour '{text}': use #RGB, #RGBA, #RRGGBB or #RRGGBBAA"))),
    };
    Ok([r, g, b, a])
}

/// Parse a shortcut such as `Ctrl+Shift+Z`, `1`, `,` or `Ctrl++`.
pub(crate) fn parse_combo(text: &str) -> ConfigResult<KeyCombo> {
    let text = text.trim();
    if text.is_empty() { return Err("empty key binding".into()); }
    let mut parts: Vec<&str> = text.split('+').collect();
    let key_name = if text.ends_with('+') {
        parts.pop(); // trailing empty segment after the '+'
        "+"
    } else {
        parts.pop().unwrap_or("")
    };
    let key = Key::from_name(key_name)
        .ok_or_else(|| err(format!("unknown key '{key_name}' in '{text}'")))?;
    let mut modifiers = Modifiers::NONE;
    for part in parts.iter().filter(|p| !p.is_empty()) {
        let m = match part.trim().to_ascii_lowercase().as_str() {
            "ctrl" | "control" => Modifiers::CTRL,
            "shift" => Modifiers::SHIFT,
            "alt" => Modifiers::ALT,
            "cmd" | "super" | "meta" | "command" => Modifiers::MAC_CMD,
            "none" => Modifiers::NONE,
            other => return Err(err(format!("unknown modifier '{other}' in '{text}'"))),
        };
        modifiers |= m;
    }
    Ok(KeyCombo { modifiers, key })
}

// ── Runtime state ─────────────────────────────────────

static CONFIG: OnceLock<RwLock<Config>> = OnceLock::new();
static PATH: OnceLock<PathBuf> = OnceLock::new();
static ERROR: RwLock<Option<String>> = RwLock::new(None);
static VERSION: AtomicU64 = AtomicU64::new(0);

fn current() -> &'static RwLock<Config> {
    CONFIG.get_or_init(|| RwLock::new(Config::default()))
}

#[cfg(not(test))]
pub(crate) fn get() -> std::sync::RwLockReadGuard<'static, Config> {
    current().read().expect("config lock poisoned")
}

/// In tests each thread can install a private configuration without touching
/// the process-wide one, so parallel tests stay independent.
#[cfg(test)]
pub(crate) fn get() -> Config {
    if let Some(cfg) = TEST_OVERRIDE.with(|o| o.borrow().clone()) {
        return cfg;
    }
    current().read().expect("config lock poisoned").clone()
}

#[cfg(test)]
thread_local! {
    static TEST_OVERRIDE: std::cell::RefCell<Option<Config>> = const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(crate) fn with_test_config<R>(cfg: Config, f: impl FnOnce() -> R) -> R {
    TEST_OVERRIDE.with(|o| *o.borrow_mut() = Some(cfg));
    let result = f();
    TEST_OVERRIDE.with(|o| *o.borrow_mut() = None);
    result
}

pub(crate) fn version() -> u64 { VERSION.load(Ordering::SeqCst) }

pub(crate) fn bindings() -> Vec<(Action, KeyCombo)> { get().bindings() }

pub(crate) fn load_error() -> Option<String> {
    ERROR.read().expect("config lock poisoned").clone()
}

/// Resolve the config path: `--config PATH` / `--config=PATH`, otherwise
/// `$XDG_CONFIG_HOME/pixesh/config.lua` or `~/.config/pixesh/config.lua`.
pub(crate) fn config_path() -> PathBuf {
    let args: Vec<std::ffi::OsString> = std::env::args_os().collect();
    for (i, arg) in args.iter().enumerate().skip(1) {
        let text = arg.to_string_lossy();
        if let Some(value) = text.strip_prefix("--config=") {
            return PathBuf::from(value);
        }
        if text == "--config" {
            if let Some(next) = args.get(i + 1) { return PathBuf::from(next); }
        }
    }
    #[cfg(windows)]
    if let Some(appdata) = std::env::var_os("APPDATA") {
        if !appdata.is_empty() {
            return PathBuf::from(appdata).join("pixesh").join("config.lua");
        }
    }
    if let Some(xdg) = std::env::var_os("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return PathBuf::from(xdg).join("pixesh").join("config.lua");
        }
    }
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("EXTERNAL_STORAGE"))
        .unwrap_or_default();
    PathBuf::from(home).join(".config").join("pixesh").join("config.lua")
}

/// Create the config file with the documented defaults if it does not exist.
/// Errors here are non-fatal: the app runs with the bundled configuration.
pub(crate) fn init(path: PathBuf) {
    let _ = PATH.set(path.clone());
    if !path.exists() {
        if let Some(dir) = path.parent() {
            if let Err(e) = std::fs::create_dir_all(dir) {
                set_error(format!("cannot create {}: {e}", dir.display()));
                return;
            }
        }
        if let Err(e) = std::fs::write(&path, TEMPLATE) {
            set_error(format!("cannot write {}: {e}", path.display()));
            return;
        }
    }
    let _ = reload();
}

/// Re-read the file chosen at startup. Returns a human-readable status.
pub(crate) fn reload() -> Result<String, String> {
    let path = PATH.get().cloned().unwrap_or_else(config_path);
    match std::fs::read_to_string(&path) {
        Err(e) => {
            let msg = format!("cannot read {}: {e}", path.display());
            set_error(msg.clone());
            Err(msg)
        }
        Ok(src) => {
            let defaults = Config::default();
            match Config::parse(&src, &defaults).map_err(|e| e.to_string()) {
                Ok(cfg) => {
                    *current().write().expect("config lock poisoned") = cfg;
                    *ERROR.write().expect("config lock poisoned") = None;
                    VERSION.fetch_add(1, Ordering::SeqCst);
                    Ok(format!("config reloaded: {}", path.display()))
                }
                Err(e) => {
                    let msg = format!("{}: {e}", path.display());
                    set_error(msg.clone());
                    Err(msg)
                }
            }
        }
    }
}

fn set_error(msg: String) {
    *ERROR.write().expect("config lock poisoned") = Some(msg);
}

/// Handy for unit tests: parse source without touching global state.
#[cfg(test)]
pub(crate) fn parse_for_test(src: &str) -> ConfigResult<Config> {
    Config::parse(src, &Config::default())
}

// ── Tests ─────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_template_parses_and_covers_every_action() {
        let cfg = Config::default();
        assert_eq!(cfg.colors.accent, Color32::from_rgb(200, 120, 60));
        assert_eq!(cfg.numbers.max_side, 4096);
        for action in ALL {
            assert!(cfg.keys.contains_key(action), "missing binding for {}", action.name());
        }
    }

    #[test]
    fn partial_user_config_keeps_defaults_and_overrides_fields() {
        let cfg = parse_for_test(
            r##"
            colors = { accent = "#010203" }
            constants = { font_size = 33, max_side = 128 }
            keys = { undo = "Ctrl+Y" }
            "##,
        ).unwrap();
        assert_eq!(cfg.colors.accent, Color32::from_rgb(1, 2, 3));
        assert_eq!(cfg.colors.bg, Config::default().colors.bg);
        assert_eq!(cfg.numbers.font_size, 33.0);
        assert_eq!(cfg.numbers.max_side, 128);
        assert_eq!(cfg.keys[&Action::Undo], vec![parse_combo("Ctrl+Y").unwrap()]);
        assert_eq!(cfg.keys[&Action::Redo], Config::default().keys[&Action::Redo]);

        let disabled = parse_for_test("keys = { undo = false }").unwrap();
        assert!(!disabled.keys.contains_key(&Action::Undo));
        assert!(disabled.keys.contains_key(&Action::Redo));
    }

    #[test]
    fn palette_accepts_rgba_tables() {
        let cfg = parse_for_test("colors = { palette = { '#01020304', {9, 8, 7} } }").unwrap();
        assert_eq!(cfg.palette[0], [1, 2, 3, 4]);
        assert_eq!(cfg.palette[1], [9, 8, 7, 255]);
        assert_eq!(cfg.palette[2], Config::default().palette[2]);
    }

    #[test]
    fn invalid_values_are_rejected() {
        assert!(parse_for_test("colors = { bg = 'red' }").is_err());
        assert!(parse_for_test("constants = { font_size = -5 }").is_err());
        assert!(parse_for_test("keys = { no_such_action = 'A' }").is_err());
        assert!(parse_for_test("keys = { undo = 'Ctrl+Unknown' }").is_err());
        assert!(parse_for_test("this is not lua").is_err());
    }

    #[test]
    fn key_specs_parse_modifiers_and_symbols() {
        assert_eq!(parse_combo("Ctrl+Shift+Z").unwrap(), KeyCombo {
            modifiers: Modifiers::CTRL | Modifiers::SHIFT, key: Key::Z,
        });
        assert_eq!(parse_combo(",").unwrap(), KeyCombo { modifiers: Modifiers::NONE, key: Key::Comma });
        assert_eq!(parse_combo("Ctrl++").unwrap(), KeyCombo { modifiers: Modifiers::CTRL, key: Key::Plus });
        assert_eq!(parse_combo("Alt+Left").unwrap(), KeyCombo { modifiers: Modifiers::ALT, key: Key::ArrowLeft });
        assert!(parse_combo("Ctrl+").is_ok());
        assert!(parse_combo("Ctrl+Nope").is_err());
    }

    #[test]
    fn more_specific_bindings_come_first() {
        let cfg = Config::default();
        let pairs = cfg.bindings();
        let weight = |c: KeyCombo| {
            [c.modifiers.ctrl, c.modifiers.shift, c.modifiers.alt, c.modifiers.mac_cmd]
                .iter().filter(|v| **v).count()
        };
        assert!(weight(pairs[0].1) >= weight(pairs[pairs.len() - 1].1));
    }
}
