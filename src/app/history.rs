use super::{Document, Snapshot, SnapshotLayer};

impl Document {
    pub(crate) fn push_undo(&mut self) {
        self.unsaved = true;
        self.undo_stack.push(Snapshot {
            layers: self.layers.iter().map(|l| SnapshotLayer {
                name: l.name.clone(),
                pixels: l.pixels.clone(),
                visible: l.visible,
            }).collect(),
            active: self.active_layer,
            width: self.width,
            height: self.height,
            sel: self.sel,
            sel_buffer: self.sel_buffer.clone(),
            sel_buf_w: self.sel_buf_w,
            sel_buf_h: self.sel_buf_h,
            pasting: self.pasting,
            clipboard: self.clipboard.clone(),
            clip_w: self.clip_w,
            clip_h: self.clip_h,
        });
        self.redo_stack.clear();
        if self.undo_stack.len() > 50 {
            self.undo_stack.remove(0);
        }
    }

    pub(crate) fn undo(&mut self) {
        if let Some(state) = self.undo_stack.pop() {
            self.redo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| SnapshotLayer {
                    name: l.name.clone(),
                    pixels: l.pixels.clone(),
                    visible: l.visible,
                }).collect(),
                active: self.active_layer,
                width: self.width,
                height: self.height,
                sel: self.sel,
                sel_buffer: self.sel_buffer.clone(),
                sel_buf_w: self.sel_buf_w,
                sel_buf_h: self.sel_buf_h,
                pasting: self.pasting,
                clipboard: self.clipboard.clone(),
                clip_w: self.clip_w,
                clip_h: self.clip_h,
            });
            self.restore_snapshot(state);
        }
    }

    pub(crate) fn redo(&mut self) {
        if let Some(state) = self.redo_stack.pop() {
            self.undo_stack.push(Snapshot {
                layers: self.layers.iter().map(|l| SnapshotLayer {
                    name: l.name.clone(),
                    pixels: l.pixels.clone(),
                    visible: l.visible,
                }).collect(),
                active: self.active_layer,
                width: self.width,
                height: self.height,
                sel: self.sel,
                sel_buffer: self.sel_buffer.clone(),
                sel_buf_w: self.sel_buf_w,
                sel_buf_h: self.sel_buf_h,
                pasting: self.pasting,
                clipboard: self.clipboard.clone(),
                clip_w: self.clip_w,
                clip_h: self.clip_h,
            });
            self.restore_snapshot(state);
        }
    }

    fn restore_snapshot(&mut self, state: super::Snapshot) {
        use super::Layer;
        self.width = state.width;
        self.height = state.height;
        self.layers = state.layers.into_iter().map(|sl| Layer {
            name: sl.name,
            pixels: sl.pixels,
            visible: sl.visible,
        }).collect();
        self.active_layer = state.active.min(self.layers.len().saturating_sub(1));
        self.sel = state.sel;
        self.sel_buffer = state.sel_buffer;
        self.sel_buf_w = state.sel_buf_w;
        self.sel_buf_h = state.sel_buf_h;
        self.pasting = state.pasting;
        self.clipboard = state.clipboard;
        self.clip_w = state.clip_w;
        self.clip_h = state.clip_h;
        self.sel_tex = None;
        self.sel_move_origin = None;
        self.sel_move_current = None;
        self.sel_start = None;
        self.sel_end = None;
        self.tex = None;
        self.canvas_dirty = true;
    }
}
