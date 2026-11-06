use std::sync::Arc;

use eframe::egui::{self, Color32};

use crate::app::effects;

use super::PixeshApp;

impl PixeshApp {
    pub(crate) fn dialog_open(&self) -> bool {
        self.show_resize || self.show_export
            || self.show_panels || self.show_settings || self.show_scale
            || self.show_quit_dialog || self.show_text || self.show_welcome
            || self.show_effects
    }

    /// Show a message in the status bar for a couple of seconds.
    pub(crate) fn hint(&mut self, msg: impl Into<String>) {
        self.status_hint = msg.into();
        self.status_hint_until = self.last_time + 2.5;
    }

    /// Applies the selected effect to the document layers.
    /// One call = one undo entry, and only if something actually changed.
    pub(crate) fn apply_effect_to(&mut self, tab: usize) {
        let doc = &mut self.docs[tab];
        let sel = doc.sel;
        let (w, h) = (doc.width, doc.height);
        let frames: Vec<usize> = if self.effect_all_frames {
            (0..doc.frames).collect()
        } else {
            vec![doc.active_frame]
        };
        let layers: Vec<usize> = if self.effect_all_layers {
            (0..doc.layers.len()).collect()
        } else {
            vec![doc.active_layer]
        };

        // compute the result on the side first — if nothing changed, pollute
        // neither the document state nor the undo stack
        let mut results: Vec<(usize, usize, Vec<Color32>)> = Vec::new();
        for &li in &layers {
            if li >= doc.layers.len() { continue; }
            for &f in &frames {
                let Some(cel) = doc.layers[li].cels.get(f) else { continue };
                if cel.len() != w * h { continue; }
                let mut buf = (**cel).clone();
                effects::apply(&mut buf, w, h, &self.effect, sel);
                if buf != **cel {
                    results.push((li, f, buf));
                }
            }
        }
        if results.is_empty() { return; }

        doc.push_undo();
        for (li, f, buf) in results {
            doc.layers[li].cels[f] = Arc::new(buf);
        }
        doc.canvas_dirty = true;
        doc.sel_tex = None;
        // the preview will be recalculated the next time the panel is opened
        self.effect_prev_sig.clear();
    }

    pub(crate) fn update_playback(&mut self, ctx: &egui::Context) {
        let t = ctx.input(|i| i.time);
        for doc in &mut self.docs {
            if doc.playing {
                let interval = 1.0 / (doc.fps as f64).max(0.1);
                if doc.last_play == 0.0 {
                    doc.last_play = t;
                } else if t - doc.last_play >= interval {
                    doc.last_play = t;
                    let next = (doc.active_frame + 1) % doc.frames;
                    doc.set_active_frame(next);
                    ctx.request_repaint();
                }
            } else {
                doc.last_play = 0.0;
            }
        }
    }
}
