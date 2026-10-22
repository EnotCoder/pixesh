pub mod anim;
pub mod canvas;
pub mod config;
pub mod effects;
pub mod history;
pub mod input;
pub mod io;
pub mod panel_canvas;
pub mod panel_drag;
pub mod panel_dialogs;
pub mod panel_layers;
    pub mod panel_status;
    pub mod panel_timeline;
    pub mod panel_toolbar;
pub mod text;
pub mod tools;

use std::sync::Arc;

use eframe::egui::{self, Color32, Pos2, Rect, Stroke, Vec2};

use crate::constants::*;
use crate::ui::btn_min_w;

// ── Layer / Snapshot ─────────────────────────────────
pub(crate) struct Layer {
    pub(crate) name: String,
    // one pixel buffer ("cel") per animation frame
    pub(crate) cels: Vec<Arc<Vec<Color32>>>,
    pub(crate) visible: bool,
}

pub(crate) struct SnapshotLayer {
    pub(crate) name: String,
    pub(crate) cels: Vec<Arc<Vec<Color32>>>,
    pub(crate) visible: bool,
}

pub(crate) struct Snapshot {
    pub(crate) layers: Vec<SnapshotLayer>,
    pub(crate) active: usize,
    pub(crate) frames: usize,
    pub(crate) active_frame: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) sel: Option<(i32, i32, i32, i32)>,
    pub(crate) sel_buffer: Option<Vec<Color32>>,
    pub(crate) sel_buf_w: usize,
    pub(crate) sel_buf_h: usize,
    pub(crate) pasting: bool,
    pub(crate) clipboard: Option<Vec<Color32>>,
    pub(crate) clip_w: usize,
    pub(crate) clip_h: usize,
}

// ── Document: per-image state ────────────────────────
pub(crate) struct Document {
    pub(crate) name: String,
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,

    pub(crate) frames: usize,
    pub(crate) active_frame: usize,
    pub(crate) playing: bool,
    pub(crate) fps: f32,
    pub(crate) last_play: f64,

    pub(crate) undo_stack: Vec<Snapshot>,
    pub(crate) redo_stack: Vec<Snapshot>,

    pub(crate) canvas_dirty: bool,
    pub(crate) display_cell: i32,
    pub(crate) display_buf: Vec<Color32>,
    pub(crate) tex: Option<egui::TextureHandle>,

    pub(crate) sel: Option<(i32, i32, i32, i32)>,
    pub(crate) sel_start: Option<(i32, i32)>,
    pub(crate) sel_end: Option<(i32, i32)>,
    pub(crate) sel_move_origin: Option<(i32, i32)>,
    pub(crate) sel_move_current: Option<(i32, i32)>,
    pub(crate) sel_buffer: Option<Vec<Color32>>,
    pub(crate) sel_buf_w: usize,
    pub(crate) sel_buf_h: usize,
    pub(crate) sel_tex: Option<egui::TextureHandle>,

    pub(crate) canvas_move_origin: Option<(i32, i32)>,
    pub(crate) canvas_move_current: Option<(i32, i32)>,

    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) grid: bool,

    pub(crate) last_px_primary: Option<(i32, i32)>,
    pub(crate) last_px_secondary: Option<(i32, i32)>,
    pub(crate) mid_pan_pos: Option<Pos2>,

    pub(crate) unsaved: bool,
    pub(crate) export_path: String,
    pub(crate) export_name: String,
    pub(crate) needs_zoom_fit: bool,

    pub(crate) clipboard: Option<Vec<Color32>>,
    pub(crate) clip_w: usize,
    pub(crate) clip_h: usize,
    pub(crate) pasting: bool,

    pub(crate) transforming: bool,
    pub(crate) transform_orig_rect: Option<(i32, i32, i32, i32)>,
    pub(crate) transform_corner: Option<usize>,
}

/// Сторона холста по умолчанию. Совпадает с размером `img/pixel_logo.png`,
/// чтобы маскот лёг на холст один в один, без всякого ужатия.
const MASCOT_SIDE: usize = 32;

/// Пиксели совы из `img/pixel_logo.png`, приведённые к размеру стартового холста.
///
/// Источник ровно такого же размера, как холст, так что `resize` здесь просто
/// копирует. Фильтр оставлен nearest неспроста: если размеры когда-нибудь
/// разойдутся, nearest сохранит пиксель-арт стиль совы, а сглаживание на
/// заметном уменьшении превратило бы маскот в пёстрое пятно.
///
/// `None` — если картинка не декодируется: тогда холст просто останется
/// пустым, приложение при этом не упадёт.
fn logo_pixels(w: usize, h: usize) -> Option<Vec<Color32>> {
    let img = image::load_from_memory(include_bytes!("../../img/pixel_logo.png"))
        .ok()?
        .to_rgba8();
    let small = image::imageops::resize(
        &img,
        w as u32,
        h as u32,
        image::imageops::FilterType::Nearest,
    );
    Some(
        small
            .pixels()
            .map(|p| Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]))
            .collect(),
    )
}

impl Document {
    pub(crate) fn new_sized(name: &str, w: usize, h: usize) -> Self {
        let mut doc = Document::new(name);
        doc.width = w;
        doc.height = h;
        doc.layers[0].cels = vec![Arc::new(vec![Color32::TRANSPARENT; w * h])];
        doc
    }

    /// Документ по умолчанию: чистый холст 32×32, залитый совой из
    /// `img/pixel_logo.png`. Нужен, чтобы при запуске и при закрытии последней
    /// вкладки холст не выглядел пустым серым квадратом.
    pub(crate) fn new_logo(name: &str) -> Self {
        let mut doc = Document::new_sized(name, MASCOT_SIDE, MASCOT_SIDE);
        // Длина проверяется на месте: если размеры когда-нибудь разойдутся
        // с размером картинки, документ остаётся пустым, а не падает по индексу.
        if let Some(px) = logo_pixels(doc.width, doc.height)
            .filter(|px| px.len() == doc.width * doc.height)
        {
            doc.layers[0].cels[0] = Arc::new(px);
        }
        doc
    }

    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            layers: vec![Layer {
                name: "Background".into(),
                cels: vec![Arc::new(vec![Color32::TRANSPARENT; 16 * 16])],
                visible: true,
            }],
            active_layer: 0,
            width: 16,
            height: 16,
            frames: 1,
            active_frame: 0,
            playing: false,
            fps: 12.0,
            last_play: 0.0,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            canvas_dirty: true,
            display_cell: 0,
            display_buf: Vec::new(),
            tex: None,
            sel: None,
            sel_start: None,
            sel_end: None,
            sel_move_origin: None,
            sel_move_current: None,
            sel_buffer: None,
            sel_buf_w: 0,
            sel_buf_h: 0,
            sel_tex: None,
            canvas_move_origin: None,
            canvas_move_current: None,
            zoom: 46.0,
            pan: Vec2::ZERO,
            grid: false,
            last_px_primary: None,
            last_px_secondary: None,
            mid_pan_pos: None,
            unsaved: false,
            export_path: String::new(),
            export_name: "pixesh.png".into(),
            needs_zoom_fit: true,
            clipboard: None,
            clip_w: 0,
            clip_h: 0,
            pasting: false,
            transforming: false,
            transform_orig_rect: None,
            transform_corner: None,
        }
    }
}

// ── App ──────────────────────────────────────────────
pub struct PixeshApp {
    pub(crate) docs: Vec<Document>,
    pub(crate) active_tab: usize,

    pub(crate) color: Color32,
    pub(crate) hsv_h: f32,
    pub(crate) hsv_s: f32,
    pub(crate) hsv_v: f32,
    pub(crate) rgb_r: f32,
    pub(crate) rgb_g: f32,
    pub(crate) rgb_b: f32,
    pub(crate) rgb_a: f32,
    pub(crate) brush: f32,
    pub(crate) brush_shape: BrushShape,
    pub(crate) tool: Tool,
    pub(crate) tool_saved: Option<Tool>,
    pub(crate) tool_saved_shift: Option<Tool>,

    pub(crate) brush_tex: Option<egui::TextureHandle>,
    pub(crate) eraser_tex: Option<egui::TextureHandle>,
    pub(crate) fill_tex: Option<egui::TextureHandle>,
    pub(crate) drop_tex: Option<egui::TextureHandle>,
    pub(crate) clear_tex: Option<egui::TextureHandle>,
    pub(crate) logo_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex: Option<egui::TextureHandle>,
    pub(crate) sv_tex_h: f32,
    pub(crate) select_tex: Option<egui::TextureHandle>,
    pub(crate) move_tex: Option<egui::TextureHandle>,
    pub(crate) h_tex: Option<egui::TextureHandle>,
    pub(crate) mirror_h_tex: Option<egui::TextureHandle>,
    pub(crate) mirror_v_tex: Option<egui::TextureHandle>,
    pub(crate) plus_layer_tex: Option<egui::TextureHandle>,
    pub(crate) minus_layer_tex: Option<egui::TextureHandle>,
    pub(crate) clone_layer_tex: Option<egui::TextureHandle>,
    pub(crate) set_all_tex: Option<egui::TextureHandle>,
    pub(crate) tab_close_tex: Option<egui::TextureHandle>,
    pub(crate) tab_plus_tex: Option<egui::TextureHandle>,

    pub(crate) show_resize: bool,
    pub(crate) resize_w: f32,
    pub(crate) resize_h: f32,
    pub(crate) show_scale: bool,
    pub(crate) scale_w: f32,
    pub(crate) scale_h: f32,
    /// Держать пропорции: ввод W тянет за собой H и наоборот.
    /// По умолчанию выключено, иначе тот, кто правит одну сторону, получал бы
    /// тихо подменённую вторую.
    pub(crate) scale_lock_ratio: bool,
    pub(crate) resize_lock_ratio: bool,
    pub(crate) show_export: bool,
    pub(crate) export_scale: i32,
    pub(crate) export_bg: ExportBg,
    pub(crate) export_layers: bool,
    pub(crate) export_sheet: bool,
    pub(crate) show_panels: bool,
    pub(crate) show_settings: bool,
    pub(crate) show_top_panel: bool,
    pub(crate) show_right_panel: bool,
    pub(crate) show_status_bar: bool,
    pub(crate) show_timeline: bool,

    /// Край окна для каждой перетаскиваемой панели
    pub(crate) toolbar_edge: Edge,
    pub(crate) layers_edge: Edge,
    pub(crate) status_edge: Edge,
    pub(crate) timeline_edge: Edge,

    // ── перетаскивание панелей между краями ──
    /// Край, на который в данный момент наведён курсор при переносе
    pub(crate) drop_edge: Option<Edge>,
    /// Панель, которую сейчас тащим
    pub(crate) dragging_panel: Option<PanelId>,
    /// Откуда начали тащить — отменяем перенос, если бросили туда же
    pub(crate) drag_from_edge: Edge,
    pub(crate) show_quit_dialog: bool,
    pub(crate) show_welcome: bool,
    pub(crate) welcome_show_again: bool,

    pub(crate) arrow_speed: f32,
    pub(crate) zoom_speed: f32,

    pub(crate) color_history: Vec<Color32>,
    pub(crate) renaming_layer: Option<usize>,
    pub(crate) rename_buf: String,

    pub(crate) show_text: bool,
    pub(crate) text_cursor: Option<(i32, i32)>,
    pub(crate) text_buffer: String,
    pub(crate) text_scale: i32,

    /// Временная подсказка в статус-баре (буфер обмена, выделение и т.п.)
    pub(crate) status_hint: String,
    pub(crate) status_hint_until: f64,
    /// Время последнего кадра — чтобы hint() гасил подсказку без доступа к ctx
    pub(crate) last_time: f64,

    // ── панель эффектов (порты из Pixelorama) ──
    pub(crate) show_effects: bool,
    pub(crate) effect: effects::EffectParams,
    /// на сколько слоёв/кадров применять эффект
    pub(crate) effect_all_layers: bool,
    pub(crate) effect_all_frames: bool,
    /// кэш превью: сигнатура параметров + текстура
    pub(crate) effect_prev_sig: String,
    pub(crate) effect_prev_tex: Option<egui::TextureHandle>,
    pub(crate) effect_prev_px: Vec<Color32>,
    pub(crate) effect_prev_size: (usize, usize),
    pub(crate) effect_prev_time: f64,

    pub(crate) cursor_px: Option<(i32, i32)>,
    pub(crate) close_handled: bool,

    pub(crate) logo_easter_egg: f32,
}

impl PixeshApp {
    pub fn new() -> Self {
        // дефолты настроек живут в одном месте — в Prefs, иначе «что было при
        // первом запуске» и «что лежит в файле» разъезжаются
        let prefs = Self::initial_prefs();
        Self {
            docs: vec![Document::new_logo("Untitled")],
            active_tab: 0,
            color: Color32::BLACK,
            hsv_h: 0.0, hsv_s: 0.0, hsv_v: 0.0,
            rgb_r: 0.0, rgb_g: 0.0, rgb_b: 0.0, rgb_a: 255.0,
            brush: prefs.brush,
            brush_shape: prefs.brush_shape,
            tool: prefs.tool,
            tool_saved: None,
            tool_saved_shift: None,
            brush_tex: None, eraser_tex: None, fill_tex: None,
            drop_tex: None, clear_tex: None, logo_tex: None,
            sv_tex: None, sv_tex_h: -1.0,
            select_tex: None, move_tex: None, h_tex: None,
            mirror_h_tex: None, mirror_v_tex: None,
            plus_layer_tex: None, minus_layer_tex: None,
            clone_layer_tex: None, set_all_tex: None,
            tab_close_tex: None, tab_plus_tex: None,
            show_resize: false, resize_w: 64.0, resize_h: 64.0,
            show_scale: false, scale_w: 64.0, scale_h: 64.0,
            scale_lock_ratio: false, resize_lock_ratio: false,
            show_export: false, export_scale: 1, export_bg: ExportBg::Transparent,
            export_layers: false, export_sheet: false,
            show_panels: false, show_settings: false,
            show_top_panel: prefs.show_top_panel,
            show_right_panel: prefs.show_right_panel,
            show_status_bar: prefs.show_status_bar,
            show_timeline: prefs.show_timeline,
            toolbar_edge: prefs.toolbar_edge,
            layers_edge: prefs.layers_edge,
            status_edge: prefs.status_edge,
            timeline_edge: prefs.timeline_edge,
            drop_edge: None,
            dragging_panel: None,
            drag_from_edge: Edge::Top,
            show_quit_dialog: false,
            welcome_show_again: prefs.welcome_show_again,
            show_welcome: prefs.welcome_show_again,
            arrow_speed: prefs.arrow_speed,
            zoom_speed: prefs.zoom_speed,
            color_history: Vec::new(),
            renaming_layer: None, rename_buf: String::new(),
            status_hint: String::new(),
            status_hint_until: 0.0,
            last_time: 0.0,
            show_text: false,
            text_cursor: None,
            text_buffer: String::new(),
            text_scale: 2,
            show_effects: false,
            effect: effects::EffectParams::new(effects::EffectKind::OffsetScale),
            effect_all_layers: false,
            effect_all_frames: false,
            effect_prev_sig: String::new(),
            effect_prev_tex: None,
            effect_prev_px: Vec::new(),
            effect_prev_size: (0, 0),
            effect_prev_time: 0.0,
            cursor_px: None,
            close_handled: false,
            logo_easter_egg: 0.0,
        }
    }
}

impl PixeshApp {
    /// Настройки по умолчанию. Без фичи persistence файла нет, поэтому
    /// Preferences просто превращается в набор исходных значений полей.
    fn initial_prefs() -> config::Prefs {
        config::Prefs::default()
    }

    /// Край, на котором сейчас панель.
    pub(crate) fn edge_of(&self, p: PanelId) -> Edge {
        match p {
            PanelId::Toolbar => self.toolbar_edge,
            PanelId::Layers => self.layers_edge,
            PanelId::Status => self.status_edge,
            PanelId::Timeline => self.timeline_edge,
        }
    }

    pub(crate) fn set_edge(&mut self, p: PanelId, e: Edge) {
        let e = config::clamp_edge(e, p.default_edge());
        match p {
            PanelId::Toolbar => self.toolbar_edge = e,
            PanelId::Layers => self.layers_edge = e,
            PanelId::Status => self.status_edge = e,
            PanelId::Timeline => self.timeline_edge = e,
        }
    }

    pub(crate) fn panel_visible(&self, p: PanelId) -> bool {
        match p {
            PanelId::Toolbar => self.show_top_panel,
            PanelId::Layers => self.show_right_panel,
            PanelId::Status => self.show_status_bar,
            PanelId::Timeline => self.show_timeline,
        }
    }

    /// Порядок отрисовки панелей: по группам краёв, а внутри края — по
    /// каноническому порядку. Возвращает только видимые панели.
    ///
    /// Вынесено отдельно и без egui, чтобы поведение можно было проверить
    /// тестом: главное свойство здесь — что все горизонтальные полосы идут
    /// раньше боковых. Иначе верхняя панель, нарисованная после боковой,
    /// получила бы не всю ширину окна: egui вырезает панели из остатка
    /// прямоугольника строго в порядке вызовов.
    pub(crate) fn draw_order(&self) -> Vec<PanelId> {
        let mut out: Vec<PanelId> = [PanelId::Toolbar, PanelId::Status, PanelId::Timeline, PanelId::Layers]
            .into_iter()
            .filter(|p| self.panel_visible(*p))
            .collect();
        out.sort_by_key(|p| (self.edge_of(*p).draw_group(), p_order(*p)));
        out
    }

    /// Ставит текущий цвет сразу во все представления: Color32, RGBA-поля
    /// и HSV. Держать их по отдельности нельзя — пикер и слайдеры пишут в
    /// одни и те же поля, и рассинхрон виден пользователю.
    pub(crate) fn set_color(&mut self, c: Color32) {
        self.color = c;
        self.rgb_r = c.r() as f32;
        self.rgb_g = c.g() as f32;
        self.rgb_b = c.b() as f32;
        self.rgb_a = c.a() as f32;
        let (h, s, v) = crate::color::rgb_to_hsv(c.r(), c.g(), c.b());
        self.hsv_h = h;
        self.hsv_s = s;
        self.hsv_v = v;
    }

    /// Подправить HSV и пересчитать из него RGB. `color`, `rgb_*` и `hsv_*`
    /// обязаны оставаться согласованными — иначе пикер и слайдеры разъезжаются.
    ///
    /// Оттенок заворачивается по кругу, а не упирается в clamp: диапазон
    /// 0–360° — это замкнутое кольцо, и на границе вращение не должно
    /// останавливаться.
    pub(crate) fn bump_hsv(&mut self, dh: f32, ds: f32, dv: f32) {
        self.hsv_h = (self.hsv_h + dh).rem_euclid(360.0);
        self.hsv_s = (self.hsv_s + ds).clamp(0.0, 255.0);
        self.hsv_v = (self.hsv_v + dv).clamp(0.0, 255.0);
        let (r, g, b) = crate::color::hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
        self.rgb_r = r as f32;
        self.rgb_g = g as f32;
        self.rgb_b = b as f32;
        self.color = Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
    }

    /// Прозрачность правится отдельным методом: она не входит в HSV, поэтому
    /// пересчёт оттенка ей не нужен, а цвет менять нельзя.
    pub(crate) fn bump_alpha(&mut self, d: f32) {
        self.rgb_a = (self.rgb_a + d).clamp(0.0, 255.0);
        self.color = Color32::from_rgba_unmultiplied(
            self.rgb_r as u8,
            self.rgb_g as u8,
            self.rgb_b as u8,
            self.rgb_a as u8,
        );
    }

    pub(crate) fn dialog_open(&self) -> bool {
        self.show_resize || self.show_export
            || self.show_panels || self.show_settings || self.show_scale
            || self.show_quit_dialog || self.show_text || self.show_welcome
            || self.show_effects
    }

    /// Показать сообщение в статус-баре на пару секунд.
    pub(crate) fn hint(&mut self, msg: impl Into<String>) {
        self.status_hint = msg.into();
        self.status_hint_until = self.last_time + 2.5;
    }

    /// Снимок сохраняемого состояния. Список намеренно короткий: сюда попадает
    /// только то, что имеет смысл помнить между запусками. Флаги диалогов
    /// (show_resize/show_export/show_settings/show_panels/show_text/
    /// show_effects/show_quit_dialog) сюда НЕ идут — иначе при запуске вылезет
    /// диалог, и «закрытое окно» будет открываться само.
    #[cfg_attr(not(feature = "persistence"), allow(dead_code))]
    pub(crate) fn prefs(&self) -> config::Prefs {
        config::Prefs {
            show_top_panel: self.show_top_panel,
            show_right_panel: self.show_right_panel,
            show_status_bar: self.show_status_bar,
            show_timeline: self.show_timeline,
            toolbar_edge: self.toolbar_edge,
            layers_edge: self.layers_edge,
            status_edge: self.status_edge,
            timeline_edge: self.timeline_edge,
            welcome_show_again: self.welcome_show_again,
            arrow_speed: self.arrow_speed,
            zoom_speed: self.zoom_speed,
            brush: self.brush,
            brush_shape: self.brush_shape,
            tool: self.tool,
            rgba: [
                self.rgb_r.clamp(0.0, 255.0) as u8,
                self.rgb_g.clamp(0.0, 255.0) as u8,
                self.rgb_b.clamp(0.0, 255.0) as u8,
                self.rgb_a.clamp(0.0, 255.0) as u8,
            ],
            color_history: self
                .color_history
                .iter()
                .map(|c| [c.r(), c.g(), c.b(), c.a()])
                .collect(),
        }
    }

    /// Восстанавливает состояние из файла. Метода `App::restore` в eframe 0.31
    /// нет — читать хранилище приходится здесь, в замыкании создания.
    pub(crate) fn load_prefs(&mut self, storage: Option<&dyn eframe::Storage>) {
        let prefs = config::Prefs::load(storage);

        self.show_top_panel = prefs.show_top_panel;
        self.show_right_panel = prefs.show_right_panel;
        self.show_status_bar = prefs.show_status_bar;
        self.show_timeline = prefs.show_timeline;
        self.toolbar_edge = config::clamp_edge(prefs.toolbar_edge, Edge::Top);
        self.layers_edge = config::clamp_edge(prefs.layers_edge, Edge::Right);
        self.status_edge = config::clamp_edge(prefs.status_edge, Edge::Bottom);
        self.timeline_edge = config::clamp_edge(prefs.timeline_edge, Edge::Bottom);
        self.welcome_show_again = prefs.welcome_show_again;
        self.arrow_speed = prefs.arrow_speed;
        self.zoom_speed = prefs.zoom_speed;
        self.brush = prefs.brush;
        self.brush_shape = prefs.brush_shape;
        self.tool = prefs.tool;
        self.set_color(Color32::from_rgba_unmultiplied(
            prefs.rgba[0], prefs.rgba[1], prefs.rgba[2], prefs.rgba[3],
        ));
        self.color_history = prefs
            .color_history
            .iter()
            .map(|c| Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
            .collect();

        // dialogs must all start closed
        self.show_resize = false;
        self.show_scale = false;
        self.show_export = false;
        self.show_panels = false;
        self.show_settings = false;
        self.show_quit_dialog = false;
        self.show_effects = false;
        self.show_text = false;
        self.show_welcome = self.welcome_show_again;
        if self.tool == Tool::Effects { self.tool = Tool::Brush; }
        self.renaming_layer = None;
    }

    /// Открывает картинку в НОВОЙ вкладке. Общая точка для Ctrl+L и для
    /// перетаскивания файла в окно, поэтому загрузка не зависит от наличия rfd.
    /// Возвращает имя файла, если открыть удалось.
    pub(crate) fn open_path(&mut self, path: &std::path::Path) -> Option<String> {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());

        let mut doc = Document::new(&name);
        match doc.load_png(&path_str) {
            Ok(()) => {
                self.docs.push(doc);
                self.active_tab = self.docs.len() - 1;
                Some(name)
            }
            Err(e) => {
                // вкладку не создаём: пустая вкладка с именем файла только путает
                self.hint(format!("{}: {}", name, e));
                None
            }
        }
    }

    /// Применить выбранный эффект к слоям документа.
    /// Один вызов = одна запись в undo, и только если что-то реально изменилось.
    pub(crate) fn apply_effect_to(&mut self, tab: usize) {
        let doc = &mut self.docs[tab];
        let sel = doc.sel;
        let (w, h) = (doc.width, doc.height);
        let frames: Vec<usize> = if self.effect_all_frames {
            (0..doc.frames).collect()
        } else {
            vec![doc.active_frame]
        };
        let layers: Vec<usize> = if self.effect_all_layers {
            (0..doc.layers.len()).collect()
        } else {
            vec![doc.active_layer]
        };

        // сначала считаем результат в стороне — если ничего не изменилось,
        // не засоряем ни состояние документа, ни стек undo
        let mut results: Vec<(usize, usize, Vec<Color32>)> = Vec::new();
        for &li in &layers {
            if li >= doc.layers.len() { continue; }
            for &f in &frames {
                let Some(cel) = doc.layers[li].cels.get(f) else { continue };
                if cel.len() != w * h { continue; }
                let mut buf = (**cel).clone();
                effects::apply(&mut buf, w, h, &self.effect, sel);
                if buf != **cel {
                    results.push((li, f, buf));
                }
            }
        }
        if results.is_empty() { return; }

        doc.push_undo();
        for (li, f, buf) in results {
            doc.layers[li].cels[f] = Arc::new(buf);
        }
        doc.canvas_dirty = true;
        doc.sel_tex = None;
        // превью пересчитается при следующем открытии панели
        self.effect_prev_sig.clear();
    }

    pub(crate) fn any_unsaved(&self) -> bool {
        self.docs.iter().any(|d| d.unsaved)
    }

    pub(crate) fn update_playback(&mut self, ctx: &egui::Context) {
        let t = ctx.input(|i| i.time);
        for doc in &mut self.docs {
            if doc.playing {
                let interval = 1.0 / (doc.fps as f64).max(0.1);
                if doc.last_play == 0.0 {
                    doc.last_play = t;
                } else if t - doc.last_play >= interval {
                    doc.last_play = t;
                    let next = (doc.active_frame + 1) % doc.frames;
                    doc.set_active_frame(next);
                    ctx.request_repaint();
                }
            } else {
                doc.last_play = 0.0;
            }
        }
    }

    /// Перетаскивание файлов из файлового менеджера в окно.
    /// Каждый файл открывается отдельной вкладкой, активной становится последняя.
    /// Само подсвечивание при перетаскивании — в draw_drop_overlay.
    pub(crate) fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() { return; }
        let mut opened = 0usize;
        let mut failed = 0usize;
        for file in &dropped {
            // path опционален: для «виртуальных» файлов (например, из
            // встроенного браузера) его нет, и открыть такой нечего
            let Some(path) = file.path.as_ref() else {
                failed += 1;
                continue;
            };
            match self.open_path(path) {
                Some(_) => opened += 1,
                None => failed += 1,
            }
        }
        if opened > 0 {
            if opened == 1 {
                self.hint("image opened");
            } else {
                self.hint(format!("{} images opened", opened));
            }
        } else if failed > 1 {
            // про один файл open_path уже сообщил конкретную причину
            self.hint(format!("{} files could not be opened", failed));
        }
    }

    /// Рамка «отпустите, чтобы открыть» поверх всего окна, пока файл над ним.
    fn draw_drop_overlay(&self, ctx: &egui::Context) {
        let hovered = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if !hovered { return; }
        let screen = ctx.screen_rect();
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drop_overlay")));
        p.rect_filled(
            screen,
            0.0,
            Color32::from_black_alpha(140),
        );
        let inset = 8.0;
        let r = Rect::from_min_max(screen.min + Vec2::splat(inset), screen.max - Vec2::splat(inset));
        p.rect_stroke(r, 0.0, Stroke::new(6.0, ACCENT), egui::StrokeKind::Inside);
        let label = match ctx.input(|i| i.raw.hovered_files.len()) {
            1 => "Open image".to_owned(),
            n => format!("Open {} images", n),
        };
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(32.0),
            ACCENT,
        );
    }

    pub(crate) fn close_tab(&mut self, idx: usize) {
        if self.docs.len() <= 1 {
            self.docs[0] = Document::new_logo("Untitled");
            self.active_tab = 0;
            return;
        }
        self.docs.remove(idx);
        if self.active_tab >= self.docs.len() {
            self.active_tab = self.docs.len() - 1;
        } else if self.active_tab > idx {
            self.active_tab -= 1;
        }
    }
}

// ── eframe::App ──────────────────────────────────────
impl eframe::App for PixeshApp {
    /// eframe вызывает это раз в auto_save_interval и при выходе. Метода
    /// restore в eframe 0.31 нет, поэтому чтение живёт в load_prefs.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        #[cfg(feature = "persistence")]
        eframe::set_value(storage, config::PREFS_KEY, &self.prefs());
        #[cfg(not(feature = "persistence"))]
        let _ = storage;
    }

    /// Дефолт — 30 секунд. Столько теряется при падении, а панель тянут
    /// мышкой и вполне можно успеть за это время закрыть приложение.
    fn auto_save_interval(&self) -> std::time::Duration {
        std::time::Duration::from_secs(10)
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.last_time = ctx.input(|i| i.time);
        self.handle_input(ctx);
        self.update_playback(ctx);
        self.handle_dropped_files(ctx);

        // ── Desktop-style layout for everyone ──
        // порядок панелей вычисляется из раскладки: сначала верхние и нижние
        // полосы, потом боковые, потом холст
        for p in self.draw_order() {
            match p {
                PanelId::Toolbar => self.ui_toolbar(ctx),
                PanelId::Layers => self.ui_layers(ctx),
                PanelId::Status => self.ui_status(ctx),
                PanelId::Timeline => self.ui_timeline(ctx),
            }
        }
        self.ui_canvas(ctx);

        self.ui_dialogs(ctx);
        self.draw_drop_overlay(ctx);
        self.ui_panel_drag(ctx);

        if ctx.input(|i| i.viewport().close_requested()) && self.any_unsaved() && !self.show_quit_dialog && !self.close_handled {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_quit_dialog = true;
        }
        if !ctx.input(|i| i.viewport().close_requested()) {
            self.close_handled = false;
        }
        if self.show_quit_dialog {
            egui::Area::new("quit_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(260.0);
                    
                    let t = ui.ctx().animate_bool(ui.make_persistent_id("quit_anim"), self.show_quit_dialog);
                    let size = size * (0.8 + 0.2 * t);
                    
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let p = ui.painter();
                    
                    let _alpha = (t * 255.0) as u8;
                    let bg = PANEL.gamma_multiply(t);
                    let brd = BORDER.gamma_multiply(t);
                    
                    p.rect_filled(rect, 0.0, bg);
                    p.rect_stroke(rect, 0.0, Stroke::new(4.0, brd), egui::StrokeKind::Outside);
                    let mut child = ui.new_child(
                        egui::UiBuilder::new()
                            .layout(egui::Layout::top_down(egui::Align::Center))
                            .max_rect(rect),
                    );
                    child.style_mut().text_styles.insert(
                        egui::TextStyle::Body,
                        egui::FontId::proportional(24.0),
                    );
                    child.style_mut().text_styles.insert(
                        egui::TextStyle::Button,
                        egui::FontId::proportional(28.0),
                    );
                    child.add_space(24.0);
                    child.label(egui::RichText::new("Unsaved changes!").size(26.0).color(TEXT));
                    child.add_space(12.0);
                    child.label(egui::RichText::new("Quit anyway?").size(22.0).color(TEXT));
                    child.add_space(child.available_height() - 44.0);
                    let spacing = child.style().spacing.item_spacing.x;
                    let half = (child.available_width() - spacing) / 2.0;
                    child.horizontal(|ui| {
                        if btn_min_w(ui, "Quit", half) {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if btn_min_w(ui, "Cancel", half) {
                            self.show_quit_dialog = false;
                            self.close_handled = true;
                        }
                    });
                });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_png(name: &str, w: u32, h: u32) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("pixesh_open_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let img = image::RgbaImage::from_pixel(w, h, image::Rgba([1, 2, 3, 255]));
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn default_document_carries_the_mascot() {
        let doc = Document::new_logo("Untitled");
        assert_eq!(
            (doc.width, doc.height),
            (MASCOT_SIDE, MASCOT_SIDE),
            "стартовый холст подгонен под размер совы"
        );
        let px = &doc.layers[0].cels[0];
        assert_eq!(px.len(), MASCOT_SIDE * MASCOT_SIDE, "залит весь холст");
        assert!(px.iter().any(|c| c.a() > 0), "сова нарисована, а не пустой холст");
        assert!(!doc.unsaved, "совы по умолчанию — не правка пользователя");
    }

    #[test]
    fn blank_document_stays_blank() {
        let doc = Document::new("Untitled");
        assert!(
            doc.layers[0].cels[0].iter().all(|c| c.a() == 0),
            "Document::new по-прежнему даёт пустой холст — на нём держатся другие тесты"
        );
    }

    #[test]
    fn mascot_is_a_faithful_copy_of_the_source_png() {
        let doc = Document::new_logo("t");
        let src = image::load_from_memory(include_bytes!("../../img/pixel_logo.png"))
            .expect("совы должна декодироваться")
            .to_rgba8();
        assert_eq!(
            src.dimensions(),
            (MASCOT_SIDE as u32, MASCOT_SIDE as u32),
            "источник обязан совпадать с холстом — иначе маскот начнёт ужиматься"
        );
        let px = &doc.layers[0].cels[0];
        for y in 0..MASCOT_SIDE {
            for x in 0..MASCOT_SIDE {
                let p = src.get_pixel(x as u32, y as u32);
                let want = Color32::from_rgba_unmultiplied(p[0], p[1], p[2], p[3]);
                assert_eq!(px[y * MASCOT_SIDE + x], want, "пиксель ({x},{y}) разошёлся");
            }
        }
    }

    #[test]
    fn open_path_adds_a_tab_and_activates_it() {
        let path = tmp_png("first.png", 8, 5);
        let mut app = PixeshApp::new();
        let tabs_before = app.docs.len();

        let opened = app.open_path(&path).expect("файл должен открыться");
        assert_eq!(opened, "first.png");
        assert_eq!(app.docs.len(), tabs_before + 1, "появилась новая вкладка");
        assert_eq!(app.active_tab, app.docs.len() - 1, "новая вкладка активна");

        let doc = &app.docs[app.active_tab];
        assert_eq!((doc.width, doc.height), (8, 5), "размеры взяты из картинки");
        assert_eq!(doc.name, "first.png", "имя вкладки — имя файла");
        assert!(!doc.unsaved, "только что открытый файл не считается изменённым");
    }

    #[test]
    fn open_path_keeps_previous_tabs_untouched() {
        let mut app = PixeshApp::new();
        app.docs[0].name = "Untitled".into();
        app.docs[0].width = 32;
        app.docs[0].height = 32;

        app.open_path(&tmp_png("second.png", 7, 9));
        assert_eq!(app.docs[0].name, "Untitled");
        assert_eq!((app.docs[0].width, app.docs[0].height), (32, 32),
                   "прежний документ не тронут — это отдельная вкладка");
    }

    #[test]
    fn open_path_rejects_non_image_without_adding_a_tab() {
        let dir = std::env::temp_dir().join("pixesh_open_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("readme.txt");
        std::fs::write(&path, b"not an image").unwrap();

        let mut app = PixeshApp::new();
        let tabs_before = app.docs.len();
        assert!(app.open_path(&path).is_none(), "не-картинка не открывается");
        assert_eq!(app.docs.len(), tabs_before, "пустую вкладку не создаём");
        assert!(!app.status_hint.is_empty(), "пользователь должен узнать о проблеме");
    }
}

// ── сохранение настроек ───────────────────────────────
#[cfg(feature = "persistence")]
#[cfg(test)]
mod prefs_tests {
    use super::*;
    use eframe::Storage;
    use std::cell::RefCell;
    use std::collections::HashMap;

    #[derive(Default)]
    struct MemStore(RefCell<HashMap<String, String>>);

    impl Storage for MemStore {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.borrow().get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.borrow_mut().insert(key.to_owned(), value);
        }
        fn flush(&mut self) {}
    }

    fn store_with(app: &PixeshApp) -> MemStore {
        let mut s = MemStore::default();
        eframe::set_value(&mut s, config::PREFS_KEY, &app.prefs());
        s
    }

    #[test]
    fn panel_visibility_survives_a_restart() {
        let mut first = PixeshApp::new();
        // прячем тулбар и таймлайн — как это делает диалог панелей
        first.show_top_panel = false;
        first.show_timeline = false;
        first.show_status_bar = false;
        first.welcome_show_again = false;

        let store = store_with(&first);
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert!(!second.show_top_panel, "закрытый тулбар не должен вернуться");
        assert!(!second.show_timeline, "закрытый таймлайн не должен вернуться");
        assert!(!second.show_status_bar);
        assert!(second.show_right_panel, "не тронутая панель остаётся на месте");
        assert!(!second.welcome_show_again);
    }

    #[test]
    fn tool_and_brush_survive_a_restart() {
        let mut first = PixeshApp::new();
        first.tool = Tool::Text;
        first.brush = 9.0;
        first.brush_shape = BrushShape::Round;
        first.arrow_speed = 33.0;

        let store = store_with(&first);
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert_eq!(second.tool, Tool::Text);
        assert_eq!(second.brush, 9.0);
        assert_eq!(second.brush_shape, BrushShape::Round);
        assert_eq!(second.arrow_speed, 33.0);
    }

    /// Цвет хранится как RGBA, а HSV/RGB-поля из него пересчитываются —
    /// иначе пикер и слайдеры показывали бы разные цвета.
    #[test]
    fn color_is_restored_consistently_across_representations() {
        let mut first = PixeshApp::new();
        first.set_color(Color32::from_rgb(200, 100, 50));
        first.color_history.push(Color32::from_rgb(1, 2, 3));

        let store = store_with(&first);
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert_eq!(second.color, Color32::from_rgb(200, 100, 50));
        assert_eq!(second.rgb_r, 200.0);
        assert_eq!(second.rgb_g, 100.0);
        let (h, s, v) = crate::color::rgb_to_hsv(200, 100, 50);
        assert_eq!(second.hsv_h, h);
        assert_eq!(second.hsv_s, s);
        assert_eq!(second.hsv_v, v);
        assert_eq!(second.color_history, vec![Color32::from_rgb(1, 2, 3)]);
    }

    /// 0–360° — замкнутое кольцо: за порогом цвет должен вернуться к красному,
    /// а не застрять на clamp. Иначе клавишей оттенка замкнуть круг нельзя.
    #[test]
    fn bump_hsv_wraps_hue_around_the_wheel() {
        let mut app = PixeshApp::new();
        app.hsv_h = 358.0;
        app.bump_hsv(5.0, 0.0, 0.0);
        assert_eq!(app.hsv_h, 3.0, "358 + 5 даёт 3, а не 360");

        app.bump_hsv(-5.0, 0.0, 0.0);
        assert_eq!(app.hsv_h, 358.0, "и обратно, через 0");
    }

    /// Шаг насыщенности и яркости ограничен диапазоном: иначе цветовые поля
    /// ушли бы за 255 и пересчёт через u8 начал бы врать.
    #[test]
    fn bump_hsv_clamps_saturation_and_value() {
        let mut app = PixeshApp::new();
        app.hsv_s = 253.0;
        app.hsv_v = 3.0;
        app.bump_hsv(0.0, 5.0, -5.0);
        assert_eq!(app.hsv_s, 255.0, "насыщенность упирается в 255");
        assert_eq!(app.hsv_v, 0.0, "яркость упирается в 0");

        app.bump_hsv(0.0, -5.0, 0.0);
        assert_eq!(app.hsv_s, 250.0, "и обратно вниз работает");
    }

    /// Главный инвариант: что ни подправь в HSV, `color`, `rgb_*` и `hsv_*`
    /// обязаны описывать один и тот же цвет. На этом держатся пикер и слайдеры.
    #[test]
    fn bump_hsv_keeps_every_colour_representation_in_sync() {
        let mut app = PixeshApp::new();
        app.set_color(Color32::from_rgb(200, 100, 50));
        app.bump_hsv(30.0, -20.0, 15.0);

        let (r, g, b) = crate::color::hsv_to_rgb(app.hsv_h, app.hsv_s, app.hsv_v);
        assert_eq!(app.color.r(), r, "color.r не совпал с hsv_*");
        assert_eq!(app.color.g(), g, "color.g не совпал с hsv_*");
        assert_eq!(app.color.b(), b, "color.b не совпал с hsv_*");
        assert_eq!(app.rgb_r, r as f32, "rgb_r не совпал");
        assert_eq!(app.rgb_g, g as f32, "rgb_g не совпал");
        assert_eq!(app.rgb_b, b as f32, "rgb_b не совпал");
    }

    /// Прозрачность живёт отдельно от HSV: её изменение не должно сдвигать
    /// ни цвет, ни hsv_*.
    ///
    /// Сравнивать здесь `color.r()` с исходным нельзя: egui хранит `Color32`
    /// премультиплицированным, поэтому у полупрозрачного цвета `r()` отдаёт
    /// уже умноженный на альфу канал. Проверяем собственные поля приложения,
    /// которые как раз ведут себя как неумноженные.
    #[test]
    fn bump_alpha_leaves_the_colour_itself_alone() {
        let mut app = PixeshApp::new();
        app.set_color(Color32::from_rgb(200, 100, 50));
        let rgb_before = (app.rgb_r, app.rgb_g, app.rgb_b);
        let hsv_before = (app.hsv_h, app.hsv_s, app.hsv_v);

        app.bump_alpha(-5.0);
        assert_eq!(app.rgb_a, 250.0, "альфа уменьшилась на шаг");
        assert_eq!((app.rgb_r, app.rgb_g, app.rgb_b), rgb_before, "RGB не тронут");
        assert_eq!((app.hsv_h, app.hsv_s, app.hsv_v), hsv_before, "HSV не тронут");
        assert_eq!(app.color.a(), 250, "color.a согласован с rgb_a");
    }

    /// Главная защита от раздражающего бага: диалоги не должны вылезать
    /// при запуске, даже если в файле что-то о них есть.
    #[test]
    fn dialogs_never_restore() {
        let mut first = PixeshApp::new();
        first.show_resize = true;
        first.show_scale = true;
        first.show_export = true;
        first.show_panels = true;
        first.show_settings = true;
        first.show_quit_dialog = true;
        first.show_effects = true;
        first.show_text = true;
        first.renaming_layer = Some(0);

        let store = store_with(&first);
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert!(!second.show_resize && !second.show_scale && !second.show_export);
        assert!(!second.show_panels && !second.show_settings && !second.show_quit_dialog);
        assert!(!second.show_effects && !second.show_text);
        assert!(second.renaming_layer.is_none());
    }

    /// Эффекты сохранять не просили, и инструмент Effects без открытой панели
    /// бессмысленен — приводим к кисти.
    #[test]
    fn effects_tool_is_not_restored() {
        let mut first = PixeshApp::new();
        first.tool = Tool::Effects;
        first.welcome_show_again = false;
        let store = store_with(&first);

        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));
        assert_eq!(second.tool, Tool::Brush);
        assert!(!second.show_welcome);
    }

    /// Пустое хранилище = первый запуск: ничего не ломается, всё живо.
    #[test]
    fn first_run_without_a_file_is_all_defaults() {
        let mut app = PixeshApp::new();
        app.load_prefs(None);
        assert!(app.show_top_panel && app.show_right_panel);
        assert!(app.show_status_bar && app.show_timeline);
        assert!(app.welcome_show_again && app.show_welcome);
        assert_eq!(app.tool, Tool::Brush);
        assert_eq!(app.brush, 1.0);
    }
}

// ── раскладка панелей ────────────────────────────────
#[cfg(test)]
mod layout_tests {
    use super::*;

    /// Главное свойство порядка отрисовки: все горизонтальные полосы раньше
    /// боковых. Иначе верхняя панель, нарисованная после боковой, получила бы
    /// не всю ширину окна — egui вырезает панели из остатка по порядку вызовов.
    #[test]
    fn horizontal_panels_are_drawn_before_side_panels() {
        let mut app = PixeshApp::new();
        // худший случай: тулбар уехал вниз, слои — влево
        app.toolbar_edge = Edge::Bottom;
        app.layers_edge = Edge::Left;
        app.status_edge = Edge::Top;
        app.timeline_edge = Edge::Top;

        let order = app.draw_order();
        let first_side = order
            .iter()
            .position(|p| app.edge_of(*p).is_vertical())
            .unwrap_or(order.len());
        let last_horizontal = order
            .iter()
            .rposition(|p| !app.edge_of(*p).is_vertical())
            .unwrap_or(0);
        assert!(
            last_horizontal < first_side,
            "боковая панель нарисовалась раньше горизонтальной: {order:?}"
        );
    }

    /// Взаимный порядок панелей не зависит от того, на каком краю они стоят:
    /// он канонический, иначе панели прыгали бы местами при каждом переносе.
    #[test]
    fn order_inside_an_edge_is_stable() {
        let canonical = vec![PanelId::Toolbar, PanelId::Status, PanelId::Timeline, PanelId::Layers];

        // все три горизонтальные наверху, слои слева
        let mut app = PixeshApp::new();
        app.toolbar_edge = Edge::Top;
        app.status_edge = Edge::Top;
        app.timeline_edge = Edge::Top;
        app.layers_edge = Edge::Left;
        assert_eq!(app.draw_order(), canonical, "верх + лево");

        // те же панели, но все внизу, слои справа — порядок тот же
        let mut app = PixeshApp::new();
        app.toolbar_edge = Edge::Bottom;
        app.status_edge = Edge::Bottom;
        app.timeline_edge = Edge::Bottom;
        app.layers_edge = Edge::Right;
        assert_eq!(app.draw_order(), canonical, "низ + право");

        // перенос таймлайна наверх не меняет его места относительно соседей
        let mut app = PixeshApp::new();
        app.timeline_edge = Edge::Top;
        app.status_edge = Edge::Bottom;
        assert_eq!(
            app.draw_order(),
            vec![PanelId::Toolbar, PanelId::Timeline, PanelId::Status, PanelId::Layers],
            "таймлайн наверху, статус внизу — но таймлайн всё равно раньше"
        );
    }

    #[test]
    fn hidden_panels_are_not_drawn() {
        let mut app = PixeshApp::new();
        app.show_timeline = false;
        app.show_status_bar = false;
        let order = app.draw_order();
        assert!(!order.contains(&PanelId::Timeline));
        assert!(!order.contains(&PanelId::Status));
        assert_eq!(order, vec![PanelId::Toolbar, PanelId::Layers]);
    }

    /// Перенос не должен уводить панель на запрещённый край, даже если его
    /// подсунули руками (например, правкой файла настроек).
    #[test]
    fn set_edge_refuses_forbidden_edges() {
        let mut app = PixeshApp::new();
        app.set_edge(PanelId::Timeline, Edge::Left);
        assert_ne!(app.timeline_edge, Edge::Left, "таймлайн нельзя в бок");
        app.set_edge(PanelId::Layers, Edge::Top);
        assert_ne!(app.layers_edge, Edge::Top, "слои нельзя наверх");
        app.set_edge(PanelId::Timeline, Edge::Top);
        assert_eq!(app.timeline_edge, Edge::Top, "разрешённый край принимаем");
        app.set_edge(PanelId::Layers, Edge::Left);
        assert_eq!(app.layers_edge, Edge::Left);
    }

    #[test]
    fn panel_ids_differ_per_edge_so_sizes_do_not_mix() {
        // ключевой момент: PanelState лежит в egui::Memory по Id, и для боковой
        // панели читается как rect.width(), а для горизонтальной как
        // rect.height(). Разные Id на каждый край — иначе панель, переехавшая с
        // низа на бок, прочитала бы старую высоту как ширину
        assert_ne!(
            PanelId::Timeline.ui_id(Edge::Bottom),
            PanelId::Timeline.ui_id(Edge::Top)
        );
        assert_ne!(
            PanelId::Layers.ui_id(Edge::Left),
            PanelId::Layers.ui_id(Edge::Right)
        );
        // и при этом Id уникальны между панелями
        assert_ne!(
            PanelId::Timeline.ui_id(Edge::Top),
            PanelId::Toolbar.ui_id(Edge::Top)
        );
    }

    #[test]
    fn edges_survive_a_restart() {
        let mut first = PixeshApp::new();
        first.set_edge(PanelId::Timeline, Edge::Top);
        first.set_edge(PanelId::Toolbar, Edge::Bottom);
        first.set_edge(PanelId::Layers, Edge::Left);

        let mut store = config::MemStore::default();
        eframe::set_value(&mut store, config::PREFS_KEY, &first.prefs());
        let mut second = PixeshApp::new();
        second.load_prefs(Some(&store));

        assert_eq!(second.timeline_edge, Edge::Top);
        assert_eq!(second.toolbar_edge, Edge::Bottom);
        assert_eq!(second.layers_edge, Edge::Left);
        assert_eq!(second.status_edge, Edge::Bottom);
    }

    /// Файл настроек от старой версии без полей краёв читается как исходная
    /// раскладка, а не как «панели перепутаны».
    #[test]
    fn layout_without_edge_fields_keeps_the_original_arrangement() {
        let mut store = config::MemStore::default();
        eframe::set_value(
            &mut store,
            config::PREFS_KEY,
            &config::Prefs { show_timeline: false, ..Default::default() },
        );
        let mut app = PixeshApp::new();
        app.load_prefs(Some(&store));

        assert!(!app.show_timeline);
        assert_eq!(app.toolbar_edge, Edge::Top);
        assert_eq!(app.layers_edge, Edge::Right);
        assert_eq!(app.status_edge, Edge::Bottom);
        assert_eq!(app.timeline_edge, Edge::Bottom);
    }

    #[test]
    fn a_forbidden_edge_in_the_file_is_repaired() {
        let mut store = config::MemStore::default();
        eframe::set_value(
            &mut store,
            config::PREFS_KEY,
            &config::Prefs {
                timeline_edge: Edge::Left, // мусор в файле
                layers_edge: Edge::Top,     // мусор в файле
                ..Default::default()
            },
        );
        let mut app = PixeshApp::new();
        app.load_prefs(Some(&store));
        assert_eq!(app.timeline_edge, Edge::Bottom, "вернули таймлайн вниз");
        assert_eq!(app.layers_edge, Edge::Right, "вернули слои вправо");
    }
}
