use eframe::egui::Color32;

use crate::constants::NUM_PALETTE;

use super::PixeshApp;

/// Prefill for the Ctrl+K dialog: the current colour in the same format the
/// field accepts. Alpha is omitted unless it differs from fully opaque.
pub(crate) fn format_rgb_input(c: Color32) -> String {
    if c.a() == 255 {
        format!("{}, {}, {}", c.r(), c.g(), c.b())
    } else {
        format!("{}, {}, {}, {}", c.r(), c.g(), c.b(), c.a())
    }
}

/// Parses `r,g,b`, `r,g,b,a`, `#RRGGBB` or `#RRGGBBAA`. Values outside 0..=255
/// are rejected rather than silently clamped: a typo like `300` should not
/// quietly turn into white.
pub(crate) fn parse_rgb_input(s: &str) -> Option<Color32> {
    let s = s.trim();
    if let Some(hex) = s.strip_prefix('#') {
        let hex = hex.trim();
        if hex.len() != 6 && hex.len() != 8 {
            return None;
        }
        let mut c = [0u8; 4];
        for (dst, pair) in c.iter_mut().zip(hex.as_bytes().chunks(2)) {
            let pair = std::str::from_utf8(pair).ok()?;
            *dst = u8::from_str_radix(pair, 16).ok()?;
        }
        return Some(if hex.len() == 6 {
            Color32::from_rgb(c[0], c[1], c[2])
        } else {
            Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
        });
    }

    let parts: Vec<&str> = s.split(',').map(str::trim).collect();
    if parts.len() != 3 && parts.len() != 4 {
        return None;
    }
    let mut c = [0u8; 4];
    c[3] = 255;
    for (dst, part) in c.iter_mut().zip(parts.iter()) {
        let v: u16 = part.parse().ok()?;
        if v > 255 {
            return None;
        }
        *dst = v as u8;
    }
    Some(Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3]))
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_decimal_rgb_and_alpha() {
        assert_eq!(
            parse_rgb_input(" 12, 34, 56 "),
            Some(Color32::from_rgb(12, 34, 56))
        );
        assert_eq!(
            parse_rgb_input("1,2,3,4"),
            Some(Color32::from_rgba_unmultiplied(1, 2, 3, 4))
        );
    }

    #[test]
    fn parses_hex_with_and_without_alpha() {
        assert_eq!(
            parse_rgb_input("#ff8000"),
            Some(Color32::from_rgb(255, 128, 0))
        );
        assert_eq!(
            parse_rgb_input("#FF800080"),
            Some(Color32::from_rgba_unmultiplied(255, 128, 0, 128))
        );
    }

    #[test]
    fn rejects_bad_input() {
        for bad in ["", "1,2", "1,2,3,4,5", "256,0,0", "1,,3", "#fff", "#gggggg"] {
            assert_eq!(parse_rgb_input(bad), None, "input: {bad:?}");
        }
    }

    #[test]
    fn format_round_trips_through_parser() {
        let c = Color32::from_rgba_unmultiplied(7, 8, 9, 10);
        let text = format_rgb_input(c);
        assert_eq!(parse_rgb_input(&text), Some(c));
        assert_eq!(format_rgb_input(Color32::from_rgb(7, 8, 9)), "7, 8, 9");
    }

    #[test]
    fn live_color_input_does_not_touch_history() {
        let mut app = PixeshApp::new();
        app.color_history.push(Color32::RED);
        app.set_color(parse_rgb_input("10,20,30").unwrap());
        assert_eq!(app.color, Color32::from_rgb(10, 20, 30));
        assert_eq!(app.color_history, vec![Color32::RED]);
    }
}
