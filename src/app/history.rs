use super::{PixeshApp, Snapshot};

impl PixeshApp {
    // сохранить текущее состояние в undo-стек (снапшот пикселей всех слоёв)
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

    // отменить последнее действие — восстановить снапшот из undo-стека
    pub(crate) fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
            });
            for (i, layer) in self.layers.iter_mut().enumerate() {
                if i < state.layers.len() {
                    layer.pixels = state.layers[i].clone();
                }
            }
            self.active_layer = state.active;
            self.canvas_dirty = true;
        }
    }

    // повторить отменённое действие — восстановить снапшот из redo-стека
    pub(crate) fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| l.pixels.clone()).collect(),
                active: self.active_layer,
            });
            for (i, layer) in self.layers.iter_mut().enumerate() {
                if i < state.layers.len() {
                    layer.pixels = state.layers[i].clone();
                }
            }
            self.active_layer = state.active;
            self.canvas_dirty = true;
        }
    }
}
