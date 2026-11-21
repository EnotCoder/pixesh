mod app;
mod color;
mod constants;
mod ui;

use eframe::egui;

use constants::*;

/// What to do with the command line before the window is created.
enum Cli {
    /// Print this text and exit (``--version`` / ``--help``).
    Print(&'static str),
    /// Launch the GUI with these images opened in tabs.
    Launch(Vec<std::path::PathBuf>),
}

const HELP: &str = "Usage: pixesh [FILE...]\n\nOpen images in Pixesh, one tab per file.\n\n  -h, --help     Print this help\n  -V, --version  Print the version\n";

/// Separates the two CLI flags from file paths. Unknown flags are ignored so
/// that desktop environments passing extra arguments cannot break startup.
fn parse_args(args: Vec<std::ffi::OsString>) -> Cli {
    let mut paths = Vec::new();
    for arg in args {
        match arg.to_str() {
            Some("-V" | "--version") => {
                return Cli::Print(concat!("pixesh ", env!("CARGO_PKG_VERSION"), "\n"));
            }
            Some("-h" | "--help") => return Cli::Print(HELP),
            _ => {}
        }
        let text = arg.to_string_lossy();
        if text.starts_with('-') && text.len() > 1 {
            eprintln!("pixesh: unknown option '{text}' (ignored)");
            continue;
        }
        paths.push(std::path::PathBuf::from(arg));
    }
    Cli::Launch(paths)
}

/// Boot the egui app. Called from `fn main` in src/main.rs.
pub fn run_native_app(
    native_options: eframe::NativeOptions,
    args: Vec<std::ffi::OsString>,
) -> eframe::Result {
    let paths = match parse_args(args) {
        Cli::Print(text) => {
            print!("{text}");
            return Ok(());
        }
        Cli::Launch(paths) => paths,
    };

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

            // Command-line files behave exactly like dropped ones. This has to
            // happen after load_prefs, which resets dialogs and tab state.
            for path in &paths {
                if app.open_path(path).is_some() {
                    app.show_welcome = false;
                }
            }

            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn launch(args: &[&str]) -> Vec<std::path::PathBuf> {
        match parse_args(args.iter().map(std::ffi::OsString::from).collect()) {
            Cli::Launch(paths) => paths,
            Cli::Print(_) => panic!("expected a launch, got an early exit"),
        }
    }

    #[test]
    fn files_from_argv_become_paths() {
        assert_eq!(
            launch(&["one.png", "dir/two.jpg"]),
            vec![
                std::path::PathBuf::from("one.png"),
                std::path::PathBuf::from("dir/two.jpg")
            ]
        );
    }

    #[test]
    fn flags_exit_early() {
        for flag in ["-V", "--version", "-h", "--help"] {
            let args = vec![std::ffi::OsString::from(flag)];
            assert!(matches!(parse_args(args), Cli::Print(_)), "{flag}");
        }
    }

    #[test]
    fn unknown_flags_do_not_become_files() {
        assert_eq!(
            launch(&["--unknown", "pic.png"]),
            vec![std::path::PathBuf::from("pic.png")]
        );
    }
}
