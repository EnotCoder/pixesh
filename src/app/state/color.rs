use eframe::egui::Color32;

use crate::constants::NUM_PALETTE;

use super::PixeshApp;

impl PixeshApp {
    /// Writes the current colour into every representation at once: Color32,
    /// RGBA fields and HSV. They cannot be kept separately — the picker and the
    /// sliders write to the same fields, and any desync is visible to the user.
    pub(crate) fn set_color(&mut self, c: Color32) {
        self.color = c;
        self.rgb_r = c.r() as f32;
        self.rgb_g = c.g() as f32;
        self.rgb_b = c.b() as f32;
        self.rgb_a = c.a() as f32;
        let (h, s, v) = crate::color::rgb_to_hsv(c.r(), c.g(), c.b());
        self.hsv_h = h;
        self.hsv_s = s;
        self.hsv_v = v;
    }

    /// Adjust HSV and recompute RGB from it. `color`, `rgb_*` and `hsv_*` must
    /// stay consistent — otherwise the picker and the sliders drift apart.
    ///
    /// Hue wraps around instead of being clamped: the 0–360° range is a closed
    /// ring, and rotation must not stop at the boundary.
    pub(crate) fn bump_hsv(&mut self, dh: f32, ds: f32, dv: f32) {
        self.hsv_h = (self.hsv_h + dh).rem_euclid(360.0);
        self.hsv_s = (self.hsv_s + ds).clamp(0.0, 255.0);
        self.hsv_v = (self.hsv_v + dv).clamp(0.0, 255.0);
        let (r, g, b) = crate::color::hsv_to_rgb(self.hsv_h, self.hsv_s, self.hsv_v);
        self.rgb_r = r as f32;
        self.rgb_g = g as f32;
        self.rgb_b = b as f32;
        self.color = Color32::from_rgba_unmultiplied(r, g, b, self.rgb_a as u8);
    }

    /// Opacity is edited by a separate method: it is not part of HSV, so it
    /// needs no hue recalculation, and the colour must not change.
    pub(crate) fn bump_alpha(&mut self, d: f32) {
        self.rgb_a = (self.rgb_a + d).clamp(0.0, 255.0);
        self.color = Color32::from_rgba_unmultiplied(
            self.rgb_r as u8,
            self.rgb_g as u8,
            self.rgb_b as u8,
            self.rgb_a as u8,
        );
    }

    /// Keys 1–9: apply a ready-made colour from `NUM_PALETTE`. The index is
    /// zero-based, so the "1" key is `NUM_PALETTE[0]`.
    ///
    /// `set_color` propagates the colour to `color`, `rgb_*` and `hsv_*`, so
    /// the picker and the sliders show the new value immediately.
    pub(crate) fn pick_preset(&mut self, idx: usize) {
        let [r, g, b, a] = NUM_PALETTE[idx];
        self.set_color(Color32::from_rgba_unmultiplied(r, g, b, a));
        self.push_history(self.color);
    }

    /// Adds a colour to the history, removing duplicates and keeping the
    /// length capped. This logic used to live only in the eyedropper, and
    /// having it in two places is unsafe: a limit change would be forgotten
    /// in one of them.
    pub(crate) fn push_history(&mut self, c: Color32) {
        self.color_history.retain(|&x| x != c);
        self.color_history.push(c);
        if self.color_history.len() > 4 {
            self.color_history.remove(0);
        }
    }
}
