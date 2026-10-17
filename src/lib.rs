mod app;
mod color;
mod constants;
mod ui;

use eframe::egui;

use constants::*;

/// Boot the egui app. Called from `fn main` in src/main.rs.
pub fn run_native_app(native_options: eframe::NativeOptions) -> eframe::Result {
    // читаем файл шрифта во время компиляции (лежит в корне проекта)
    let font_data: &'static [u8] = include_bytes!("../tex/font.otf");

    eframe::run_native(
        "Pixesh",
        native_options,
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
            style.spacing.item_spacing = egui::Vec2::new(10.0, 6.0);   // отступы между виджетами
            style.spacing.button_padding = egui::Vec2::new(10.0, 6.0); // отступы внутри кнопок
            // пиксель-арт стиль: все скругления = 0
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 1.0 };
            style.text_styles.insert(
                egui::TextStyle::Body,                  // текстовый стиль "Body"
                egui::FontId::proportional(FONT_SZ),    // используем наш размер шрифта
            );
            cc.egui_ctx.set_style(style);

            // создаём экземпляр приложения
            let mut app = app::PixeshApp::new();

            // размеры панелей и геометрию окна egui восстанавливает сам из
            // app.ron (persist_egui_memory); здесь — настройки, о которых
            // egui не знает: видимость панелей, кисть, инструмент, цвета
            app.load_prefs(cc.storage);

            Ok(Box::new(app))
        }),
    )
}