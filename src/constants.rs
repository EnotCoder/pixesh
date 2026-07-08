use eframe::egui::Color32;

// какие инструменты есть
#[derive(PartialEq, Clone, Copy)]
pub enum Tool { Brush, Eraser, Fill, Eyedropper, Select }

// ── цвета темы ──
pub const BG: Color32 = Color32::from_rgb(24, 24, 24);          // самый тёмный (фон холста)
pub const PANEL: Color32 = Color32::from_rgb(32, 32, 32);       // панели
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(44, 44, 44); // панели чуть светлее
pub const BORDER: Color32 = Color32::from_rgb(80, 80, 80);      // границы
pub const TEXT: Color32 = Color32::from_rgb(220, 220, 220);      // текст
pub const ACCENT: Color32 = Color32::from_rgb(200, 120, 60);     // акцент (выделение)
pub const HOVER: Color32 = Color32::from_rgb(60, 60, 60);       // наведение мыши

// ── размеры ──
pub const FONT_SZ: f32 = 20.0;    // размер шрифта в пикселях
pub const CHAR_W: f32 = 11.0;     // примерная ширина одного символа (для расчётов)
pub const ROW_H: f32 = 22.0;      // высота строки в пикселях
pub const PANEL_PAD: f32 = 6.0;   // отступ от краёв панели (слои, цвет)
