use eframe::egui::Color32;

#[derive(PartialEq, Clone, Copy)]
pub enum Tool { Brush, Eraser, Fill, Eyedropper }

pub const BG: Color32 = Color32::from_rgb(24, 24, 32);
pub const PANEL: Color32 = Color32::from_rgb(32, 32, 40);
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(44, 44, 54);
pub const BORDER: Color32 = Color32::from_rgb(80, 80, 90);
pub const TEXT: Color32 = Color32::from_rgb(220, 220, 230);
pub const ACCENT: Color32 = Color32::from_rgb(200, 120, 60);
pub const HOVER: Color32 = Color32::from_rgb(60, 60, 72);
pub const FONT_SZ: f32 = 20.0;
pub const CHAR_W: f32 = 11.0;
pub const ROW_H: f32 = 22.0;
