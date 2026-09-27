//! Эффекты, портированные из открытого кода Pixelorama
//! (<https://github.com/Orama-Interactive/Pixelorama>).
//!
//! В оригинале эффекты — это GLSL-шейдеры (`src/Shaders/Effects/*.gdshaderinc`),
//! а список и категории берутся из меню Effects
//! (`src/UI/TopMenuContainer/TopMenuContainer.gd::_setup_effects_menu`).
//! Здесь те же эффекты переписаны в обычный CPU-код поверх `Vec<Color32>`.
//!
//! Структура пакета повторяет оригинальную:
//!   * `Cat`      — категории меню (Transform / Color / Procedural / Blur);
//!   * `EffectDef` — описание эффекта: имя, категория и список параметров;
//!   * `EffectParams` — значения параметров (числа / флаги / цвета).
//!
//! Значения параметров хранятся «плоско» (по имени), чтобы список эффектов
//! можно было расширять новыми строками в `EFFECTS` без правок логики.

use std::collections::HashMap;

use eframe::egui::Color32;

use crate::color::{hsv_to_rgb, rgb_to_hsv};

// ── категории (как в меню Effects оригинала) ──────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Cat {
    Transform,
    Color,
    Procedural,
    Blur,
}

impl Cat {
    pub fn name(self) -> &'static str {
        match self {
            Cat::Transform => "Transform",
            Cat::Color => "Color",
            Cat::Procedural => "Procedural",
            Cat::Blur => "Blur",
        }
    }
}

pub const CATS: [Cat; 4] = [Cat::Transform, Cat::Color, Cat::Procedural, Cat::Blur];

// ── типы параметров ───────────────────────────────────

#[derive(Clone, Copy)]
pub enum PT {
    /// ползунок с шагом 1
    I(i32, i32),
    /// ползунок с дробным шагом
    F(f32, f32),
    /// чекбокс
    B,
    /// выбор цвета
    C,
    /// сегментированный выбор из нескольких вариантов
    E(&'static [&'static str]),
}

pub struct ParamDef {
    pub name: &'static str,
    pub ty: PT,
}

const fn i_(name: &'static str, min: i32, max: i32) -> ParamDef {
    ParamDef { name, ty: PT::I(min, max) }
}
const fn f_(name: &'static str, min: f32, max: f32) -> ParamDef {
    ParamDef { name, ty: PT::F(min, max) }
}
const fn b_(name: &'static str) -> ParamDef {
    ParamDef { name, ty: PT::B }
}
const fn c_(name: &'static str) -> ParamDef {
    ParamDef { name, ty: PT::C }
}
const fn e_(name: &'static str, labels: &'static [&'static str]) -> ParamDef {
    ParamDef { name, ty: PT::E(labels) }
}

// ── сами эффекты ──────────────────────────────────────

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EffectKind {
    // Transform
    OffsetScale,
    Mirror,
    Rotate,
    FlatToIsometric,
    // Color
    Invert,
    Desaturate,
    Hsv,
    BrightnessContrast,
    ColorCurves,
    Palettize,
    Posterize,
    GradientMap,
    // Procedural
    Outline,
    DropShadow,
    Gradient,
    // Blur
    Pixelize,
    GaussianBlur,
}

pub struct EffectDef {
    pub kind: EffectKind,
    pub name: &'static str,
    pub cat: Cat,
    pub params: &'static [ParamDef],
}

/// Порядок вариантов `EffectKind` обязан совпадать с порядком `EFFECTS`
/// (проверяется тестом внизу файла).
pub static EFFECTS: &[EffectDef] = &[
    // ── Transform ────────────────────────────────────
    EffectDef {
        kind: EffectKind::OffsetScale,
        name: "Offset & Scale",
        cat: Cat::Transform,
        params: &[
            i_("Offset X", -64, 64),
            i_("Offset Y", -64, 64),
            f_("Scale", 1.0, 400.0),
            b_("Wrap Around"),
        ],
    },
    EffectDef {
        kind: EffectKind::Mirror,
        name: "Mirror Image",
        cat: Cat::Transform,
        params: &[b_("Horizontal"), b_("Vertical")],
    },
    EffectDef {
        kind: EffectKind::Rotate,
        name: "Rotate Image",
        cat: Cat::Transform,
        params: &[f_("Angle", -180.0, 180.0)],
    },
    EffectDef {
        kind: EffectKind::FlatToIsometric,
        name: "Flat to Isometric",
        cat: Cat::Transform,
        params: &[i_("Origin", 0, 64), i_("Step", 0, 16), i_("Deadzone", 0, 16)],
    },
    // ── Color ────────────────────────────────────────
    EffectDef {
        kind: EffectKind::Invert,
        name: "Invert Colors",
        cat: Cat::Color,
        params: &[b_("Red"), b_("Green"), b_("Blue"), b_("Alpha")],
    },
    EffectDef {
        kind: EffectKind::Desaturate,
        name: "Desaturation",
        cat: Cat::Color,
        params: &[b_("Red"), b_("Green"), b_("Blue"), b_("Alpha")],
    },
    EffectDef {
        kind: EffectKind::Hsv,
        name: "Adjust Hue/Sat/Value",
        cat: Cat::Color,
        params: &[
            i_("Hue", -180, 180),
            i_("Saturation", -100, 100),
            i_("Value", -100, 100),
            b_("Wrap Overflowing"),
        ],
    },
    EffectDef {
        kind: EffectKind::BrightnessContrast,
        name: "Adjust Brightness/Contrast",
        cat: Cat::Color,
        params: &[
            i_("Brightness", -100, 100),
            i_("Contrast", 0, 300),
            i_("Saturation", 0, 300),
            i_("Red Shift", -100, 100),
            i_("Green Shift", -100, 100),
            i_("Blue Shift", -100, 100),
            b_("Wrap Overflowing"),
        ],
    },
    EffectDef {
        kind: EffectKind::ColorCurves,
        name: "Color Curves",
        cat: Cat::Color,
        params: &[
            f_("Red Gamma", 0.1, 4.0),
            f_("Red Black", 0.0, 1.0),
            f_("Red White", 0.0, 1.0),
            f_("Green Gamma", 0.1, 4.0),
            f_("Green Black", 0.0, 1.0),
            f_("Green White", 0.0, 1.0),
            f_("Blue Gamma", 0.1, 4.0),
            f_("Blue Black", 0.0, 1.0),
            f_("Blue White", 0.0, 1.0),
        ],
    },
    EffectDef {
        kind: EffectKind::Palettize,
        name: "Palettize",
        cat: Cat::Color,
        params: &[i_("Colors", 2, 64)],
    },
    EffectDef {
        kind: EffectKind::Posterize,
        name: "Posterize",
        cat: Cat::Color,
        params: &[f_("Colors", 1.0, 32.0), f_("Dither", 0.0, 0.5)],
    },
    EffectDef {
        kind: EffectKind::GradientMap,
        name: "Gradient Map",
        cat: Cat::Color,
        params: &[
            c_("Shadow"),
            c_("Mid"),
            c_("Highlight"),
        ],
    },
    // ── Procedural ───────────────────────────────────
    EffectDef {
        kind: EffectKind::Outline,
        name: "Outline",
        cat: Cat::Procedural,
        params: &[
            c_("Color"),
            i_("Width", 0, 10),
            e_("Brush", &["Diamond", "Circle", "Square"]),
            b_("Inside"),
            b_("Border Only"),
        ],
    },
    EffectDef {
        kind: EffectKind::DropShadow,
        name: "Drop Shadow",
        cat: Cat::Procedural,
        params: &[i_("Offset X", -32, 32), i_("Offset Y", -32, 32), c_("Color")],
    },
    EffectDef {
        kind: EffectKind::Gradient,
        name: "Gradient",
        cat: Cat::Procedural,
        params: &[
            c_("From"),
            c_("To"),
            e_("Shape", &["Linear", "Radial"]),
            e_("Repeat", &["None", "Repeat", "Mirrored"]),
            f_("Angle", 0.0, 360.0),
            f_("Size", 0.05, 2.0),
            f_("Position", -0.5, 0.5),
            b_("Dithering"),
        ],
    },
    // ── Blur ─────────────────────────────────────────
    EffectDef {
        kind: EffectKind::Pixelize,
        name: "Pixelize",
        cat: Cat::Blur,
        params: &[i_("Size X", 1, 64), i_("Size Y", 1, 64)],
    },
    EffectDef {
        kind: EffectKind::GaussianBlur,
        name: "Gaussian Blur",
        cat: Cat::Blur,
        params: &[
            e_("Type", &["Spiral", "Monk's", "NoDev Single", "NoDev Multi"]),
            i_("Amount", 0, 32),
            f_("Radius", 0.0, 32.0),
        ],
    },
];

pub fn def(kind: EffectKind) -> &'static EffectDef {
    debug_assert_eq!(EFFECTS[kind as usize].kind, kind);
    &EFFECTS[kind as usize]
}

/// Значения параметров эффекта. Хранятся по имени, а не по индексу,
/// чтобы добавление параметра в `EFFECTS` не ломало существующий код.
pub struct EffectParams {
    pub kind: EffectKind,
    /// числа (ползунки) — целые и дробные хранятся вместе
    pub num: Vec<f64>,
    /// чекбоксы и варианты сегментированного выбора (индекс в `E`)
    pub flag: Vec<f64>,
    /// цвета
    pub col: Vec<Color32>,
}

impl EffectParams {
    /// Параметры по умолчанию для эффекта (значения — середины диапазонов,
    /// как в диалогах Pixelorama).
    pub fn new(kind: EffectKind) -> Self {
        let d = def(kind);
        let mut num = Vec::new();
        let mut flag = Vec::new();
        let mut col = Vec::new();
        for p in d.params {
            match p.ty {
                PT::I(..) | PT::F(..) => num.push(default_num(p)),
                PT::B => flag.push(if default_bool(p.name) { 1.0 } else { 0.0 }),
                PT::C => col.push(default_color(p.name)),
                PT::E(_) => flag.push(0.0),
            }
        }
        Self { kind, num, flag, col }
    }

    pub fn def(&self) -> &'static EffectDef {
        def(self.kind)
    }

    /// Есть ли параметр с таким именем у текущего эффекта.
    /// Нужна UI: список параметров может оказаться от предыдущего эффекта,
    /// и обращаться к нему по имени тогда нельзя.
    pub fn has(&self, name: &str) -> bool {
        self.def().params.iter().any(|p| p.name == name)
    }

    fn num_ix(&self, name: &str) -> usize {
        let mut n = 0usize;
        for p in self.def().params {
            match p.ty {
                PT::I(_, _) | PT::F(_, _) => {
                    if p.name == name {
                        return n;
                    }
                    n += 1;
                }
                _ => {}
            }
        }
        panic!("unknown numeric effect param: {name}");
    }

    fn flag_ix(&self, name: &str) -> usize {
        let mut n = 0usize;
        for p in self.def().params {
            match p.ty {
                PT::B | PT::E(_) => {
                    if p.name == name {
                        return n;
                    }
                    n += 1;
                }
                _ => {}
            }
        }
        panic!("unknown flag effect param: {name}");
    }

    fn col_ix(&self, name: &str) -> usize {
        let mut n = 0usize;
        for p in self.def().params {
            if matches!(p.ty, PT::C) {
                if p.name == name {
                    return n;
                }
                n += 1;
            }
        }
        panic!("unknown color effect param: {name}");
    }

    // ── доступ к значениям ──
    pub fn f(&self, name: &str) -> f32 {
        self.num[self.num_ix(name)] as f32
    }
    /// Ссылка на числовой параметр — для слайдеров в UI.
    pub fn num_mut(&mut self, name: &str) -> &mut f64 {
        let i = self.num_ix(name);
        &mut self.num[i]
    }
    /// Целое значение ползунка (округлённое).
    pub fn i(&self, name: &str) -> i32 {
        self.num[self.num_ix(name)].round() as i32
    }
    pub fn flag(&self, name: &str) -> bool {
        self.flag[self.flag_ix(name)] > 0.5
    }
    pub fn set_flag(&mut self, name: &str, v: bool) {
        let i = self.flag_ix(name);
        self.flag[i] = if v { 1.0 } else { 0.0 };
    }
    /// Индекс выбранного варианта для `PT::E`.
    pub fn choice(&self, name: &str) -> usize {
        self.flag[self.flag_ix(name)].round().max(0.0) as usize
    }
    pub fn set_choice(&mut self, name: &str, v: usize) {
        let i = self.flag_ix(name);
        self.flag[i] = v as f64;
    }
    pub fn color(&self, name: &str) -> Color32 {
        self.col[self.col_ix(name)]
    }
    pub fn set_color(&mut self, name: &str, c: Color32) {
        let i = self.col_ix(name);
        self.col[i] = c;
    }

    /// Подпись состояния — используется как ключ кэша превью и как ID
    /// анимации egui, чтобы не пересчитывать превью без изменений.
    pub fn signature(&self) -> String {
        let d = self.def();
        let mut s = String::with_capacity(64);
        s.push_str(d.name);
        for p in d.params {
            s.push('|');
            s.push_str(p.name);
            s.push('=');
            match p.ty {
                PT::I(..) | PT::F(..) => s.push_str(&format!("{:.3}", self.num[self.num_ix(p.name)])),
                PT::B | PT::E(_) => s.push_str(&format!("{}", self.flag[self.flag_ix(p.name)])),
                PT::C => s.push_str(&format!("{:?}", self.col[self.col_ix(p.name)])),
            }
        }
        s
    }
}

/// Значения по умолчанию — как в диалогах Pixelorama.
fn default_num(pd: &ParamDef) -> f64 {
    let name = pd.name;
    // Одни и те же имена встречаются в разных эффектах, поэтому часть
    // умолчаний различается по диапазону параметра.
    let (lo, hi) = match pd.ty {
        PT::I(lo, hi) => (lo, hi),
        PT::F(lo, hi) => (lo as i32, hi as i32),
        _ => (0, 0),
    };
    match name {
        // Transform
        "Scale" => 100.0,
        "Step" => 2.0,
        "Origin" => 32.0,
        // Color
        "Contrast" => 100.0,
        "Saturation" if lo < 0 => 0.0,   // сдвиг в HSV
        "Saturation" => 100.0,           // множитель в Brightness/Contrast
        "Black" => 0.0,
        "White" => 1.0,
        "Gamma" => 1.0,
        "Colors" => match pd.ty {
            PT::I(..) => 16.0, // Palettize
            _ => 2.0,          // Posterize
        },
        "Dither" => 0.0,
        // Procedural
        "Width" => 1.0,
        // тень по умолчанию сдвинута на 5,5 (так в DropShadow.gdshaderinc),
        // а у Offset & Scale сдвиг по умолчанию нулевой
        "Offset X" | "Offset Y" => match (lo, hi) {
            (-32, 32) => 5.0,
            _ => 0.0,
        },
        "Angle" => 0.0,
        "Size" => 1.0,
        "Position" => 0.0,
        // Blur
        "Size X" | "Size Y" => 2.0,
        "Amount" => 8.0,
        "Radius" => 1.0,
        // Hue/Value — нейтральное смещение
        "Hue" | "Value" => 0.0,
        _ => 0.0,
    }
}

fn default_bool(name: &str) -> bool {
    // чекбоксы каналов и горизонтальное отражение — включены, остальное выключено
    matches!(name, "Red" | "Green" | "Blue" | "Horizontal")
}

fn default_color(name: &str) -> Color32 {
    match name {
        // тень и контур по умолчанию чёрные
        "Color" => Color32::from_rgb(0, 0, 0),
        "Shadow" => Color32::from_rgb(0, 0, 0),
        "Mid" => Color32::from_rgb(128, 128, 128),
        "Highlight" => Color32::from_rgb(255, 255, 255),
        "From" => Color32::from_rgb(0, 0, 0),
        "To" => Color32::from_rgb(255, 255, 255),
        _ => Color32::WHITE,
    }
}

// ── преобразования цвета ──────────────────────────────

/// sRGB канал → линейный (для корректной яркости, как в Desaturate.gdshaderinc).
#[inline]
fn srgb_to_linear(x: f32) -> f32 {
    if x < 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).powf(2.4) }
}

#[inline]
fn linear_to_srgb(x: f32) -> f32 {
    if x > 0.0031308 { 1.055 * x.powf(1.0 / 2.4) - 0.055 } else { x * 12.92 }
}

#[inline]
fn q(v: f32) -> f32 { v.clamp(0.0, 1.0) }

#[inline]
fn b255(v: f32) -> u8 { (q(v) * 255.0 + 0.5) as u8 }

/// `Color32` → нелинейный RGBA в диапазоне 0..1.
#[inline]
fn rgba(c: Color32) -> [f32; 4] {
    let s = c.to_srgba_unmultiplied();
    [s[0] as f32 / 255.0, s[1] as f32 / 255.0, s[2] as f32 / 255.0, s[3] as f32 / 255.0]
}

#[inline]
fn pack(c: [f32; 4]) -> Color32 {
    Color32::from_rgba_unmultiplied(b255(c[0]), b255(c[1]), b255(c[2]), b255(c[3]))
}

#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 { a + (b - a) * t }

/// Пиксель внутри выделения? `None` — выделения нет, значит весь слой.
#[inline]
fn in_sel(sel: Option<(i32, i32, i32, i32)>, x: i32, y: i32) -> bool {
    match sel {
        Some((x0, y0, x1, y1)) => x >= x0 && x <= x1 && y >= y0 && y <= y1,
        None => true,
    }
}

/// Сдвиг значения по шкале 0..max: с клампом или с переносом через край
/// (флаг `Wrap Overflowing` в оригинальных шейдерах).
#[inline]
fn shift_scale(v: i32, d: i32, max: i32, wrap: bool) -> i32 {
    if d == 0 { return v; }
    let s = v + d;
    if wrap { s.rem_euclid(max.max(1)) } else { s.clamp(0, max) }
}

// ── главная точка входа ───────────────────────────────

/// Применяет эффект к пикселям слоя. `pixels.len()` должен быть `w * h`.
pub fn apply(
    pixels: &mut Vec<Color32>,
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) {
    if w == 0 || h == 0 || pixels.len() != w * h {
        return;
    }
    let src = std::mem::take(pixels);
    let out = run(p.kind, &src, w, h, p, sel);
    *pixels = out;
}

fn run(
    kind: EffectKind,
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    match kind {
        EffectKind::Invert => fx_invert(src, w, h, p, sel),
        EffectKind::Desaturate => fx_desaturate(src, w, h, p, sel),
        EffectKind::Hsv => fx_hsv(src, w, h, p, sel),
        EffectKind::BrightnessContrast => fx_brightness_contrast(src, w, h, p, sel),
        EffectKind::ColorCurves => fx_color_curves(src, w, h, p, sel),
        EffectKind::Palettize => fx_palettize(src, w, h, p, sel),
        EffectKind::Posterize => fx_posterize(src, w, h, p, sel),
        EffectKind::GradientMap => fx_gradient_map(src, w, h, p, sel),
        EffectKind::Outline => fx_outline(src, w, h, p, sel),
        EffectKind::DropShadow => fx_drop_shadow(src, w, h, p, sel),
        EffectKind::Gradient => fx_gradient(src, w, h, p, sel),
        EffectKind::Pixelize => fx_pixelize(src, w, h, p, sel),
        EffectKind::GaussianBlur => fx_gaussian_blur(src, w, h, p, sel),
        EffectKind::OffsetScale => fx_offset_scale(src, w, h, p, sel),
        EffectKind::Mirror => fx_mirror(src, w, h, p, sel),
        EffectKind::Rotate => fx_rotate(src, w, h, p, sel),
        EffectKind::FlatToIsometric => fx_flat_to_isometric(src, w, h, p, sel),
    }
}

// ── Color ─────────────────────────────────────────────

// Invert.gdshaderinc
fn fx_invert(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let (r, g, b, a) = (p.flag("Red"), p.flag("Green"), p.flag("Blue"), p.flag("Alpha"));
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let mut c = rgba(src[y * w + x]);
            if r { c[0] = 1.0 - c[0]; }
            if g { c[1] = 1.0 - c[1]; }
            if b { c[2] = 1.0 - c[2]; }
            if a { c[3] = 1.0 - c[3]; }
            out[y * w + x] = pack(c);
        }
    }
    out
}

// Desaturate.gdshaderinc — яркость считается в линейном пространстве.
fn fx_desaturate(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let (dr, dg, db, da) = (p.flag("Red"), p.flag("Green"), p.flag("Blue"), p.flag("Alpha"));
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let c = rgba(src[y * w + x]);
            let lin = [
                srgb_to_linear(c[0]),
                srgb_to_linear(c[1]),
                srgb_to_linear(c[2]),
            ];
            let lum = 0.21264935 * lin[0] + 0.71516913 * lin[1] + 0.07218152 * lin[2];
            let des = [
                if dr { lum } else { lin[0] },
                if dg { lum } else { lin[1] },
                if db { lum } else { lin[2] },
            ];
            let mut o = [
                linear_to_srgb(q(des[0])),
                linear_to_srgb(q(des[1])),
                linear_to_srgb(q(des[2])),
                c[3],
            ];
            if da { o[3] = q(linear_to_srgb(q(lum))); }
            out[y * w + x] = pack(o);
        }
    }
    out
}

// HSV.gdshaderinc — сдвиги в градусах / процентах, как в оригинале.
fn fx_hsv(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let hue = p.i("Hue");
    let sat = p.i("Saturation");
    let val = p.i("Value");
    let wrap = p.flag("Wrap Overflowing");
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let c = rgba(src[y * w + x]);
            if c[3] <= 0.0 { continue; }
            let (h0, s0, v0) = rgb_to_hsv(b255(c[0]), b255(c[1]), b255(c[2]));
            let mut h8 = ((h0.round() as i32) % 360 + 360) % 360;
            let mut s8 = (s0 * 100.0 / 255.0).round() as i32;
            let mut v8 = (v0 * 100.0 / 255.0).round() as i32;
            if c[0] != c[1] || c[1] != c[2] {
                h8 = shift_scale(h8, hue, 360, wrap);
            }
            s8 = shift_scale(s8, sat, 100, wrap);
            v8 = shift_scale(v8, val, 100, wrap);
            let (r, g, b) = hsv_to_rgb(h8 as f32, s8 as f32 * 255.0 / 100.0, v8 as f32 * 255.0 / 100.0);
            out[y * w + x] = Color32::from_rgba_unmultiplied(r, g, b, b255(c[3]));
        }
    }
    out
}

// BrightnessContrast.gdshaderinc — матрицы насыщенности/контраста/яркости.
fn fx_brightness_contrast(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let bright = p.f("Brightness") / 100.0;
    let contrast = p.f("Contrast") / 100.0;
    let sat = p.f("Saturation") / 100.0;
    let rs = p.f("Red Shift") / 100.0;
    let gs = p.f("Green Shift") / 100.0;
    let bs = p.f("Blue Shift") / 100.0;
    let wrap = p.flag("Wrap Overflowing");
    let oms = 1.0 - sat;
    let lum = [0.3086f32, 0.6094, 0.0820];
    let t = (1.0 - contrast) * 0.5;

    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let orig = rgba(src[y * w + x]);
            let mut c = orig;
            for (k, sh) in [rs, gs, bs].iter().enumerate() {
                if *sh != 0.0 {
                    c[k] = if wrap { (c[k] + sh).rem_euclid(1.0) } else { q(c[k] + sh) };
                }
            }
            // матрица насыщенности (вклад каналов одинаковый: 1.0)
            let mut s = [0.0f32; 3];
            for (k, out_ch) in s.iter_mut().enumerate() {
                let mut acc = 0.0;
                for j in 0..3 {
                    acc += (lum[j] * oms + if j == k { sat } else { 0.0 }) * c[j];
                }
                *out_ch = acc;
            }
            // контраст, затем яркость
            for v in s.iter_mut() {
                *v = q(*v * contrast + t + bright);
            }
            out[y * w + x] = pack([s[0], s[1], s[2], orig[3]]);
        }
    }
    out
}

// ColorCurves.gdshaderinc — на канал: гамма + чёрная/белая точка.
fn curve(x: f32, gamma: f32, black: f32, white: f32) -> f32 {
    if (white - black).abs() < 1e-6 { return q(x); }
    let t = q((x - black) / (white - black));
    q(t.powf(gamma) * (white - black) + black)
}

fn fx_color_curves(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let rg = p.f("Red Gamma");
    let rb = p.f("Red Black");
    let rw = p.f("Red White");
    let gg = p.f("Green Gamma");
    let gb = p.f("Green Black");
    let gw = p.f("Green White");
    let bg = p.f("Blue Gamma");
    let bb = p.f("Blue Black");
    let bw = p.f("Blue White");
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let c = rgba(src[y * w + x]);
            out[y * w + x] = pack([
                curve(c[0], rg, rb, rw),
                curve(c[1], gg, gb, gw),
                curve(c[2], bg, bb, bw),
                c[3],
            ]);
        }
    }
    out
}

// Palettize.gdshaderinc — ближайший цвет из палитры (евклидово расстояние).
// Сама палитра строится из изображения алгоритмом median cut.
fn fx_palettize(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let want = p.i("Colors").clamp(2, 256) as usize;
    let mut counts: HashMap<[u8; 4], u32> = HashMap::new();
    for px in src {
        *counts.entry(px.to_srgba_unmultiplied()).or_insert(0) += 1;
    }
    let mut uniq: Vec<([u8; 4], u32)> = counts.into_iter().collect();
    uniq.sort_unstable_by_key(|(_, n)| std::cmp::Reverse(*n));
    let colors: Vec<[u8; 4]> = uniq.iter().map(|(c, _)| *c).collect();
    let weights: Vec<u32> = uniq.iter().map(|(_, n)| *n).collect();
    let palette = median_cut(&colors, &weights, want);
    if palette.is_empty() { return src.to_vec(); }

    // запоминаем сопоставление: у пиксель-арта мало уникальных цветов
    let mut cache: HashMap<[u8; 4], Color32> = HashMap::new();
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let i = y * w + x;
            let key = src[i].to_srgba_unmultiplied();
            if key[3] <= 2 { continue; } // прозрачные не трогаем (alpha <= 0.01 в шейдере)
            let mapped = *cache.entry(key).or_insert_with(|| {
                let mut best = palette[0];
                let mut bd = i32::MAX;
                for c in palette.iter() {
                    let dr = c[0] as i32 - key[0] as i32;
                    let dg = c[1] as i32 - key[1] as i32;
                    let db = c[2] as i32 - key[2] as i32;
                    let da = c[3] as i32 - key[3] as i32;
                    let d = dr * dr + dg * dg + db * db + da * da;
                    if d < bd { bd = d; best = *c; }
                }
                Color32::from_rgba_unmultiplied(best[0], best[1], best[2], best[3])
            });
            out[i] = mapped;
        }
    }
    out
}

/// Median cut: делим цветовое пространство на `want` коробок по самому
/// длинному каналу, цветом коробки считаем взвешенное среднее.
fn median_cut(colors: &[[u8; 4]], weights: &[u32], want: usize) -> Vec<[u8; 4]> {
    if colors.is_empty() { return Vec::new(); }
    let mut boxes: Vec<Vec<usize>> = vec![(0..colors.len()).collect()];
    while boxes.len() < want {
        let mut best = usize::MAX;
        let mut best_ch = 0usize;
        let mut best_range = 0i32;
        for (bi, b) in boxes.iter().enumerate() {
            if b.len() < 2 { continue; }
            for (ch, range) in (0..3).map(|ch| {
                let mn = b.iter().map(|&i| colors[i][ch]).min().unwrap();
                let mx = b.iter().map(|&i| colors[i][ch]).max().unwrap();
                (ch, mx as i32 - mn as i32)
            }) {
                if range > best_range { best_range = range; best = bi; best_ch = ch; }
            }
        }
        if best == usize::MAX || best_range == 0 { break; }
        let mut items = std::mem::take(&mut boxes[best]);
        items.sort_unstable_by_key(|&i| colors[i][best_ch]);
        let total: u32 = items.iter().map(|&i| weights[i]).sum();
        let half = total / 2;
        let mut acc = 0u32;
        let mut cut = 1usize;
        for (k, &i) in items.iter().enumerate() {
            acc += weights[i];
            cut = k + 1;
            if acc >= half { break; }
        }
        if cut == 0 || cut >= items.len() { boxes[best] = items; break; }
        let right = items.split_off(cut);
        boxes[best] = items;
        boxes.push(right);
    }
    boxes
        .iter()
        .map(|b| {
            let mut acc = [0u64; 4];
            let mut wsum = 0u64;
            for &i in b {
                let wt = weights[i] as u64;
                for (k, a) in acc.iter_mut().enumerate() {
                    *a += colors[i][k] as u64 * wt;
                }
                wsum += wt;
            }
            // взвешенное среднее по цвету коробки
            let avg = |a: u64| a.checked_div(wsum).unwrap_or(0) as u8;
            [avg(acc[0]), avg(acc[1]), avg(acc[2]), avg(acc[3])]
        })
        .collect()
}

// Posterize.gdshaderinc — с шахматным дизерингом.
fn fx_posterize(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let levels = p.f("Colors").max(1.0);
    let dith = p.f("Dither");
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let c = rgba(src[y * w + x]);
            let checker = ((x + y) % 2) as f32;
            let mut o = [0.0f32; 4];
            for k in 0..3 {
                let up = (q(c[k]) * levels + dith).round() / levels;
                let dn = (q(c[k]) * levels - dith).round() / levels;
                o[k] = up * checker + dn * (1.0 - checker);
            }
            o[3] = c[3];
            out[y * w + x] = pack(o);
        }
    }
    out
}

// GradientMap.gdshaderinc — яркость → цвет из трёх опорных точек.
fn fx_gradient_map(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let shadow = rgba(p.color("Shadow"));
    let mid = rgba(p.color("Mid"));
    let high = rgba(p.color("Highlight"));
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let c = rgba(src[y * w + x]);
            let v = 0.2126 * c[0] + 0.7152 * c[1] + 0.0722 * c[2];
            let g = if v < 0.5 {
                let t = v * 2.0;
                [lerp(shadow[0], mid[0], t), lerp(shadow[1], mid[1], t), lerp(shadow[2], mid[2], t),
                 lerp(shadow[3], mid[3], t)]
            } else {
                let t = (v - 0.5) * 2.0;
                [lerp(mid[0], high[0], t), lerp(mid[1], high[1], t), lerp(mid[2], high[2], t),
                 lerp(mid[3], high[3], t)]
            };
            out[y * w + x] = pack([g[0], g[1], g[2], c[3] * g[3]]);
        }
    }
    out
}

// ── Procedural ────────────────────────────────────────

// OutlineInline.gdshaderinc
fn fx_outline(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let color = rgba(p.color("Color"));
    let width = p.f("Width").max(0.0);
    let brush = p.choice("Brush");
    let inside = p.flag("Inside");
    let border_only = p.flag("Border Only");
    let wi = width.ceil() as i32;
    let mut out = src.to_vec();

    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let i = y * w + x;
            let orig = rgba(src[i]);
            let mut contrary = false;
            'scan: for dy in -wi..=wi {
                // профиль кисти: ромб / круг / квадрат
                let off = match brush {
                    0 => width - (dy as f32).abs(),
                    1 => ((width + 0.5) * (width + 0.5) - (dy * dy) as f32).sqrt().floor(),
                    _ => width,
                };
                let oi = off.ceil() as i32;
                for dx in -oi..=oi {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    let oob = nx < 0 || ny < 0 || nx >= w as i32 || ny >= h as i32;
                    let na = if oob {
                        0.0
                    } else {
                        rgba(src[ny as usize * w + nx as usize])[3]
                    };
                    // «противоположный» сосед: пустой снаружи / заполненный внутри
                    let hit = if inside { !oob && na > 0.0 } else { oob || na <= 0.0 };
                    if hit { contrary = true; break 'scan; }
                }
            }

            let mut o = orig;
            if ((orig[3] > 0.0) == inside) && contrary {
                o = if inside {
                    [lerp(orig[0], color[0], color[3]),
                     lerp(orig[1], color[1], color[3]),
                     lerp(orig[2], color[2], color[3]),
                     orig[3]]
                } else {
                    color
                };
                o[3] = if color[3] <= 0.0 { 0.0 } else { q(orig[3] + (1.0 - orig[3]) * color[3]) };
            } else if border_only {
                o[3] = 0.0;
            }
            out[i] = pack(o);
        }
    }
    out
}

// DropShadow.gdshaderinc — тень из альфы, сдвинутой на offset.
fn fx_drop_shadow(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let ox = p.i("Offset X");
    let oy = p.i("Offset Y");
    let sc = rgba(p.color("Color"));
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let orig = rgba(src[y * w + x]);
            // тень не рисуется поверх самого спрайта
            if orig[3] > 0.0 || ox == 0 && oy == 0 {
                out[y * w + x] = pack(orig);
                continue;
            }
            let sx = x as i32 - ox;
            let sy = y as i32 - oy;
            if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                out[y * w + x] = pack(orig);
                continue;
            }
            // тень обрезается выделением в своей точке
            let mut s = if in_sel(sel, sx, sy) {
                rgba(src[sy as usize * w + sx as usize])[3] * sc[3]
            } else {
                0.0
            };
            s = q(s);
            out[y * w + x] = pack([
                lerp(orig[0], sc[0], s),
                lerp(orig[1], sc[1], s),
                lerp(orig[2], sc[2], s),
                lerp(orig[3], 1.0, s),
            ]);
        }
    }
    out
}

// Gradient.gdshaderinc — упрощён до двух опорных цветов + дизеринг 4x4.
const BAYER4: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

fn mirror_fract(v: f32) -> f32 {
    let f = v.fract();
    if v < 0.0 { 1.0 - f } else { f }
}

fn fx_gradient(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let from = rgba(p.color("From"));
    let to = rgba(p.color("To"));
    let shape = p.choice("Shape");
    let repeat = p.choice("Repeat");
    let angle = p.f("Angle");
    let size = p.f("Size").max(0.01);
    let position = p.f("Position");
    let dither = p.flag("Dithering");

    let rad = angle.to_radians();
    let (cs, sn) = (rad.cos(), rad.sin());
    let norm = cs.abs() + sn.abs();
    let pos = position / size;

    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let orig = rgba(src[y * w + x]);
            let u = (x as f32 + 0.5) / w as f32;
            let v = (y as f32 + 0.5) / h as f32;

            let mut m = if shape == 0 {
                // линейный градиент вокруг центра, с поворотом
                let ox = u - 0.5;
                let oy = v - 0.5;
                let mut t = ox * cs - oy * sn;
                t /= norm;
                t /= size;
                t - (pos - 0.5)
            } else {
                // радиальный градиент от центра
                let ox = u * 2.0 - 1.0;
                let oy = v * 2.0 - 1.0;
                (ox * ox + oy * oy).sqrt()
            };
            match repeat {
                1 => m = m.fract(),
                2 => m = mirror_fract(m),
                _ => {}
            }
            let in_range = (0.0..=1.0).contains(&m);
            let mut t = q(m);
            if dither {
                let threshold = (BAYER4[y % 4][x % 4] + 0.5) / 16.0;
                t = if t <= threshold { 0.0 } else { 1.0 };
            }
            let mut g = [
                lerp(from[0], to[0], t),
                lerp(from[1], to[1], t),
                lerp(from[2], to[2], t),
                lerp(from[3], to[3], t),
            ];
            if !in_range { g[3] = 0.0; }
            // рисуем градиент поверх, используя его альфу
            out[y * w + x] = pack([
                lerp(orig[0], g[0], g[3]),
                lerp(orig[1], g[1], g[3]),
                lerp(orig[2], g[2], g[3]),
                orig[3],
            ]);
        }
    }
    out
}

// ── Blur ──────────────────────────────────────────────

// Pixelize.gdshaderinc — выборка из «крупного пикселя».
fn fx_pixelize(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let bx = (p.i("Size X").max(1)) as f32;
    let by = (p.i("Size Y").max(1)) as f32;
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let sx = (((x as f32 + 0.5) / bx).round() as i32 * bx as i32).clamp(0, w as i32 - 1);
            let sy = (((y as f32 + 0.5) / by).round() as i32 * by as i32).clamp(0, h as i32 - 1);
            out[y * w + x] = src[sy as usize * w + sx as usize];
        }
    }
    out
}

/// Сэмпл с клампом по краям (текстура в шейдере — repeat_disable/clamp).
#[inline]
fn at(src: &[Color32], w: i32, h: i32, x: i32, y: i32) -> [f32; 4] {
    rgba(src[y.clamp(0, h - 1) as usize * w as usize + x.clamp(0, w - 1) as usize])
}

/// Накопление цветов с весами: rgb усредняется с весом альфы,
/// альфа — обычным взвешенным средним (как в шейдерах автора).
struct Acc { r: f64, g: f64, b: f64, a: f64, w: f64 }

impl Acc {
    fn new() -> Self { Self { r: 0.0, g: 0.0, b: 0.0, a: 0.0, w: 0.0 } }
    #[inline]
    fn add(&mut self, c: [f32; 4], w: f64) {
        self.r += c[0] as f64 * c[3] as f64 * w;
        self.g += c[1] as f64 * c[3] as f64 * w;
        self.b += c[2] as f64 * c[3] as f64 * w;
        self.a += c[3] as f64 * w;
        self.w += w;
    }
    #[inline]
    fn finish(&self) -> [f32; 4] {
        let rgb = if self.a > 0.0 {
            [(self.r / self.a) as f32, (self.g / self.a) as f32, (self.b / self.a) as f32]
        } else {
            [0.0, 0.0, 0.0]
        };
        let alpha = if self.w > 0.0 { (self.a / self.w) as f32 } else { 0.0 };
        [rgb[0], rgb[1], rgb[2], q(alpha)]
    }
}

// GaussianBlur.gdshaderinc — все четыре варианта из оригинала.
fn fx_gaussian_blur(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let kind = p.choice("Type");
    let amount = p.i("Amount").clamp(0, 32);
    let radius = p.f("Radius").max(0.0);
    let wi = w as i32;
    let hi = h as i32;
    let mut buf: Vec<Color32> = src.to_vec();

    // промежуточный проход для «multi-pass» вариантов
    let pass = |input: &[Color32], dx: bool, iterations: i32, sigma_scale: f32| -> Vec<Color32> {
        let mut out = input.to_vec();
        for y in 0..hi {
            for x in 0..wi {
                if !in_sel(sel, x, y) { continue; }
                let mut acc = Acc::new();
                for k in -iterations..=iterations {
                    let kf = k as f32;
                    let weight = (-(kf * kf) / (2.0 * sigma_scale.max(0.5) * sigma_scale.max(0.5))).exp();
                    let (sx, sy) = if dx { (x + k, y) } else { (x, y + k) };
                    acc.add(at(input, wi, hi, sx, sy), weight as f64);
                }
                out[y as usize * w + x as usize] = pack(acc.finish());
            }
        }
        out
    };

    match kind {
        // Xor's Gaussian Blur — спиральное семплирование
        0 => {
            const ITER: i32 = 16;
            const QUALITY: i32 = 4;
            for y in 0..hi {
                for x in 0..wi {
                    if !in_sel(sel, x, y) { continue; }
                    let mut acc = Acc::new();
                    acc.add(at(src, wi, hi, x, y), 1.0);
                    for d in 0..ITER {
                        let theta = std::f32::consts::TAU * d as f32 / ITER as f32;
                        let (cs, sn) = (theta.cos(), theta.sin());
                        for step in 1..=QUALITY {
                            let i_f = step as f32 / QUALITY as f32;
                            let rad = amount as f32 * i_f;
                            let weight = (-(i_f * i_f) * 0.5).exp();
                            let sx = (x as f32 + cs * rad).round() as i32;
                            let sy = (y as f32 + sn * rad).round() as i32;
                            acc.add(at(src, wi, hi, sx, sy), weight as f64);
                        }
                    }
                    buf[y as usize * w + x as usize] = pack(acc.finish());
                }
            }
        }
        // Monk's Multi-Pass Blur — разделяемый, с затуханием по итерациям
        1 => {
            let iter = amount.max(1);
            let horiz = pass(src, true, iter, (iter as f32 * 0.8).sqrt().max(1.0));
            buf = pass(&horiz, false, iter, (iter as f32 * 0.8).sqrt().max(1.0));
        }
        // NoDev's Single-Pass Blur — линейное семплирование в обе стороны
        2 => {
            let n = amount;
            let sigma = (amount as f32 * 0.7).max(1.0);
            for y in 0..hi {
                for x in 0..wi {
                    if !in_sel(sel, x, y) { continue; }
                    let mut acc = Acc::new();
                    for k in -n..=n {
                        let d = k as f32 / std::f32::consts::PI;
                        let a = (d.cos() * radius * k as f32).round() as i32;
                        let b = (d.sin() * radius * k as f32).round() as i32;
                        let weight = (-((k * k) as f32) / (2.0 * sigma * sigma)).exp();
                        acc.add(at(src, wi, hi, x + a, y + b), weight as f64);
                    }
                    buf[y as usize * w + x as usize] = pack(acc.finish());
                }
            }
        }
        // NoDev's Multi-Pass Blur
        _ => {
            let n = amount;
            let sigma = (amount as f32 * 0.7).max(1.0);
            let horiz = pass(src, true, n, sigma);
            buf = pass(&horiz, false, n, sigma);
        }
    }
    buf
}

// ── Transform ─────────────────────────────────────────

// OffsetPixels.gdshaderinc
fn fx_offset_scale(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let ox = p.i("Offset X") as f32;
    let oy = p.i("Offset Y") as f32;
    let scale = (p.f("Scale") / 100.0).max(0.01);
    let wrap = p.flag("Wrap Around");
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            // uv сдвигаем на offset, затем масштабируем от центра
            let u = (x as f32 + 0.5 - ox) / w as f32;
            let v = (y as f32 + 0.5 - oy) / h as f32;
            let mut zu = (u - 0.5) / scale + 0.5;
            let mut zv = (v - 0.5) / scale + 0.5;
            if wrap {
                zu = zu.rem_euclid(1.0);
                zv = zv.rem_euclid(1.0);
            }
            if !(0.0..=1.0).contains(&zu) || !(0.0..=1.0).contains(&zv) {
                out[y * w + x] = Color32::TRANSPARENT;
                continue;
            }
            let sx = ((zu * w as f32) as i32).clamp(0, w as i32 - 1);
            let sy = ((zv * h as f32) as i32).clamp(0, h as i32 - 1);
            out[y * w + x] = src[sy as usize * w + sx as usize];
        }
    }
    out
}

/// FlipImageDialog.gd
fn fx_mirror(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let hz = p.flag("Horizontal");
    let vt = p.flag("Vertical");
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let sx = if hz { w as i32 - 1 - x as i32 } else { x as i32 };
            let sy = if vt { h as i32 - 1 - y as i32 } else { y as i32 };
            out[y * w + x] = src[sy as usize * w + sx as usize];
        }
    }
    out
}

/// Rotation/NearestNeighbour.gdshader — поворот вокруг центра холста.
fn fx_rotate(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let angle = p.f("Angle");
    if angle == 0.0 { return src.to_vec(); }
    let rad = angle.to_radians();
    let (sin, cos) = (rad.sin(), rad.cos());
    let cx = (w as f32 - 1.0) * 0.5;
    let cy = (h as f32 - 1.0) * 0.5;
    let mut out = vec![Color32::TRANSPARENT; w * h];
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            // обратный поворот: destination -> source
            let sx = (cos * dx + sin * dy + cx).round() as i32;
            let sy = (-sin * dx + cos * dy + cy).round() as i32;
            if sx >= 0 && sy >= 0 && sx < w as i32 && sy < h as i32 {
                out[y * w + x] = src[sy as usize * w + sx as usize];
            }
        }
    }
    out
}

/// FlatToIsometric.gdshaderinc
fn fx_flat_to_isometric(
    src: &[Color32],
    w: usize,
    h: usize,
    p: &EffectParams,
    sel: Option<(i32, i32, i32, i32)>,
) -> Vec<Color32> {
    let origin = p.i("Origin") as f32;
    let step = p.i("Step");
    let deadzone = p.i("Deadzone") as f32;
    if step == 0 { return src.to_vec(); }
    let ratio = w as f32 / h as f32;
    let ox = origin / w as f32;
    let dz = deadzone / w as f32;
    let step = step as f32;
    let mut out = src.to_vec();
    for y in 0..h {
        for x in 0..w {
            if !in_sel(sel, x as i32, y as i32) { continue; }
            let u = (x as f32 + 0.5) / w as f32;
            let dist = (u - ox).abs();
            let shifted = (dist - dz).max(0.0);
            let v = (y as f32 + 0.5) / h as f32 + shifted / step * ratio;
            if !(0.0..=1.0).contains(&v) {
                out[y * w + x] = Color32::TRANSPARENT;
                continue;
            }
            let sx = ((u * w as f32) as usize).min(w - 1);
            let sy = ((v * h as f32) as usize).min(h - 1);
            out[y * w + x] = src[sy * w + sx];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effect_kinds_match_table_order() {
        for (i, d) in EFFECTS.iter().enumerate() {
            assert_eq!(d.kind as usize, i, "EFFECTS[{i}] = {:?} out of order", d.kind);
        }
        assert_eq!(EFFECTS.len(), 17, "все 17 эффектов меню Effects Pixelorama");
    }

    #[test]
    fn every_effect_has_reachable_params() {
        for (i, d) in EFFECTS.iter().enumerate() {
            let p = EffectParams::new(d.kind);
            for pd in d.params {
                match pd.ty {
                    PT::I(..) | PT::F(..) => { let _ = p.f(pd.name); }
                    PT::B => { let _ = p.flag(pd.name); }
                    PT::C => { let _ = p.color(pd.name); }
                    PT::E(_) => { let _ = p.choice(pd.name); }
                }
            }
            let _ = p.signature();
            let _ = i;
        }
    }

    #[test]
    fn all_effects_run_on_a_small_image() {
        let (w, h) = (8usize, 8usize);
        let mut px: Vec<Color32> = (0..w * h)
            .map(|i| Color32::from_rgba_unmultiplied((i * 7 % 256) as u8, 40, 200, 255))
            .collect();
        for d in EFFECTS {
            let p = EffectParams::new(d.kind);
            let mut copy = px.clone();
            apply(&mut copy, w, h, &p, Some((1, 1, 6, 6)));
            assert_eq!(copy.len(), w * h, "{:?} изменил размер буфера", d.name);
            px = copy;
        }
    }

    #[test]
    fn invert_honors_per_channel_flags() {
        let w = 4;
        let h = 4;
        let mut px = vec![Color32::from_rgba_unmultiplied(10, 20, 30, 255); w * h];
        let mut p = EffectParams::new(EffectKind::Invert);
        apply(&mut px, w, h, &p, None);
        assert_eq!(px[0].to_srgba_unmultiplied(), [245, 235, 225, 255]);
        // гасим R — он должен остаться 245 (инверсии не было)
        p.set_flag("Red", false);
        apply(&mut px, w, h, &p, None);
        assert_eq!(
            px[0].to_srgba_unmultiplied(),
            [245, 20, 30, 255],
            "Red=false не должен инвертировать R"
        );
    }

    #[test]
    fn selection_is_respected() {
        let w = 4;
        let h = 4;
        let mut px = vec![Color32::from_rgba_unmultiplied(0, 0, 0, 255); w * h];
        let p = EffectParams::new(EffectKind::Invert);
        apply(&mut px, w, h, &p, Some((0, 0, 1, 1)));
        assert_eq!(px[0].r(), 255, "внутри выделения эффект применился");
        assert_eq!(px[3].r(), 0, "вне выделения эффект не применился");
    }
}
