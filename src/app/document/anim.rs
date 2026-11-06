use std::sync::Arc;

use eframe::egui::Color32;

use super::Document;

impl Document {
    // composite a specific animation frame into a flat RGBA buffer
    pub(crate) fn composite_at(&self, frame: usize) -> Vec<Color32> {
        let mut out = vec![Color32::TRANSPARENT; self.width * self.height];
        if self.layers.is_empty() || frame >= self.frames {
            return out;
        }
        for layer in self.layers.iter().rev() {
            if !layer.visible { continue; }
            let cel = &layer.cels[frame];
            for (i, &p) in cel.iter().enumerate() {
                if p != Color32::TRANSPARENT { out[i] = p; }
            }
        }
        out
    }

    pub(crate) fn set_active_frame(&mut self, f: usize) {
        let f = f.min(self.frames.saturating_sub(1));
        if f == self.active_frame { return; }
        self.active_frame = f;
        self.canvas_dirty = true;
        self.tex = None;
    }

    // append a blank frame after the current one
    pub(crate) fn add_frame_blank(&mut self) {
        self.push_undo();
        for layer in &mut self.layers {
            layer.cels.insert(
                self.active_frame + 1,
                Arc::new(vec![Color32::TRANSPARENT; self.width * self.height]),
            );
        }
        self.frames += 1;
        self.active_frame += 1;
        self.canvas_dirty = true;
        self.tex = None;
    }

    // duplicate the current frame
    pub(crate) fn duplicate_frame(&mut self) {
        self.push_undo();
        for layer in &mut self.layers {
            let cel = layer.cels[self.active_frame].clone();
            layer.cels.insert(self.active_frame + 1, cel);
        }
        self.frames += 1;
        self.active_frame += 1;
        self.canvas_dirty = true;
        self.tex = None;
    }

    pub(crate) fn delete_frame(&mut self) {
        if self.frames <= 1 { return; }
        self.push_undo();
        for layer in &mut self.layers {
            layer.cels.remove(self.active_frame);
        }
        self.frames -= 1;
        if self.active_frame >= self.frames {
            self.active_frame = self.frames - 1;
        }
        self.canvas_dirty = true;
        self.tex = None;
    }

    pub(crate) fn move_frame(&mut self, from: usize, to: usize) {
        if from == to || from >= self.frames || to >= self.frames { return; }
        self.push_undo();
        for layer in &mut self.layers {
            let cel = layer.cels.remove(from);
            layer.cels.insert(to, cel);
        }
        self.active_frame = to;
        self.canvas_dirty = true;
        self.tex = None;
    }
}
