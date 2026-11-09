//! Falling-snow overlay for the canvas area.
//!
//! The flakes are white pixel-art squares with a frost pattern, embedded in
//! the binary like the toolbar icons. Their positions are normalized, so the
//! snowfall keeps the same density when the window is resized.

use eframe::egui::{self, Color32, Pos2, Rect, TextureHandle, Vec2};

use crate::ui::load_icon_texture;

const FLAKE_TEXTURE: &[u8] = include_bytes!("../../tex/snow/snowflake.png");
const TREE_TEXTURE: &[u8] = include_bytes!("../../tex/snow/new_year_tree.png");
/// Size of the festive tree in the lower-left corner, in screen pixels.
const TREE_SIZE: f32 = 64.0;
/// Gap between the tree and the canvas-area edges.
const TREE_PAD: f32 = 12.0;

/// One particle. `x` and `y` are fractions of the canvas area: `y < 0` means
/// the flake is still above the top edge.
#[derive(Clone, Copy)]
struct Flake {
    x: f32,
    y: f32,
    /// Screen heights per second.
    speed: f32,
    /// Horizontal sway amplitude in pixels.
    sway: f32,
    sway_speed: f32,
    phase: f32,
    size: f32,
    alpha: u8,
}

pub(crate) struct Snowfall {
    flakes: Vec<Flake>,
    tex: Option<TextureHandle>,
    tree_tex: Option<TextureHandle>,
    rng: u32,
}

impl Default for Snowfall {
    fn default() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u32)
            .unwrap_or(0x9e37_79b9);
        Self {
            flakes: Vec::new(),
            tex: None,
            tree_tex: None,
            rng: seed.max(1),
        }
    }
}

impl Snowfall {
    /// Advances and paints the snowfall. When disabled, the particles are
    /// dropped so re-enabling starts from a fresh, evenly filled screen.
    pub(crate) fn paint(&mut self, ui: &egui::Ui, area: Rect, enabled: bool) {
        if !enabled {
            self.flakes.clear();
            return;
        }
        if area.width() <= 0.0 || area.height() <= 0.0 {
            return;
        }

        // The overlay is animated independently of mouse movement.
        ui.ctx().request_repaint();

        let target = ((area.width() * area.height()) / 9000.0) as usize;
        let target = target.clamp(35, 220);
        while self.flakes.len() < target {
            let flake = self.new_flake(true);
            self.flakes.push(flake);
        }
        self.flakes.truncate(target);

        let dt = ui.input(|i| i.unstable_dt).clamp(0.0, 0.1);
        for flake in &mut self.flakes {
            flake.y += flake.speed * dt;
        }
        for i in 0..self.flakes.len() {
            if self.flakes[i].y > 1.08 {
                self.flakes[i] = self.new_flake(false);
            }
        }

        let time = ui.input(|i| i.time) as f32;
        let tex_id = self
            .tex
            .get_or_insert_with(|| load_icon_texture(ui, "snowflake", FLAKE_TEXTURE))
            .id();
        let tree_tex_id = self
            .tree_tex
            .get_or_insert_with(|| load_icon_texture(ui, "new_year_tree", TREE_TEXTURE))
            .id();
        let painter = ui.painter();
        let uv = Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0));

        for flake in &self.flakes {
            let x = area.left() + flake.x * area.width()
                + (time * flake.sway_speed + flake.phase).sin() * flake.sway;
            let y = area.top() + flake.y * area.height();
            let rect = Rect::from_center_size(Pos2::new(x, y), Vec2::splat(flake.size));
            if rect.intersects(area) {
                painter.image(
                    tex_id,
                    rect,
                    uv,
                    Color32::from_white_alpha(flake.alpha),
                );
            }
        }

        // Festive tree in the lower-left corner while the snowfall is on.
        let tree_rect = Rect::from_min_size(
            Pos2::new(area.left() + TREE_PAD, area.bottom() - TREE_SIZE - TREE_PAD),
            Vec2::splat(TREE_SIZE),
        );
        painter.image(tree_tex_id, tree_rect, uv, Color32::WHITE);
    }

    fn new_flake(&mut self, anywhere: bool) -> Flake {
        Flake {
            x: self.rand(),
            // New flakes enter from above the canvas instead of popping into view.
            y: if anywhere { self.rand() } else { -0.12 * self.rand() - 0.02 },
            speed: self.range(0.035, 0.09),
            sway: self.range(2.0, 12.0),
            sway_speed: self.range(0.3, 1.2),
            phase: self.range(0.0, std::f32::consts::TAU),
            size: self.range(7.0, 18.0),
            alpha: self.range(150.0, 245.0) as u8,
        }
    }

    fn rand(&mut self) -> f32 {
        // xorshift32 — enough for snow and keeps the crate dependency-free.
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.rng = x;
        (x >> 8) as f32 / 16_777_216.0
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.rand()
    }
}
