use eframe::egui::Color32;

use crate::app::config;
use crate::constants::{BrushShape, Tool};

use super::PixeshApp;

fn store_with(app: &PixeshApp) -> config::MemStore {
    let mut s = config::MemStore::default();
    eframe::set_value(&mut s, config::PREFS_KEY, &app.prefs());
    s
}

#[test]
fn panel_visibility_survives_a_restart() {
    let mut first = PixeshApp::new();
    // hide the toolbar and timeline — as the panels dialog does
    first.show_top_panel = false;
    first.show_timeline = false;
    first.show_status_bar = false;
    first.welcome_show_again = false;
    first.show_snow = true;

    let store = store_with(&first);
    let mut second = PixeshApp::new();
    second.load_prefs(Some(&store));

    assert!(!second.show_top_panel, "закрытый тулбар не должен вернуться");
    assert!(!second.show_timeline, "закрытый таймлайн не должен вернуться");
    assert!(!second.show_status_bar);
    assert!(second.show_right_panel, "не тронутая панель остаётся на месте");
    assert!(!second.welcome_show_again);
    assert!(second.show_snow, "включённый снег переживает перезапуск");
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

/// The colour is stored as RGBA, and the HSV/RGB fields are recomputed
/// from it — otherwise the picker and the sliders would show different
/// colours.
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

/// 0–360° is a closed ring: past the threshold the colour must wrap back
/// to red instead of getting stuck at the clamp. Otherwise the hue key
/// could not complete the circle.
#[test]
fn bump_hsv_wraps_hue_around_the_wheel() {
    let mut app = PixeshApp::new();
    app.hsv_h = 358.0;
    app.bump_hsv(5.0, 0.0, 0.0);
    assert_eq!(app.hsv_h, 3.0, "358 + 5 даёт 3, а не 360");

    app.bump_hsv(-5.0, 0.0, 0.0);
    assert_eq!(app.hsv_h, 358.0, "и обратно, через 0");
}

/// Saturation and value steps are bounded by the range: otherwise the
/// colour fields would go past 255 and the u8 round-trip would start to lie.
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

/// The main invariant: whatever you adjust in HSV, `color`, `rgb_*` and
/// `hsv_*` must describe the same colour. The picker and sliders rely on
/// this.
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

/// Opacity lives separately from HSV: changing it must not shift either
/// the colour or hsv_*.
///
/// `color.r()` cannot be compared with the original here: egui stores
/// `Color32` premultiplied, so for a semi-transparent colour `r()` returns
/// the channel already multiplied by alpha. We check the app's own fields,
/// which behave as non-premultiplied values.
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

#[test]
fn pick_preset_sets_the_expected_colour() {
    let mut app = PixeshApp::new();
    app.pick_preset(0);
    assert_eq!(app.color, Color32::from_rgb(255, 0, 0), "клавиша 1 — красный");
    app.pick_preset(2);
    assert_eq!(app.color, Color32::from_rgb(0, 0, 255), "клавиша 3 — синий");
    app.pick_preset(8);
    assert_eq!(app.color, Color32::from_rgb(0, 0, 0), "клавиша 9 — чёрный");
}

/// A preset is a ready-made opaque colour. If it reused the current alpha,
/// red dimmed to 128 would stay semi-transparent even after pressing "1".
#[test]
fn pick_preset_always_uses_full_opacity() {
    let mut app = PixeshApp::new();
    app.rgb_a = 128.0;
    app.pick_preset(0);
    assert_eq!(app.color.a(), 255, "пресет игнорирует текущую прозрачность");
    assert_eq!(app.rgb_a, 255.0, "rgb_a согласован");
}

#[test]
fn pick_preset_keeps_every_colour_representation_in_sync() {
    let mut app = PixeshApp::new();
    app.pick_preset(4); // cyan
    let (r, g, b) = crate::color::hsv_to_rgb(app.hsv_h, app.hsv_s, app.hsv_v);
    assert_eq!(
        (app.color.r(), app.color.g(), app.color.b()),
        (r, g, b),
        "color не совпал с hsv_*"
    );
    assert_eq!(
        (app.rgb_r, app.rgb_g, app.rgb_b),
        (r as f32, g as f32, b as f32),
        "rgb_* не совпал с hsv_*"
    );
}

#[test]
fn push_history_dedupes_and_caps_at_four() {
    let mut app = PixeshApp::new();
    for i in 0u8..6 {
        app.push_history(Color32::from_rgb(i * 10, 0, 0));
    }
    assert_eq!(app.color_history.len(), 4, "история ограничена четырьмя");
    assert_eq!(app.color_history[0], Color32::from_rgb(20, 0, 0), "старые выпали");

    // an existing colour does not create a duplicate but moves to the end
    app.push_history(Color32::from_rgb(20, 0, 0));
    assert_eq!(app.color_history.len(), 4, "дубликат не добавил длины");
    assert_eq!(
        *app.color_history.last().unwrap(),
        Color32::from_rgb(20, 0, 0),
        "повторно выбранный цвет всплыл наверх"
    );
}

/// The main protection against an annoying bug: dialogs must not pop up on
/// startup, even if the file contains something about them.
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

/// Nobody asked to persist effects, and the Effects tool is meaningless
/// without an open panel — fall back to the brush.
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

/// Empty storage = first launch: nothing breaks, everything is alive.
#[test]
fn first_run_without_a_file_is_all_defaults() {
    let mut app = PixeshApp::new();
    app.load_prefs(None);
    assert!(app.show_top_panel && app.show_right_panel);
    assert!(app.show_status_bar && app.show_timeline);
    assert!(!app.show_snow);
    assert!(app.welcome_show_again && app.show_welcome);
    assert_eq!(app.tool, Tool::Brush);
    assert_eq!(app.brush, 1.0);
}
