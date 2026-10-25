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

// ── палитра на клавишах ──
/// Готовые цвета, выбираемые клавишами 1–9. Индекс массива плюс один равен
/// цифре на клавиатуре.
///
/// Альфа везде 255 не случайно: пресет — это конкретный непрозрачный цвет.
/// Прозрачность регулируется отдельно (`,` и `.`), и клавиша пресета не должна
/// её сбрасывать молча.
///
/// Порядок: три канала RGB, затем их сочетания, затем нейтральные — так, чтобы
/// под рукой были и цвета, и инструменты обводки с заливкой.
pub const NUM_PALETTE: [[u8; 4]; 9] = [
    [255, 0, 0, 255],     // 1 красный
    [0, 255, 0, 255],     // 2 зелёный
    [0, 0, 255, 255],     // 3 синий
    [255, 255, 0, 255],   // 4 жёлтый
    [0, 255, 255, 255],   // 5 циан
    [255, 0, 255, 255],   // 6 пурпурный
    [255, 255, 255, 255], // 7 белый
    [128, 128, 128, 255], // 8 серый
    [0, 0, 0, 255],       // 9 чёрный
];

// ── размеры ──
pub const FONT_SZ: f32 = 20.0;    // размер шрифта в пикселях
pub const CHAR_W: f32 = 11.0;     // примерная ширина одного символа (для расчётов)
pub const ROW_H: f32 = 22.0;      // высота строки в пикселях
pub const PANEL_PAD: f32 = 12.0;   // отступ от краёв панели (слои, цвет)
pub const CHECKER_PX: f32 = 8.0;  // желаемый экранный размер клетки шахматки (прозрачный фон)

/// Максимальный размер стороны холста — тот же предел, что у DragValue
/// в диалогах изменения размера и масштаба.
pub const MAX_SIDE: i32 = 4096;

/// Возвращает пару (W, H), в которой изменённая сторона согласована с другой
/// по пропорциям исходного холста.
///
/// Опорой служит именно `orig_w`/`orig_h`, а не текущая пара `w`/`h`. Если
/// брать за опору вторую сторону, пропорции «поедут»: у холста 64x48 шаг W=100
/// даёт H=75, а следующий шаг H=76 вернул бы W=101 — и с каждым щелчком
/// расхождение накапливалось бы. Якорь не меняется, поэтому дрейфа нет.
pub fn linked_side(orig_w: i32, orig_h: i32, w: i32, h: i32, edited_w: bool) -> (i32, i32) {
    let clamp = |v: i32| v.clamp(1, MAX_SIDE);
    if edited_w {
        if orig_w <= 0 { return (clamp(w), clamp(h)); }
        (clamp(w), clamp(((w as f64 * orig_h as f64 / orig_w as f64).round()) as i32))
    } else {
        if orig_h <= 0 { return (clamp(w), clamp(h)); }
        (clamp(((h as f64 * orig_w as f64 / orig_h as f64).round()) as i32), clamp(h))
    }
}

#[cfg(test)]
mod linked_tests {
    use super::*;

    #[test]
    fn width_edit_keeps_aspect() {
        // 100x50 -> ширина 200, высота обязана стать 100
        assert_eq!(linked_side(100, 50, 200, 50, true), (200, 100));
    }

    #[test]
    fn height_edit_keeps_aspect() {
        // 100x50 — пропорция 2, поэтому H=200 это четырехкратное увеличение
        // и ширина обязана стать 400, а не 200
        assert_eq!(linked_side(100, 50, 100, 200, false), (400, 200));
        // а вот H=100 — ровно вдвое
        assert_eq!(linked_side(100, 50, 100, 100, false), (200, 100));
    }

    #[test]
    fn square_canvas_is_unchanged() {
        assert_eq!(linked_side(64, 64, 128, 64, true), (128, 128));
        assert_eq!(linked_side(64, 64, 64, 32, false), (32, 32));
    }

    /// Главное свойство: последовательность правок не уходит от пропорций.
    /// Раньше считая от показанных значений, второй шаг дал бы 101 вместо 100.
    #[test]
    fn successive_edits_do_not_drift() {
        let (ow, oh) = (64, 48);
        let (w, h) = linked_side(ow, oh, 100, 48, true);
        assert_eq!((w, h), (100, 75));
        let (w, h) = linked_side(ow, oh, w, 76, false);
        assert_eq!((w, h), (101, 76), "ширина считается от якоря, а не от 100");
        let (w, h) = linked_side(ow, oh, 202, h, true);
        assert_eq!((w, h), (202, 152));
    }

    /// Пропорция итоговой пары совпадает с исходной с точностью до округления.
    /// Случаи, где результат упрётся в предел 4096, пропускаем: там пропорция
    /// ломается намеренно, это проверяет отдельный тест про кламп.
    #[test]
    fn result_stays_within_one_pixel_of_the_original_ratio() {
        for (ow, oh) in [(64, 48), (100, 50), (37, 91), (16, 16), (5, 3)] {
            for target in [1, 7, 33, 64, 200, 4096] {
                let expect = target as f64 * oh as f64 / ow as f64;
                if !(1.0..=MAX_SIDE as f64).contains(&expect.round()) { continue; }
                let (w, h) = linked_side(ow, oh, target, oh, true);
                assert_eq!(w, target, "{ow}x{oh}, ширина не должна меняться");
                assert!(
                    (h as f64 - expect).abs() <= 0.5,
                    "{ow}x{oh} -> {w}x{h}, ожидалось ~{expect}"
                );
            }
        }
    }

    #[test]
    fn values_are_clamped_to_the_dialog_range() {
        assert_eq!(linked_side(1, 4096, 4096, 4096, true).1, 4096);
        // крошечная высота и огромная ширина не должны обнулить высоту
        let (_, h) = linked_side(4096, 1, 4096, 1, true);
        assert_eq!(h, 1);
        let (w, _) = linked_side(1, 4096, 1, 4096, false);
        assert_eq!(w, 1);
        // и наоборот: узкий холст при огромной ширине упрётся в предел
        assert_eq!(linked_side(37, 91, 4096, 91, true).1, MAX_SIDE);
    }

    /// Вырожденный якорь не должен ронять расчёт (деление на ноль).
    #[test]
    fn degenerate_anchor_passes_values_through() {
        assert_eq!(linked_side(0, 10, 5, 10, true), (5, 10));
        assert_eq!(linked_side(10, 0, 10, 5, false), (10, 5));
    }
}
