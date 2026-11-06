use eframe::egui::{self, Color32, Rect, Stroke, Vec2};

use crate::app::document::Document;
use crate::constants::*;

use super::PixeshApp;

impl PixeshApp {
    /// Opens an image in a NEW tab. The shared entry point for Ctrl+L and for
    /// dropping a file onto the window, so loading does not depend on rfd.
    /// Returns the file name if it was opened successfully.
    pub(crate) fn open_path(&mut self, path: &std::path::Path) -> Option<String> {
        let path_str = path.to_string_lossy().into_owned();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Untitled".into());

        let mut doc = Document::new(&name);
        match doc.load_png(&path_str) {
            Ok(()) => {
                self.docs.push(doc);
                self.active_tab = self.docs.len() - 1;
                Some(name)
            }
            Err(e) => {
                // do not create a tab: an empty tab named after the file only confuses
                self.hint(format!("{}: {}", name, e));
                None
            }
        }
    }

    pub(crate) fn any_unsaved(&self) -> bool {
        self.docs.iter().any(|d| d.unsaved)
    }

    /// Dragging files from the file manager into the window.
    /// Each file opens in its own tab, and the last one becomes active.
    /// The highlight during dragging itself is in draw_drop_overlay.
    pub(crate) fn handle_dropped_files(&mut self, ctx: &egui::Context) {
        let dropped = ctx.input(|i| i.raw.dropped_files.clone());
        if dropped.is_empty() { return; }
        let mut opened = 0usize;
        let mut failed = 0usize;
        for file in &dropped {
            // path is optional: "virtual" files (for example from an embedded
            // browser) have none, and there is nothing to open for them
            let Some(path) = file.path.as_ref() else {
                failed += 1;
                continue;
            };
            match self.open_path(path) {
                Some(_) => opened += 1,
                None => failed += 1,
            }
        }
        if opened > 0 {
            if opened == 1 {
                self.hint("image opened");
            } else {
                self.hint(format!("{} images opened", opened));
            }
        } else if failed > 1 {
            // for a single file open_path has already reported the specific reason
            self.hint(format!("{} files could not be opened", failed));
        }
    }

    /// A "release to open" frame over the whole window while a file is above it.
    pub(super) fn draw_drop_overlay(&self, ctx: &egui::Context) {
        let hovered = ctx.input(|i| !i.raw.hovered_files.is_empty());
        if !hovered { return; }
        let screen = ctx.screen_rect();
        let p = ctx.layer_painter(egui::LayerId::new(egui::Order::Tooltip, egui::Id::new("drop_overlay")));
        p.rect_filled(
            screen,
            0.0,
            Color32::from_black_alpha(140),
        );
        let inset = 8.0;
        let r = Rect::from_min_max(screen.min + Vec2::splat(inset), screen.max - Vec2::splat(inset));
        p.rect_stroke(r, 0.0, Stroke::new(6.0_f32, ACCENT), egui::StrokeKind::Inside);
        let label = match ctx.input(|i| i.raw.hovered_files.len()) {
            1 => "Open image".to_owned(),
            n => format!("Open {} images", n),
        };
        p.text(
            r.center(),
            egui::Align2::CENTER_CENTER,
            label,
            egui::FontId::proportional(32.0),
            ACCENT,
        );
    }

    pub(crate) fn close_tab(&mut self, idx: usize) {
        if self.docs.len() <= 1 {
            self.docs[0] = Document::new_logo("Untitled");
            self.active_tab = 0;
            return;
        }
        self.docs.remove(idx);
        if self.active_tab >= self.docs.len() {
            self.active_tab = self.docs.len() - 1;
        } else if self.active_tab > idx {
            self.active_tab -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_png(name: &str, w: u32, h: u32) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join("pixesh_open_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let img = image::RgbaImage::from_pixel(w, h, image::Rgba([1, 2, 3, 255]));
        let path = dir.join(name);
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn open_path_adds_a_tab_and_activates_it() {
        let path = tmp_png("first.png", 8, 5);
        let mut app = PixeshApp::new();
        let tabs_before = app.docs.len();

        let opened = app.open_path(&path).expect("файл должен открыться");
        assert_eq!(opened, "first.png");
        assert_eq!(app.docs.len(), tabs_before + 1, "появилась новая вкладка");
        assert_eq!(app.active_tab, app.docs.len() - 1, "новая вкладка активна");

        let doc = &app.docs[app.active_tab];
        assert_eq!((doc.width, doc.height), (8, 5), "размеры взяты из картинки");
        assert_eq!(doc.name, "first.png", "имя вкладки — имя файла");
        assert!(!doc.unsaved, "только что открытый файл не считается изменённым");
    }

    #[test]
    fn open_path_keeps_previous_tabs_untouched() {
        let mut app = PixeshApp::new();
        app.docs[0].name = "Untitled".into();
        app.docs[0].width = 32;
        app.docs[0].height = 32;

        app.open_path(&tmp_png("second.png", 7, 9));
        assert_eq!(app.docs[0].name, "Untitled");
        assert_eq!((app.docs[0].width, app.docs[0].height), (32, 32),
                   "прежний документ не тронут — это отдельная вкладка");
    }

    #[test]
    fn open_path_rejects_non_image_without_adding_a_tab() {
        let dir = std::env::temp_dir().join("pixesh_open_tests");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("readme.txt");
        std::fs::write(&path, b"not an image").unwrap();

        let mut app = PixeshApp::new();
        let tabs_before = app.docs.len();
        assert!(app.open_path(&path).is_none(), "не-картинка не открывается");
        assert_eq!(app.docs.len(), tabs_before, "пустую вкладку не создаём");
        assert!(!app.status_hint.is_empty(), "пользователь должен узнать о проблеме");
    }
}
