//! Панель эффектов: список эффектов из Pixelorama + их параметры + превью.

use eframe::egui::{self, Color32, ColorImage, Pos2, Rect, Sense, Stroke, Vec2};

use crate::app::effects::{self, EffectParams, PT};
use crate::app::{Document, PixeshApp};
use crate::color::lerp_color;
use crate::constants::*;

/// Максимальный размер картинки-превью (в пикселях холста).
const PREVIEW_MAX: usize = 64;

impl PixeshApp {
    pub(crate) fn ui_effects_panel(&mut self, ctx: &egui::Context) {
        if !self.show_effects {
            return;
        }
        let tab = self.active_tab;
        let esc = ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape));
        if esc {
            self.show_effects = false;
            self.tool = Tool::Brush;
        }

        let size = if self.mobile { Vec2::new(300.0, 520.0) } else { Vec2::new(520.0, 620.0) };
        let font = if self.mobile { 0.8 } else { 1.0 };
        let sz = 21.0 * font;

        egui::Area::new("effects_panel".into())
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                let p = ui.painter();
                p.rect_filled(rect, 0.0, PANEL);
                p.rect_stroke(rect, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);

                let pad = 10.0;
                let body = rect.shrink2(Vec2::splat(pad));
                let mut ui2 = ui.new_child(
                    egui::UiBuilder::new()
                        .layout(egui::Layout::top_down(egui::Align::Min))
                        .max_rect(body),
                );
                ui2.spacing_mut().item_spacing = Vec2::new(4.0, 5.0);

                // ── заголовок ──
                ui2.horizontal(|ui| {
                    ui.label(egui::RichText::new("Effects").size(28.0 * font).color(TEXT));
                    ui.label(
                        egui::RichText::new("ported from Pixelorama")
                            .size(15.0 * font)
                            .color(DIM),
                    );
                });
                ui2.add_space(2.0);

                // ── категории ──
                // Активную категорию читаем ВНУТРИ замыкания: клик по кнопке
                // меняет self.effect прямо здесь, и следующая строка обязана
                // видеть уже новый эффект, иначе кадр нарисует старую категорию.
                ui2.horizontal_wrapped(|ui| {
                    let active = self.effect.def().cat;
                    for cat in effects::CATS {
                        if seg_btn(ui, cat.name(), cat == active, font) {
                            let first = effects::EFFECTS
                                .iter()
                                .find(|d| d.cat == cat)
                                .map(|d| d.kind)
                                .unwrap();
                            self.effect = EffectParams::new(first);
                        }
                    }
                });
                ui2.add_space(2.0);

                // ── список эффектов + параметры ──
                // обе секции в одном скролл-блоке: так колесо не «перехватывается»
                // между двумя соседними ScrollArea
                let cur_cat = self.effect.def().cat;
                let names: Vec<(effects::EffectKind, &'static str)> = effects::EFFECTS
                    .iter()
                    .filter(|d| d.cat == cur_cat)
                    .map(|d| (d.kind, d.name))
                    .collect();
                let cur_kind = self.effect.kind;
                let scroll_h = if self.mobile { 250.0 } else { 300.0 };

                egui::ScrollArea::vertical()
                    .max_height(scroll_h)
                    .auto_shrink([true, false])
                    .show(&mut ui2, |ui| {
                        // список эффектов текущей категории
                        ui.horizontal_wrapped(|ui| {
                            for (kind, name) in names {
                                if seg_btn(ui, name, kind == cur_kind, font) {
                                    self.effect = EffectParams::new(kind);
                                }
                            }
                        });

                        ui.add_space(4.0);
                        line(ui, rect.width() - pad * 2.0);
                        ui.add_space(4.0);

                        // Описание параметров читаем заново: клик по списку выше
                        // уже мог сменить эффект в этом же кадре.
                        let param_defs = self.effect.def().params;
                        if param_defs.is_empty() {
                            ui.label(egui::RichText::new("no parameters").size(sz).color(DIM));
                        }
                        for pd in param_defs {
                            self.param_row(ui, pd, font);
                        }
                    });

                ui2.add_space(4.0);
                line(&mut ui2, rect.width() - pad * 2.0);

                // ── превью + область применения ──
                // кнопки внизу занимают ~50, остаток отдаём превью — так панель
                // не остаётся с пустым местом, а превью получается крупнее
                const BUTTONS_H: f32 = 52.0;
                let spare = ui2.available_height() - BUTTONS_H - 8.0;
                let prev_sz = (spare * font).clamp(56.0, 240.0);
                ui2.horizontal(|ui| {
                    let (r, _) = ui.allocate_exact_size(Vec2::splat(prev_sz), Sense::hover());
                    // шахматка
                    let cell = 8.0;
                    let p = ui.painter();
                    let cols = (r.width() / cell).ceil() as i32;
                    let rows = (r.height() / cell).ceil() as i32;
                    for cy in 0..rows {
                        for cx in 0..cols {
                            let c = if (cx + cy) % 2 == 0 { Color32::from_gray(200) } else { Color32::from_gray(180) };
                            p.rect_filled(
                                Rect::from_min_size(
                                    r.min + Vec2::new(cx as f32 * cell, cy as f32 * cell),
                                    Vec2::splat(cell),
                                ),
                                0.0,
                                c,
                            );
                        }
                    }
                    if let Some(tex) = &self.effect_prev_tex {
                        p.image(
                            tex.id(),
                            r,
                            Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                            Color32::WHITE,
                        );
                    }
                    p.rect_stroke(r, 0.0, Stroke::new(4.0, BORDER), egui::StrokeKind::Outside);

                    ui.vertical(|ui| {
                        let info = format!(
                            "{} x {}   L{}  F{}",
                            self.docs[tab].width,
                            self.docs[tab].height,
                            self.docs[tab].active_layer,
                            self.docs[tab].active_frame,
                        );
                        ui.label(egui::RichText::new(info).size(17.0 * font).color(DIM));
                        if self.docs[tab].sel.is_some() {
                            ui.label(
                                egui::RichText::new("selection only")
                                    .size(17.0 * font)
                                    .color(ACCENT),
                            );
                        }
                        ui.add_space(4.0);
                        check_row(ui, "All layers", self.effect_all_layers, font, |v| {
                            self.effect_all_layers = v;
                        });
                        check_row(ui, "All frames", self.effect_all_frames, font, |v| {
                            self.effect_all_frames = v;
                        });
                    });
                });

                // ── кнопки ──
                ui2.add_space(6.0);
                let bw = (rect.width() - pad * 2.0 - 8.0) / 3.0;
                ui2.horizontal(|ui| {
                    if crate::ui::btn_min_w(ui, "Reset", bw) {
                        self.effect = EffectParams::new(self.effect.kind);
                    }
                    if crate::ui::btn_min_w(ui, "Apply", bw) {
                        self.apply_effect_to(tab);
                    }
                    if crate::ui::btn_min_w(ui, "Close", bw) {
                        self.show_effects = false;
                        self.tool = Tool::Brush;
                    }
                });
            });

        // превью считаем после отрисовки — параметры к этому моменту актуальны
        self.update_effect_preview(ctx, tab);
    }

    /// Строка одного параметра: слайдер / чекбокс / сегменты / цвет.
    fn param_row(&mut self, ui: &mut egui::Ui, pd: &effects::ParamDef, font: f32) {
        // pd мог достаться от предыдущего эффекта (клик по списку приходит
        // в том же кадре) — тогда параметра с таким именем просто нет
        if !self.effect.has(pd.name) { return; }
        let sz = 21.0 * font;
        let h = 28.0 * font;
        let label_w = 120.0 * font;
        let slider_w = 165.0 * font;
        match pd.ty {
            PT::B => {
                let v = self.effect.flag(pd.name);
                check_row(ui, pd.name, v, font, |nv| self.effect.set_flag(pd.name, nv));
            }
            PT::C => {
                // строка 1: название + образец цвета
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(label_w, h),
                        egui::Label::new(egui::RichText::new(pd.name).size(sz).color(TEXT)),
                    );
                    let cur = self.effect.color(pd.name);
                    // клик по образцу — взять текущий цвет приложения
                    let (sw, resp) = ui.allocate_exact_size(Vec2::new(56.0, h), Sense::click());
                    let t_hov = ui.ctx().animate_bool(resp.id.with("hov"), resp.hovered());
                    let p = ui.painter();
                    if !resp.is_pointer_button_down_on() {
                        p.rect_filled(sw.translate(Vec2::new(0.0, 2.0)), 0.0, BORDER);
                    }
                    p.rect_filled(sw, 0.0, cur);
                    p.rect_stroke(
                        sw,
                        0.0,
                        Stroke::new(2.0, lerp_color(BORDER, Color32::WHITE, t_hov)),
                        egui::StrokeKind::Outside,
                    );
                    if resp.clicked() {
                        self.effect.set_color(pd.name, self.color);
                    }
                    resp.on_hover_text("Use the current brush color");
                });
                // строка 2: точные значения каналов (с отступом под название)
                ui.horizontal(|ui| {
                    ui.add_space(label_w);
                    let cur = self.effect.color(pd.name);
                    let mut rgba_now = [cur.r(), cur.g(), cur.b(), cur.a()];
                    for (i, ch) in ["R", "G", "B", "A"].iter().enumerate() {
                        ui.add_sized(
                            Vec2::new(20.0, h),
                            egui::Label::new(egui::RichText::new(*ch).size(sz).color(DIM)),
                        );
                        let mut v = rgba_now[i] as f32;
                        if ui
                            .add_sized(
                                Vec2::new(slider_w * 0.42, h),
                                egui::DragValue::new(&mut v).range(0..=255).speed(1.0),
                            )
                            .changed()
                        {
                            rgba_now[i] = v.round().clamp(0.0, 255.0) as u8;
                            self.effect
                                .set_color(pd.name, Color32::from_rgba_unmultiplied(
                                    rgba_now[0], rgba_now[1], rgba_now[2], rgba_now[3],
                                ));
                        }
                    }
                });
            }
            PT::E(labels) => {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(label_w, h),
                        egui::Label::new(egui::RichText::new(pd.name).size(sz).color(TEXT)),
                    );
                    let cur = self.effect.choice(pd.name).min(labels.len().saturating_sub(1));
                    ui.horizontal_wrapped(|ui| {
                        for (i, lab) in labels.iter().enumerate() {
                            if seg_btn(ui, lab, i == cur, font) {
                                self.effect.set_choice(pd.name, i);
                            }
                        }
                    });
                });
            }
            PT::I(lo, hi) => {
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(label_w, h),
                        egui::Label::new(egui::RichText::new(pd.name).size(sz).color(TEXT)),
                    );
                    let mut v = *self.effect.num_mut(pd.name) as f32;
                    let resp = ui.add_sized(
                        Vec2::new(slider_w, h),
                        egui::Slider::new(&mut v, lo as f32..=hi as f32)
                            .show_value(false)
                            .step_by(1.0),
                    );
                    if resp.changed() {
                        *self.effect.num_mut(pd.name) = v.round() as f64;
                    }
                    ui.add_sized(
                        Vec2::new(58.0 * font, h),
                        egui::Label::new(
                            egui::RichText::new(format!("{}", v.round() as i32))
                                .size(sz)
                                .color(ACCENT),
                        ),
                    );
                });
            }
            PT::F(lo, hi) => {
                let mut v = *self.effect.num_mut(pd.name) as f32;
                ui.horizontal(|ui| {
                    ui.add_sized(
                        Vec2::new(label_w, h),
                        egui::Label::new(egui::RichText::new(pd.name).size(sz).color(TEXT)),
                    );
                    let decimals = if (hi - lo) > 20.0 { 0 } else { 2 };
                    let resp = ui.add_sized(
                        Vec2::new(slider_w, h),
                        egui::Slider::new(&mut v, lo..=hi)
                            .show_value(false)
                            .step_by(0.01),
                    );
                    if resp.changed() {
                        *self.effect.num_mut(pd.name) = ((v * 100.0).round() / 100.0) as f64;
                    }
                    let txt = if decimals == 0 {
                        format!("{}", v.round() as i32)
                    } else {
                        format!("{:.2}", v)
                    };
                    ui.add_sized(
                        Vec2::new(58.0 * font, h),
                        egui::Label::new(egui::RichText::new(txt).size(sz).color(ACCENT)),
                    );
                });
            }
        }
    }

    /// Пересчитывает картинку-превью: берёт текущий слой, при необходимости
    /// уменьшает, применяет эффект и заливает результат в текстуру.
    fn update_effect_preview(&mut self, ctx: &egui::Context, tab: usize) {
        let (w, h) = (self.docs[tab].width, self.docs[tab].height);
        if w == 0 || h == 0 {
            return;
        }
        let sig = format!(
            "{}|{}x{}|l{}f{}",
            self.effect.signature(),
            w,
            h,
            self.docs[tab].active_layer,
            self.docs[tab].active_frame,
        );
        // Слой, который влезает в превью целиком, считаем каждый кадр — это
        // дёшево и превью всегда актуально. Крупный пересчитываем только при
        // смене параметров, но не реже раза в 0.4 с, иначе правки на холсте
        // (у них нет общего счётчика) не попадали бы в превью.
        let now = ctx.input(|i| i.time);
        let cheap = w * h <= PREVIEW_MAX * PREVIEW_MAX;
        let stale = sig != self.effect_prev_sig || now - self.effect_prev_time > 0.4;
        if !cheap && !stale {
            return;
        }
        self.effect_prev_sig = sig;
        self.effect_prev_time = now;

        let sel = self.docs[tab].sel;
        let (px, pw, ph) = preview_source(&self.docs[tab], PREVIEW_MAX);
        if pw == 0 || ph == 0 {
            return;
        }
        let mut buf = px;
        effects::apply(&mut buf, pw, ph, &self.effect, sel);

        if self.effect_prev_size != (pw, ph) {
            self.effect_prev_tex = None;
            self.effect_prev_size = (pw, ph);
        }
        self.effect_prev_px = buf;
        let ci = ColorImage::from_rgba_unmultiplied(
            [pw, ph],
            &rgba_bytes(&self.effect_prev_px),
        );
        self.effect_prev_tex =
            Some(ctx.load_texture("effect_prev", ci, egui::TextureOptions::NEAREST));
    }
}

/// Пиксели слоя, уменьшенные nearest-neighbour'ом до `max_dim` по большей стороне.
fn preview_source(doc: &Document, max_dim: usize) -> (Vec<Color32>, usize, usize) {
    let (w, h) = (doc.width, doc.height);
    let Some(cel) = doc.layers.get(doc.active_layer).and_then(|l| l.cels.get(doc.active_frame))
    else {
        return (Vec::new(), 0, 0);
    };
    let step = w.max(h).div_ceil(max_dim).max(1);
    let pw = (w / step).max(1);
    let ph = (h / step).max(1);
    let mut out = Vec::with_capacity(pw * ph);
    for y in 0..ph {
        for x in 0..pw {
            let i = (y * step * w + x * step).min(cel.len().saturating_sub(1));
            out.push(cel[i]);
        }
    }
    (out, pw, ph)
}

fn rgba_bytes(px: &[Color32]) -> Vec<u8> {
    let mut raw = Vec::with_capacity(px.len() * 4);
    for c in px {
        let s = c.to_srgba_unmultiplied();
        raw.extend_from_slice(&s);
    }
    raw
}

// ── маленькие виджеты в стиле остальной панели ────────

fn line(ui: &mut egui::Ui, w: f32) {
    let (r, _) = ui.allocate_exact_size(Vec2::new(w, 2.0), Sense::hover());
    ui.painter().hline(r.x_range(), r.center().y, Stroke::new(2.0, BORDER));
}

/// Компактная кнопка-переключатель (категория / эффект / вариант).
fn seg_btn(ui: &mut egui::Ui, label: &str, active: bool, font: f32) -> bool {
    let sz = 20.0 * font;
    let w = label.len() as f32 * CHAR_W * (sz / FONT_SZ) + 20.0;
    let h = 30.0 * font;
    let (rect, resp) = ui.allocate_exact_size(Vec2::new(w, h), Sense::click());
    let t_hover = ui.ctx().animate_bool(resp.id.with("hov"), resp.hovered());
    let t_act = ui.ctx().animate_bool(resp.id.with("act"), active);
    let mut bg = PANEL_LIGHT;
    bg = lerp_color(bg, HOVER, t_hover);
    bg = lerp_color(bg, ACCENT, t_act);
    let p = ui.painter();
    if !resp.is_pointer_button_down_on() {
        p.rect_filled(rect.translate(Vec2::new(0.0, 2.0)), 0.0, BORDER);
    }
    p.rect_filled(rect, 0.0, bg);
    p.rect_stroke(rect, 0.0, Stroke::new(2.0, BORDER), egui::StrokeKind::Inside);
    p.text(rect.center(), egui::Align2::CENTER_CENTER, label, egui::FontId::proportional(sz), TEXT);
    resp.clicked()
}

/// Строка-чекбокс в стиле панели слоёв.
fn check_row(ui: &mut egui::Ui, label: &str, value: bool, font: f32, mut set: impl FnMut(bool)) {
    let sz = 19.0 * font;
    let row_h = sz + 12.0;
    let w = ui.available_width().min(170.0);
    let (row, _) = ui.allocate_exact_size(Vec2::new(w, row_h), Sense::hover());
    let cbs = 18.0;
    let cb_rect = Rect::from_min_size(
        Pos2::new(row.min.x, row.center().y - cbs * 0.5),
        Vec2::splat(cbs),
    );
    let p = ui.painter();
    p.text(
        Pos2::new(cb_rect.max.x + 8.0, row.center().y),
        egui::Align2::LEFT_CENTER,
        label,
        egui::FontId::proportional(sz),
        TEXT,
    );
    p.rect_filled(cb_rect, 0.0, PANEL);
    p.rect_stroke(cb_rect, 0.0, Stroke::new(3.0, BORDER), egui::StrokeKind::Outside);
    if value {
        p.rect_filled(cb_rect.shrink(4.0), 0.0, ACCENT);
    }
    if ui.interact(row, egui::Id::new(("fx_cb", label)), Sense::click()).clicked()
        || ui.interact(cb_rect, egui::Id::new(("fx_cbi", label)), Sense::click()).clicked()
    {
        set(!value);
    }
}
