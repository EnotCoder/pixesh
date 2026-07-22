// модули панелей и подсистем
pub mod canvas;          // рисование: композит, кисть, линия, заливка
pub mod history;         // undo / redo (снапшоты пикселей)
pub mod input;           // клавиатура, зум, панорама, пипетка
pub mod io;              // save/load PNG, добавление/удаление слоёв, resize
pub mod panel_canvas;    // отрисовка холста и диспатч инструментов
pub mod panel_dialogs;   // окна: Resize Canvas, Export PNG, Settings
pub mod panel_layers;    // боковая панель: слои, HSV-пикер
pub mod panel_toolbar;   // верхняя панель: инструменты, слайдеры, галки
pub mod tools;           // обработчики инструментов (кисть, ластик, заливка, пипетка, выделение)

use std::sync::Arc;

use eframe::egui::{self, Color32, Stroke, Vec2};

use crate::constants::*;
use crate::ui::btn_min_w;

// ── Layer / Snapshot ─────────────────────────────────
// слой: имя, пиксели (Arc для COW), видимость
pub(crate) struct Layer {
    pub(crate) name: String,
    pub(crate) pixels: Arc<Vec<Color32>>,
    pub(crate) visible: bool,
}

// снапшот состояния для undo/redo (пиксели всех слоёв + активный слой + размер холста)
pub(crate) struct Snapshot {
    pub(crate) layers: Vec<Arc<Vec<Color32>>>,
    pub(crate) active: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

// ── App ──────────────────────────────────────────────
// главная структура приложения — содержит всё состояние
pub struct PixeshApp {
    // ── слои / холст ──
    pub(crate) layers: Vec<Layer>,
    pub(crate) active_layer: usize,
    pub(crate) width: usize,
    pub(crate) height: usize,

    // ── цвет / кисть / инструмент ──
    pub(crate) color: Color32,
    pub(crate) hsv_h: f32,
    pub(crate) hsv_s: f32,
    pub(crate) hsv_v: f32,
    pub(crate) rgb_r: f32,
    pub(crate) rgb_g: f32,
    pub(crate) rgb_b: f32,
    pub(crate) rgb_a: f32,
    pub(crate) brush: f32,            // размер кисти (1..10)
    pub(crate) tool: Tool,
    pub(crate) tool_saved: Option<Tool>,          // для временного переключения (пипетка)
    pub(crate) last_px_primary: Option<(i32, i32)>,   // прошлый пиксель для линии (ЛКМ)
    pub(crate) last_px_secondary: Option<(i32, i32)>, // прошлый пиксель для линии (ПКМ)

    // ── отображение ──
    pub(crate) grid: bool,
    pub(crate) zoom: f32,
    pub(crate) pan: Vec2,
    pub(crate) tex: Option<egui::TextureHandle>,          // текстура холста
    pub(crate) brush_tex: Option<egui::TextureHandle>,
    pub(crate) eraser_tex: Option<egui::TextureHandle>,
    pub(crate) fill_tex: Option<egui::TextureHandle>,
    pub(crate) drop_tex: Option<egui::TextureHandle>,
    pub(crate) clear_tex: Option<egui::TextureHandle>,
    pub(crate) logo_tex: Option<egui::TextureHandle>,       // иконка логотипа (logo.png)
    pub(crate) sv_tex: Option<egui::TextureHandle>,       // текстура SV-поля
    pub(crate) sv_tex_h: f32,
    pub(crate) select_tex: Option<egui::TextureHandle>,
    pub(crate) move_tex: Option<egui::TextureHandle>,
    pub(crate) h_tex: Option<egui::TextureHandle>,          // H-strip текстура (кэш)
    pub(crate) mirror_h_tex: Option<egui::TextureHandle>,
    pub(crate) mirror_v_tex: Option<egui::TextureHandle>,

    // ── undo / redo ──
    pub(crate) undo_stack: Vec<Snapshot>,
    pub(crate) redo_stack: Vec<Snapshot>,

    // ── флаг перерисовки текстуры холста ──
    pub(crate) canvas_dirty: bool,
    pub(crate) display_buf: Vec<Color32>,       // переиспользуемый буфер для composite_display

    // ── выделение (Select tool) ──
    pub(crate) sel: Option<(i32, i32, i32, i32)>,           // финальный прямоугольник (x0,y0,x1,y1)
    pub(crate) sel_start: Option<(i32, i32)>,               // начало рисования выделения
    pub(crate) sel_end: Option<(i32, i32)>,                 // текущий конец выделения
    pub(crate) sel_move_origin: Option<(i32, i32)>,         // пиксель, за который схватили
    pub(crate) sel_move_current: Option<(i32, i32)>,        // текущая позиция при перемещении
    pub(crate) sel_buffer: Option<Vec<Color32>>,            // скопированные пиксели выделения
    pub(crate) sel_buf_w: usize,
    pub(crate) sel_buf_h: usize,
    pub(crate) sel_tex: Option<egui::TextureHandle>,

    // ── перемещение холста (Move tool, без выделения) ──
    pub(crate) canvas_move_origin: Option<(i32, i32)>,
    pub(crate) canvas_move_current: Option<(i32, i32)>,

    // ── диалоги ──
    pub(crate) show_resize: bool,
    pub(crate) resize_w: f32,
    pub(crate) resize_h: f32,

    pub(crate) show_scale: bool,
    pub(crate) scale_w: f32,
    pub(crate) scale_h: f32,

    pub(crate) show_export: bool,
    pub(crate) export_name: String,
    pub(crate) export_path: String,

    pub(crate) show_brush: bool,           // диалог размера кисти

    // ── видимость панелей ──
    pub(crate) show_panels: bool,          // диалог управления панелями
    pub(crate) show_settings: bool,        // диалог настроек
    pub(crate) show_top_panel: bool,
    pub(crate) show_right_panel: bool,

    // ── панорама ──
    pub(crate) mid_pan_pos: Option<egui::Pos2>,

    // ── история цветов ──
    pub(crate) color_history: Vec<Color32>,

    // ── переименование слоёв ──
    pub(crate) renaming_layer: Option<usize>,
    pub(crate) rename_buf: String,

    // ── настройки ──
    pub(crate) arrow_speed: f32,           // скорость панорамы стрелками
    pub(crate) zoom_speed: f32,            // скорость зума колесом

    // ── флаг несохранённых изменений ──
    pub(crate) unsaved: bool,
    pub(crate) show_quit_dialog: bool,
}

impl PixeshApp {
    pub fn new() -> Self {
        Self {
            layers: vec![Layer {
                name: "Background".into(),
                pixels: Arc::new(vec![Color32::TRANSPARENT; 16 * 16]),
                visible: true,
            }],
            active_layer: 0,
            width: 16,
            height: 16,
            color: Color32::BLACK,
            hsv_h: 0.0,
            hsv_s: 0.0,
            hsv_v: 0.0,
            rgb_r: 0.0,
            rgb_g: 0.0,
            rgb_b: 0.0,
            rgb_a: 255.0,
            brush: 1.0,
            tool: Tool::Brush,
            tool_saved: None,
            last_px_primary: None,
            last_px_secondary: None,
            grid: false,
            zoom: 46.0,
            pan: Vec2::ZERO,
            tex: None,
            brush_tex: None,
            eraser_tex: None,
            fill_tex: None,
            drop_tex: None,
            clear_tex: None,
            logo_tex: None,
            sv_tex: None,
            sv_tex_h: -1.0,
            select_tex: None,
            move_tex: None,
            h_tex: None,
            mirror_h_tex: None,
            mirror_v_tex: None,
            canvas_dirty: true,
            display_buf: Vec::new(),
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
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            show_resize: false,
            resize_w: 64.0,
            resize_h: 64.0,
            show_scale: false,
            scale_w: 64.0,
            scale_h: 64.0,
            show_export: false,
            export_name: "pixesh.png".into(),
            export_path: String::new(),
            show_brush: false,
            show_panels: false,
            show_settings: false,
            show_top_panel: true,
            show_right_panel: true,
            mid_pan_pos: None,
            color_history: Vec::new(),
            renaming_layer: None,
            rename_buf: String::new(),
            arrow_speed: 5.0,
            zoom_speed: 0.2,
            unsaved: false,
            show_quit_dialog: false,
        }
    }
}

impl PixeshApp {
    pub(crate) fn dialog_open(&self) -> bool {
        self.show_resize || self.show_export || self.show_brush || self.show_panels || self.show_settings || self.show_scale || self.show_quit_dialog
    }
}

// ── eframe::App ──────────────────────────────────────
// точка входа для eframe — каждый кадр вызывает update()
impl eframe::App for PixeshApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_input(ctx);
        if self.show_top_panel { self.ui_toolbar(ctx); }
        if self.show_right_panel { self.ui_layers(ctx); }
        self.ui_canvas(ctx);
        self.ui_dialogs(ctx);

        // ── подтверждение закрытия при несохранённых изменениях ──
        if ctx.input(|i| i.viewport().close_requested()) && self.unsaved && !self.show_quit_dialog {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.show_quit_dialog = true;
        }
        if self.show_quit_dialog {
            egui::Area::new("quit_dialog".into())
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .order(egui::Order::Foreground)
                .show(ctx, |ui| {
                    let size = Vec2::splat(260.0);
                    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
                    let p = ui.painter();
                    p.rect_filled(rect, 0.0, PANEL);
                    p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Outside);
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
                        }
                    });
                });
        }
    }
}
