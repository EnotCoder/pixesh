use super::{PixeshApp, Snapshot};

impl PixeshApp {
    pub(crate) fn push_undo(&mut self) {
        self.undo_stack.push(Snapshot {
            layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
            active: self.active_layer,
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
            });
            for (i, p) in state.layers.into_iter().enumerate() {
                if i < self.layers.len() {
                    self.layers[i].pixels = p;
                }
            }
            self.active_layer = state.active;
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
            });
            for (i, p) in state.layers.into_iter().enumerate() {
                if i < self.layers.len() {
                    self.layers[i].pixels = p;
                }
            }
            self.active_layer = state.active;
        }
    }
}
