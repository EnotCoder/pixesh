mod app;
mod color;
mod constants;
mod ui;

use eframe::egui;

// ── android entry ────────────────────────────────────
// On Android eframe links an `android_main` exported from the cdylib
// (libpixesh.so) rather than a `main()` in the binary target. So it lives
// here, in the lib, gated to Android only.
#[cfg(target_os = "android")]
#[cfg_attr(target_os = "android", unsafe(no_mangle))]
fn android_main(app: winit::platform::android::activity::AndroidApp) {
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_fullscreen(true),
        android_app: Some(app),
        ..Default::default()
    };
    let _ = run_native_app(native_options);
}

use constants::*;

/// Boot the egui app. Desktop calls this from `fn main`, Android from `android_main`.
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
            let app = app::PixeshApp::new();

            // Если это мобильная версия, уменьшаем масштаб интерфейса,
            // чтобы на маленьком экране помещалось больше элементов.
            if app.mobile {
                cc.egui_ctx.set_pixels_per_point(0.8);
            }

            Ok(Box::new(app))
        }),
    )
}