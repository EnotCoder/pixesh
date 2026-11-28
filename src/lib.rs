mod app;
mod color;
mod constants;
mod ui;

use eframe::egui;

/// What to do with the command line before the window is created.
enum Cli {
    /// Print this text and exit (``--version`` / ``--help``).
    Print(&'static str),
    /// Launch the GUI with these images opened in tabs.
    Launch(Vec<std::path::PathBuf>),
}

const HELP: &str = "Usage: pixesh [FILE...]\n\nOpen images in Pixesh, one tab per file.\n\n  -h, --help        Print this help\n  -V, --version     Print the version\n  --config PATH     Use a specific Lua config file\n";

/// Separates CLI flags from file paths. Unknown flags are ignored so that
/// desktop environments passing extra arguments cannot break startup.
/// `--config` is consumed here as well; `lua_config` reads its value from the
/// original argv when choosing a file.
fn parse_args(args: Vec<std::ffi::OsString>) -> Cli {
    let mut paths = Vec::new();
    let mut args = args.into_iter();
    while let Some(arg) = args.next() {
        match arg.to_str() {
            Some("-V" | "--version") => {
                return Cli::Print(concat!("pixesh ", env!("CARGO_PKG_VERSION"), "\n"));
            }
            Some("-h" | "--help") => return Cli::Print(HELP),
            Some("--config") => { args.next(); continue; }
            _ => {}
        }
        let text = arg.to_string_lossy();
        if text.starts_with("--config=") { continue; }
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

    // The first launch writes a documented default config; later launches and
    // Ctrl+Shift+R parse the file. A broken file leaves the defaults in place.
    app::lua_config::init(app::lua_config::config_path());

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
            // Dark pixel theme with colors from the Lua config.
            ui::apply_style(&cc.egui_ctx);

            // Create the application instance
            let mut app = app::PixeshApp::new();
            if let Some(err) = app::lua_config::load_error() {
                eprintln!("pixesh: config error: {err}");
                app.status_hint = format!("config error: {err}");
                app.status_hint_until = f64::INFINITY;
            }

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

    #[test]
    fn config_flag_and_its_path_are_not_opened_as_images() {
        assert_eq!(
            launch(&["--config", "my.lua", "pic.png"]),
            vec![std::path::PathBuf::from("pic.png")]
        );
        assert_eq!(
            launch(&["--config=my.lua"]),
            Vec::<std::path::PathBuf>::new()
        );
    }
}
