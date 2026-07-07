mod constants;
mod ui;
mod app;

use eframe::egui::{self, Vec2};

use crate::app::PixeshApp;
use crate::constants::*;

fn main() -> eframe::Result {
    let font_data: &'static [u8] = include_bytes!("../font.otf");

    eframe::run_native(
        "Pixesh",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([960.0, 700.0])
                .with_min_inner_size([400.0, 300.0]),
            ..Default::default()
        },
        Box::new(move |cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts
                .font_data
                .insert("pixelfont".into(), egui::FontData::from_static(font_data).into());
            for family in fonts.families.values_mut() {
                family.insert(0, "pixelfont".into());
            }
            cc.egui_ctx.set_fonts(fonts);

            let mut style = (*cc.egui_ctx.style()).clone();
            style.visuals = egui::Visuals {
                dark_mode: true,
                override_text_color: Some(TEXT),
                window_fill: PANEL,
                panel_fill: PANEL,
                faint_bg_color: PANEL_LIGHT,
                extreme_bg_color: BG,
                ..Default::default()
            };
            style.spacing.item_spacing = Vec2::new(6.0, 4.0);
            style.spacing.button_padding = Vec2::new(4.0, 2.0);
            style.text_styles.insert(
                egui::TextStyle::Body,
                egui::FontId::proportional(FONT_SZ),
            );
            cc.egui_ctx.set_style(style);

            Ok(Box::new(PixeshApp::new()))
        }),
    )
}
