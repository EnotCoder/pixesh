mod app;
mod color;
mod constants;
mod ui;

use eframe::egui;

use constants::*;

/// Boot the egui app. Called from `fn main` in src/main.rs.
pub fn run_native_app(native_options: eframe::NativeOptions) -> eframe::Result {
    // Read the font file at compile time (located in the project root)
    let font_data: &'static [u8] = include_bytes!("../tex/font.otf");

    eframe::run_native(
        "Pixesh",
        native_options,
        Box::new(move |cc| {
            // ── Font ──
            // Replace the default egui font with our pixel font
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("pixelfont".into(), egui::FontData::from_static(font_data).into());
            for family in fonts.families.values_mut() {
                family.insert(0, "pixelfont".into());
            }
            cc.egui_ctx.set_fonts(fonts);

            // ── Style ──
            // Configure dark theme and colors
            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals {
                dark_mode: true,
                override_text_color: Some(TEXT),        // Default text color
                window_fill: PANEL,                     // Window background
                panel_fill: PANEL,                      // Panel background
                faint_bg_color: PANEL_LIGHT,            // Faint background
                extreme_bg_color: BG,                   // Darkest background
                ..Default::default()
            };
            style.spacing.item_spacing = egui::Vec2::new(10.0, 6.0);   // Widget spacing
            style.spacing.button_padding = egui::Vec2::new(10.0, 6.0); // Button inner padding
            // Pixel-art style: zero corner radius everywhere
            style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.active.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.widgets.open.corner_radius = egui::CornerRadius::ZERO;
            style.visuals.window_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.menu_corner_radius = egui::CornerRadius::ZERO;
            style.visuals.handle_shape = egui::style::HandleShape::Rect { aspect_ratio: 1.0 };
            style.text_styles.insert(
                egui::TextStyle::Body,                  // "Body" text style
                egui::FontId::proportional(FONT_SZ),    // Custom font size
            );
            cc.egui_ctx.set_style(style);

            // Create the application instance
            let mut app = app::PixeshApp::new();

            // egui restores panel dimensions and window geometry itself from
            // app.ron (persist_egui_memory); here we load user preferences that
            // egui is unaware of: panel visibility, brush size, tool, colors
            app.load_prefs(cc.storage);

            Ok(Box::new(app))
        }),
    )
}