use eframe::egui::Color32;

// ── HSV → RGB ────────────────────────────────────────
// Converts Hue/Saturation/Value to Red/Green/Blue (each 0–255)
// h: 0–360, s: 0–255, v: 0–255
// returns (r, g, b) each 0–255
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    // Normalize S and V to 0..1
    let s = s / 255.0;
    let v = v / 255.0;
    // Color circle sector (0..5)
    let hi = (h / 60.0) as i32 % 6;
    let f = h / 60.0 - (hi as f32);
    // Auxiliary values
    let p = v * (1.0 - s);
    let q = v * (1.0 - s * f);
    let t = v * (1.0 - s * (1.0 - f));
    // Pick the color triple out of 6 permutations
    let (r, g, b) = match hi {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    // Map back to 0..255
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

// ── RGB → HSV ────────────────────────────────────────
// Reverse conversion (r, g, b: 0–255) → (h: 0–360, s: 0–255, v: 0–255)
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let rf = r as f32 / 255.0;
    let gf = g as f32 / 255.0;
    let bf = b as f32 / 255.0;
    let mx = rf.max(gf).max(bf);
    let mn = rf.min(gf).min(bf);
    let d = mx - mn;
    // Calculate hue based on position in color circle
    let h = if d == 0.0 {
        0.0
    } else if mx == rf {
        60.0 * ((gf - bf) / d % 6.0)
    } else if mx == gf {
        60.0 * ((bf - rf) / d + 2.0)
    } else {
        60.0 * ((rf - gf) / d + 4.0)
    };
    // Saturation = spread / maximum
    let s = if mx == 0.0 { 0.0 } else { d / mx * 255.0 };
    (if h < 0.0 { h + 360.0 } else { h }, s, mx * 255.0)
}

pub fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let r = (a.r() as f32 + (b.r() as f32 - a.r() as f32) * t) as u8;
    let g = (a.g() as f32 + (b.g() as f32 - a.g() as f32) * t) as u8;
    let bl = (a.b() as f32 + (b.b() as f32 - a.b() as f32) * t) as u8;
    let al = (a.a() as f32 + (b.a() as f32 - a.a() as f32) * t) as u8;
    Color32::from_rgba_unmultiplied(r, g, bl, al)
}
