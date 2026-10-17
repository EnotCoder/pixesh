//! Сохранение настроек между запусками.
//!
//! Размеры панелей и геометрия окна персистятся сами: egui держит их в
//! `egui::Memory`, а eframe сериализует всю память в app.ron, когда включена
//! фича `persistence`. Нам остаётся только состояние, о котором egui не знает.

use crate::constants::{BrushShape, Tool};

/// Стартовая директория для нативных диалогов открытия/сохранения.
/// Нужна только rfd, на Android нативных диалогов нет.
pub(crate) fn default_dir() -> String {
    std::env::var("HOME")
        .or_else(|_| std::env::var("EXTERNAL_STORAGE")) // Android fallback
        .unwrap_or_else(|_| "/".into())
}

/// Ключ настроек в хранилище eframe (RON внутри app.ron).
#[cfg_attr(not(feature = "persistence"), allow(dead_code))]
pub(crate) const PREFS_KEY: &str = "pixesh_prefs";

/// Настройки, которые переживают перезапуск.
///
/// `#[serde(default)]` на структуре — не украшение: без него добавление любого
/// нового поля ломает чтение всех ранее сохранённых файлов, и пользователь
/// молча получает дефолты. С `default` старый файл читается частично — новое
/// поле берётся из `Default`, остальное из файла.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub(crate) struct Prefs {
    pub show_top_panel: bool,
    pub show_right_panel: bool,
    pub show_status_bar: bool,
    pub show_timeline: bool,
    pub welcome_show_again: bool,
    pub arrow_speed: f32,
    pub zoom_speed: f32,
    pub brush: f32,
    pub brush_shape: BrushShape,
    pub tool: Tool,
    /// Текущий цвет как RGBA. HSV и RGB-поля из него пересчитываются при
    /// загрузке: хранить все три отдельно нельзя, они обязаны быть согласованы.
    pub rgba: [u8; 4],
    pub color_history: Vec<[u8; 4]>,
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            show_top_panel: true,
            show_right_panel: true,
            show_status_bar: true,
            show_timeline: true,
            welcome_show_again: true,
            arrow_speed: 5.0,
            zoom_speed: 1.0,
            brush: 1.0,
            brush_shape: BrushShape::Square,
            tool: Tool::Brush,
            rgba: [0, 0, 0, 255],
            color_history: Vec::new(),
        }
    }
}

impl Prefs {
    /// Читает настройки из хранилища eframe. Любая проблема — дефолты:
    /// битый файл не должен мешать запуску приложения.
    pub(crate) fn load(storage: Option<&dyn eframe::Storage>) -> Self {
        #[cfg(feature = "persistence")]
        let mut prefs: Self = storage
            .and_then(|s| eframe::get_value(s, PREFS_KEY))
            .unwrap_or_default();
        // на Android eframe не выдаёт каталог данных (storage_dir -> None),
        // так что файла там нет by design: остаются дефолты
        #[cfg(not(feature = "persistence"))]
        let mut prefs: Self = {
            let _ = storage;
            Self::default()
        };
        prefs.sanitize();
        prefs
    }

    /// Приводит значения из файла к допустимым. Файл лежит в каталоге данных
    /// приложения и остаётся руками редактируемым, а RON не проверит, что
    /// `brush` — положительное число. Без этого слайдер в диалоге настроек
    /// разъезжается, а кисть рисует точку за пределами холста.
    fn sanitize(&mut self) {
        self.arrow_speed = finite_or(self.arrow_speed, Self::default().arrow_speed)
            .clamp(1.0, 100.0);
        self.zoom_speed = finite_or(self.zoom_speed, Self::default().zoom_speed)
            .clamp(0.005, 5.0);
        // дальше тулбар всё равно поджимает кисть под размер холста
        self.brush = finite_or(self.brush, Self::default().brush).clamp(1.0, 512.0);
        if self.color_history.len() > 4 {
            self.color_history.truncate(4);
        }
    }
}

/// f32::clamp на NaN возвращает NaN, поэтому нечисловые значения отсекаем
/// до клампа, а не после.
fn finite_or(v: f32, fallback: f32) -> f32 {
    if v.is_finite() { v } else { fallback }
}

#[cfg(all(feature = "persistence", test))]
mod tests {
    use super::*;
    use eframe::Storage;
    use std::cell::RefCell;
    use std::collections::HashMap;

    /// Хранилище в памяти вместо файла. Так тесты идут через настоящий
    /// `eframe::get_value`, а не через подставной RON.
    #[derive(Default)]
    struct MemStore(RefCell<HashMap<String, String>>);

    impl eframe::Storage for MemStore {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.borrow().get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.borrow_mut().insert(key.to_owned(), value);
        }
        fn flush(&mut self) {}
    }

    fn round_trip(p: &Prefs) -> Prefs {
        let mut store = MemStore::default();
        eframe::set_value(&mut store, PREFS_KEY, p);
        Prefs::load(Some(&store))
    }

    #[test]
    fn prefs_survive_a_save_load_cycle() {
        let p = Prefs {
            show_timeline: false,
            show_status_bar: false,
            arrow_speed: 42.5,
            zoom_speed: 2.25,
            brush: 7.0,
            brush_shape: BrushShape::Round,
            tool: Tool::Text,
            rgba: [10, 20, 30, 128],
            color_history: vec![[1, 2, 3, 4], [5, 6, 7, 8]],
            ..Default::default()
        };
        let back = round_trip(&p);
        assert!(!back.show_timeline, "закрытая панель осталась закрытой");
        assert!(!back.show_status_bar);
        assert!(back.show_top_panel, "остальные видимости не пострадали");
        assert_eq!(back.arrow_speed, 42.5);
        assert_eq!(back.zoom_speed, 2.25);
        assert_eq!(back.brush, 7.0);
        assert_eq!(back.brush_shape, BrushShape::Round);
        assert_eq!(back.tool, Tool::Text);
        assert_eq!(back.rgba, [10, 20, 30, 128]);
        assert_eq!(back.color_history, vec![[1, 2, 3, 4], [5, 6, 7, 8]]);
    }

    /// Пустое хранилище — обычный первый запуск, должны быть дефолты.
    #[test]
    fn empty_storage_yields_defaults() {
        let p = Prefs::load(None);
        assert_eq!(p.show_top_panel, true);
        assert_eq!(p.show_timeline, true);
        assert_eq!(p.welcome_show_again, true);
        assert_eq!(p.tool, Tool::Brush);

        let p2 = Prefs::load(Some(&MemStore::default()));
        assert_eq!(p2.tool, Tool::Brush);
    }

    /// Файл от прошлой версии, где части полей ещё нет, читается частично,
    /// а не отбрасывается целиком.
    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let mut store = MemStore::default();
        store.set_string(PREFS_KEY, r#"(show_timeline: false, arrow_speed: 12.0)"#.into());
        let p = Prefs::load(Some(&store));
        assert!(!p.show_timeline, "что было в файле — сохраняется");
        assert_eq!(p.arrow_speed, 12.0);
        assert!(p.show_top_panel, "отсутствующее — из Default");
        assert_eq!(p.tool, Tool::Brush);
    }

    /// Нечитаемый файл не должен ронять запуск.
    #[test]
    fn corrupt_file_yields_defaults() {
        let mut store = MemStore::default();
        store.set_string(PREFS_KEY, "не RON {{{".into());
        let p = Prefs::load(Some(&store));
        assert_eq!(p.show_top_panel, true);
        assert_eq!(p.tool, Tool::Brush);
    }

    /// RON не проверяет диапазоны, а файл лежит в каталоге данных приложения
    /// и остаётся редактируемым руками.
    #[test]
    fn out_of_range_and_nan_values_are_sanitized() {
        let mut store = MemStore::default();
        store.set_string(
            PREFS_KEY,
            r#"(arrow_speed: 1e30, zoom_speed: -3.0, brush: 0.0, color_history: [])"#.into(),
        );
        let p = Prefs::load(Some(&store));
        assert!(p.arrow_speed.is_finite() && p.arrow_speed <= 100.0 && p.arrow_speed >= 1.0);
        assert!(p.zoom_speed >= 0.005 && p.zoom_speed <= 5.0);
        assert!(p.brush >= 1.0);
    }

    #[test]
    fn history_is_capped_like_in_the_eyedropper() {
        let mut store = MemStore::default();
        let long: Vec<[u8; 4]> = (0..99).map(|i| [i, i, i, 255]).collect();
        eframe::set_value(
            &mut store,
            PREFS_KEY,
            &Prefs { color_history: long, ..Default::default() },
        );
        assert_eq!(Prefs::load(Some(&store)).color_history.len(), 4);
    }
}