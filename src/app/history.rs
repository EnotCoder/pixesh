use super::{Document, Snapshot};

impl Document {
    pub(crate) fn push_undo(&mut self) {
        self.unsaved = true;
        self.undo_stack.push(Snapshot {
            layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
            active: self.active_layer,
            width: self.width,
            height: self.height,
        });
        self.redo_stack.clear();
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    pub(crate) fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
                width: self.width,
                height: self.height,
            });
            self.width = state.width;
            self.height = state.height;
            for (i, layer) in self.layers.iter_mut().enumerate() {
                if i < state.layers.len() {
                    layer.pixels = state.layers[i].clone();
                }
            }
            self.active_layer = state.active.min(self.layers.len().saturating_sub(1));
            self.tex = None;
            self.canvas_dirty = true;
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
                width: self.width,
                height: self.height,
            });
            self.width = state.width;
            self.height = state.height;
            for (i, layer) in self.layers.iter_mut().enumerate() {
                if i < state.layers.len() {
                    layer.pixels = state.layers[i].clone();
                }
            }
            self.active_layer = state.active.min(self.layers.len().saturating_sub(1));
            self.tex = None;
            self.canvas_dirty = true;
        }
    }
}
