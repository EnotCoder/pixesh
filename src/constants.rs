use eframe::egui::{Color32, Id};

// какие инструменты есть
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Tool { Brush, Eraser, Fill, Eyedropper, Select, Move, Text, Transform, Effects }

// фон при экспорте PNG
#[derive(PartialEq, Clone, Copy)]
pub enum ExportBg { Transparent, White, Black, Checker }

// форма (маска) кисти
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum BrushShape { Round, Square }

// ── раскладка панелей ──

/// Какой край окна занимает панель. Панели можно перетаскивать между краями,
/// но не на любой: полосу кадров бессмысленно прижимать к боку окна.
#[derive(PartialEq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Edge { Top, Bottom, Left, Right }

impl Edge {
    pub fn is_vertical(self) -> bool {
        matches!(self, Edge::Left | Edge::Right)
    }

    /// Порядок групп при отрисовке. Верхние и нижние панели должны идти до
    /// боковых: egui вырезает панели из остатка прямоугольника по порядку,
    /// поэтому верхняя, нарисованная после боковой, уже не растянулась бы на
    /// всю ширину окна.
    pub fn draw_group(self) -> usize {
        match self {
            Edge::Top => 0,
            Edge::Bottom => 1,
            Edge::Left => 2,
            Edge::Right => 3,
        }
    }

    /// Края, на которые панель разрешено перетаскивать. Тулбар, статус-бар и
    /// таймлайн — горизонтальные полосы, панель слоёв — вертикальный список.
    pub fn allowed_for(self) -> &'static [Edge] {
        const H: &[Edge] = &[Edge::Top, Edge::Bottom];
        const V: &[Edge] = &[Edge::Left, Edge::Right];
        if self.is_vertical() { V } else { H }
    }
}

/// Какая из перетаскиваемых панелей. Порядок вариантов задаёт канонический
/// порядок отрисовки внутри одного края — он не меняется от того, куда панель
/// перетащили, иначе панели на одном краю прыгали бы местами при каждом
/// переносе.
#[derive(PartialEq, Eq, Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum PanelId { Toolbar, Status, Timeline, Layers }

/// Порядок панели внутри одного края. Нужен sort_by_key, который сортирует
/// по целому, а не по перечислению (derive Ord тащит за собой лишнее).
pub(crate) fn p_order(p: PanelId) -> usize {
    match p {
        PanelId::Toolbar => 0,
        PanelId::Status => 1,
        PanelId::Timeline => 2,
        PanelId::Layers => 3,
    }
}

impl PanelId {
    /// Стартовый край. Панели на одном краю идут именно в этом порядке:
    /// первая отрисованная оказывается у самого края окна.
    pub fn default_edge(self) -> Edge {
        match self {
            PanelId::Toolbar => Edge::Top,
            PanelId::Status => Edge::Bottom,
            PanelId::Timeline => Edge::Bottom,
            PanelId::Layers => Edge::Right,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            PanelId::Toolbar => "Toolbar",
            PanelId::Status => "Status",
            PanelId::Timeline => "Timeline",
            PanelId::Layers => "Layers",
        }
    }

    /// Собственный край панели, по которому судим, горизонтальная она или
    /// вертикальная. Одинаков для всех краёв, допустимых для панели.
    pub fn shape_edge(self) -> Edge {
        self.default_edge()
    }

    /// Стабильный Id для egui. Край входит в Id намеренно: PanelState лежит в
    /// egui::Memory по Id и для боковой панели читается как rect.width(), а для
    /// верхней/нижней как rect.height(). Без края в Id панель, переехавшая с
    /// низа на бок, прочитала бы старые 1400 px как ширину и раздулась до
    /// max_width. С краем в Id каждая ориентация помнит свой размер отдельно.
    pub fn ui_id(self, edge: Edge) -> Id {
        Id::new((self.name(), format!("{:?}", edge)))
    }
}

// ── цвета темы ──
pub const BG: Color32 = Color32::from_rgb(24, 24, 24);          // самый тёмный (фон холста)
pub const PANEL: Color32 = Color32::from_rgb(32, 32, 32);       // панели
pub const PANEL_LIGHT: Color32 = Color32::from_rgb(44, 44, 44); // панели чуть светлее
pub const BORDER: Color32 = Color32::from_rgb(80, 80, 80);      // границы
pub const TEXT: Color32 = Color32::from_rgb(220, 220, 220);      // текст
pub const ACCENT: Color32 = Color32::from_rgb(200, 120, 60);     // акцент (выделение)
pub const HOVER: Color32 = Color32::from_rgb(60, 60, 60);       // наведение мыши
pub const DIM: Color32 = Color32::from_rgb(120, 120, 120);      // приглушённый текст

// ── размеры ──
pub const FONT_SZ: f32 = 20.0;    // размер шрифта в пикселях
pub const CHAR_W: f32 = 11.0;     // примерная ширина одного символа (для расчётов)
pub const ROW_H: f32 = 22.0;      // высота строки в пикселях
pub const PANEL_PAD: f32 = 12.0;   // отступ от краёв панели (слои, цвет)
pub const CHECKER_PX: f32 = 8.0;  // желаемый экранный размер клетки шахматки (прозрачный фон)
