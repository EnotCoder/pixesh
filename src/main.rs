// подключаем модули из других файлов
mod constants; // константы (цвета темы, размеры, Tool)
mod ui;        // готовые виджеты (кнопки, слайдеры, HSV)
mod app;       // вся логика и интерфейс

use eframe::egui::{self, Vec2};

use crate::app::PixeshApp;    // главная структура приложения
use crate::constants::*;      // цветовые константы (TEXT, PANEL, BG, FONT_SZ...)

// ── main ─────────────────────────────────────────────
// точка входа — запускает окно через eframe
fn main() -> eframe::Result {
    // читаем файл шрифта во время компиляции (лежит в корне проекта)
    let font_data: &'static [u8] = include_bytes!("../font.otf");

    // создаём окно через eframe (библиотека для окон/OpenGL)
    eframe::run_native(
        "Pixesh",                          // заголовок окна
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 700.0])   // начальный размер окна
                .with_min_inner_size([400.0, 300.0]), // минимальный размер
            ..Default::default()
        },
        // замыкание, которое вызывается один раз при старте
        Box::new(move |cc| {
            // ── шрифт ──
            // заменяем стандартный шрифт egui на наш пиксельный
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("pixelfont".into(), egui::FontData::from_static(font_data).into());
            for family in fonts.families.values_mut() {
                family.insert(0, "pixelfont".into());
            }
            cc.egui_ctx.set_fonts(fonts);

            // ── стиль ──
            // настраиваем тёмную тему и цвета
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals {
                dark_mode: true,
                override_text_color: Some(TEXT),        // цвет текста по умолчанию
                window_fill: PANEL,                     // фон окон
                panel_fill: PANEL,                      // фон панелей
                faint_bg_color: PANEL_LIGHT,            // бледный фон
                extreme_bg_color: BG,                   // самый тёмный фон
                ..Default::default()
            };
            style.spacing.item_spacing = Vec2::new(6.0, 4.0);   // отступы между виджетами
            style.spacing.button_padding = Vec2::new(4.0, 2.0); // отступы внутри кнопок
            style.text_styles.insert(
                egui::TextStyle::Body,                  // текстовый стиль "Body"
                egui::FontId::proportional(FONT_SZ),    // используем наш размер шрифта
            );
            cc.egui_ctx.set_style(style);

            // создаём экземпляр приложения — дальше eframe сам вызывает update()
            Ok(Box::new(PixeshApp::new()))
        }),
    )
}
