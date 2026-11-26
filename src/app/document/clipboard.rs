//! Pixel clipboard shared by all canvases (tabs).
//!
//! It used to live inside `Document`, so pixels copied in one tab were
//! invisible in every other one. Keeping it on `PixeshApp` makes copy/paste
//! work between documents without committing a floating paste first.

use eframe::egui::Color32;

#[derive(Clone, Default)]
pub(crate) struct Clipboard {
    pub(crate) pixels: Option<Vec<Color32>>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

impl Clipboard {
    pub(crate) fn set(&mut self, pixels: Vec<Color32>, width: usize, height: usize) {
        self.pixels = Some(pixels);
        self.width = width;
        self.height = height;
    }

    pub(crate) fn is_empty(&self) -> bool {
        match &self.pixels {
            None => true,
            Some(px) => self.width == 0 || self.height == 0 || px.len() != self.width * self.height,
        }
    }
}
