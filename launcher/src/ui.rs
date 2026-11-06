//! Look and widgets of the launcher. A port of the arelkair/wiiparty-recomp launcher UI (launcher/src/ui.cpp,
//! layout.h, titlebar.cpp; GPL-3.0) from Dear ImGui to egui: same palette, Inter type, sizes, pill buttons,
//! toggles, dropdowns, sidebar navigation, cards and the pink die logo.
use eframe::egui::{
    self, epaint::Vertex, pos2, vec2, Align2, Color32, CornerRadius, CursorIcon, FontFamily, FontId, Id, Mesh, Pos2, Rect, Response,
    Sense, Shape, Stroke, StrokeKind, Ui, UiBuilder, Vec2, WidgetText,
};
use std::sync::Arc;

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Palette {
    pub background: Color32,
    pub sidebar: Color32,
    pub surface: Color32,
    pub text: Color32,
    pub secondary: Color32,
    pub tertiary: Color32,
    pub separator: Color32,
    pub control: Color32,
    pub control_hover: Color32,
    pub control_active: Color32,
    pub primary: Color32,
    pub primary_hover: Color32,
    pub primary_text: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub danger: Color32,
    pub success: Color32,
    pub overlay: Color32,
    pub popup: Color32,
}

const fn c(r: u8, g: u8, b: u8) -> Color32 { Color32::from_rgb(r, g, b) }
const fn ca(r: u8, g: u8, b: u8, a: u8) -> Color32 { Color32::from_rgba_unmultiplied_const(r, g, b, a) }

pub const LIGHT: Palette = Palette {
    background: c(255, 255, 255), sidebar: c(245, 245, 247), surface: c(245, 245, 247), text: c(29, 29, 31),
    secondary: c(110, 110, 115), tertiary: c(174, 174, 178), separator: c(229, 229, 234), control: c(237, 237, 240),
    control_hover: c(227, 227, 232), control_active: c(214, 214, 220), primary: c(29, 29, 31), primary_hover: c(58, 58, 60),
    primary_text: c(255, 255, 255), accent: c(224, 40, 125), accent_soft: ca(224, 40, 125, 36), danger: c(215, 0, 21),
    success: c(36, 138, 61), overlay: ca(0, 0, 0, 72), popup: c(255, 255, 255),
};

pub const DARK: Palette = Palette {
    background: c(18, 18, 20), sidebar: c(26, 26, 29), surface: c(28, 28, 31), text: c(245, 245, 247),
    secondary: c(161, 161, 166), tertiary: c(99, 99, 102), separator: c(44, 44, 48), control: c(38, 38, 42),
    control_hover: c(48, 48, 53), control_active: c(58, 58, 64), primary: c(245, 245, 247), primary_hover: c(255, 255, 255),
    primary_text: c(18, 18, 20), accent: c(255, 79, 154), accent_soft: ca(255, 79, 154, 40), danger: c(255, 69, 58),
    success: c(48, 209, 88), overlay: ca(0, 0, 0, 128), popup: c(36, 36, 40),
};

pub fn palette(ctx: &egui::Context) -> Palette {
    ctx.data(|d| d.get_temp::<bool>(Id::new("wp-dark"))).map(|dark| if dark { DARK } else { LIGHT }).unwrap_or(LIGHT)
}

pub mod size {
    pub const DISPLAY: f32 = 40.0;
    pub const HEADING: f32 = 28.0;
    pub const LEAD: f32 = 15.0;
    pub const BODY: f32 = 14.0;
    pub const DETAIL: f32 = 12.5;
    pub const SMALL: f32 = 11.5;
}

#[derive(Clone, Copy, PartialEq)]
pub enum Font { Regular, Semibold }
#[derive(Clone, Copy, PartialEq)]
pub enum Kind { Primary, Secondary, Ghost }

pub fn font(f: Font, size: f32) -> FontId {
    FontId::new(size, match f { Font::Regular => FontFamily::Proportional, Font::Semibold => FontFamily::Name("semibold".into()) })
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert("inter".into(), Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-Regular.ttf"))));
    fonts.font_data.insert("inter-semibold".into(), Arc::new(egui::FontData::from_static(include_bytes!("../assets/fonts/Inter-SemiBold.ttf"))));
    let fallback = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    let mut regular = vec!["inter".to_string()];
    regular.extend(fallback.iter().cloned());
    let mut semibold = vec!["inter-semibold".to_string()];
    semibold.extend(fallback);
    fonts.families.insert(FontFamily::Proportional, regular);
    fonts.families.insert(FontFamily::Name("semibold".into()), semibold);
    ctx.set_fonts(fonts);
}

pub fn apply_style(ctx: &egui::Context, dark: bool) {
    ctx.data_mut(|d| d.insert_temp(Id::new("wp-dark"), dark));
    let p = if dark { DARK } else { LIGHT };
    let mut v = if dark { egui::Visuals::dark() } else { egui::Visuals::light() };
    v.panel_fill = p.background;
    v.window_fill = p.popup;
    v.extreme_bg_color = p.surface;
    v.faint_bg_color = p.surface;
    v.override_text_color = Some(p.text);
    v.selection.bg_fill = p.accent_soft;
    v.selection.stroke = Stroke::new(1.0_f32, p.accent);
    v.hyperlink_color = p.accent;
    v.window_stroke = Stroke::new(1.0_f32, p.separator);
    v.window_corner_radius = CornerRadius::same(12);
    v.menu_corner_radius = CornerRadius::same(12);
    v.popup_shadow = egui::epaint::Shadow { offset: [0, 6], blur: 24, spread: 0, color: ca(0, 0, 0, if dark { 90 } else { 28 }) };
    v.widgets.noninteractive.bg_stroke = Stroke::new(1.0_f32, p.separator);
    for w in [&mut v.widgets.inactive, &mut v.widgets.hovered, &mut v.widgets.active] { w.corner_radius = CornerRadius::same(10); }
    ctx.set_visuals(v);
    ctx.style_mut(|s| {
        s.spacing.item_spacing = Vec2::ZERO;
        s.spacing.button_padding = Vec2::ZERO;
        s.spacing.scroll = egui::style::ScrollStyle { bar_width: 10.0, floating: true, ..egui::style::ScrollStyle::floating() };
        s.interaction.selectable_labels = false;
        s.text_styles.insert(egui::TextStyle::Body, font(Font::Regular, size::BODY));
        s.text_styles.insert(egui::TextStyle::Button, font(Font::Regular, size::BODY));
        s.text_styles.insert(egui::TextStyle::Monospace, FontId::monospace(size::DETAIL));
    });
}

// ---------- small helpers ----------
pub fn mix(a: Color32, b: Color32, t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t).round() as u8;
    let (a, b) = (a.to_srgba_unmultiplied(), b.to_srgba_unmultiplied());
    Color32::from_rgba_unmultiplied(l(a[0], b[0]), l(a[1], b[1]), l(a[2], b[2]), l(a[3], b[3]))
}
pub fn fade(col: Color32, alpha: f32) -> Color32 {
    let [r, g, b, a] = col.to_srgba_unmultiplied();
    Color32::from_rgba_unmultiplied(r, g, b, (a as f32 * alpha.clamp(0.0, 1.0)).round() as u8)
}
pub fn ease_out(t: f32) -> f32 { let t = t.clamp(0.0, 1.0); 1.0 - (1.0 - t).powi(3) }
/// Smoothly follow `target` (exponential approach, like ui::animate in the C++ launcher).
pub fn animate(ctx: &egui::Context, id: Id, target: f32, seconds: f32) -> f32 { ctx.animate_value_with_time(id, target, seconds) }
fn anim_bool(ctx: &egui::Context, id: Id, on: bool, seconds: f32) -> f32 { ctx.animate_bool_with_time(id, on, seconds) }

pub fn gap(ui: &mut Ui, h: f32) { ui.add_space(h); }

/// A text block; `wrap` limits its width.
pub fn text(ui: &mut Ui, s: &str, f: Font, size: f32, color: Color32, wrap: Option<f32>) -> Response {
    let rt = egui::RichText::new(s).font(font(f, size)).color(color);
    match wrap {
        Some(w) => ui.scope(|ui| { ui.set_max_width(w); ui.add(egui::Label::new(rt).wrap()) }).inner,
        None => ui.add(egui::Label::new(rt).extend()),
    }
}
pub fn text_size(ui: &Ui, s: &str, f: Font, size: f32, wrap: Option<f32>) -> Vec2 {
    let g = ui.painter().layout(s.to_string(), font(f, size), Color32::WHITE, wrap.unwrap_or(f32::INFINITY));
    g.size()
}
pub fn paint_text(ui: &Ui, at: Pos2, anchor: Align2, s: &str, f: Font, size: f32, color: Color32) -> Rect {
    ui.painter().text(at, anchor, s, font(f, size), color)
}

/// Shorten from the start with an ellipsis so the end of a path stays visible.
pub fn fit_start(ui: &Ui, s: &str, f: Font, size: f32, width: f32) -> String {
    if text_size(ui, s, f, size, None).x <= width { return s.to_string(); }
    let chars: Vec<char> = s.chars().collect();
    for cut in 1..chars.len() {
        let shown: String = std::iter::once('…').chain(chars[cut..].iter().copied()).collect();
        if text_size(ui, &shown, f, size, None).x <= width { return shown; }
    }
    "…".into()
}

fn press_shrink(rect: Rect, press: f32) -> Rect { rect.shrink2(rect.size() * 0.015 * press) }

// ---------- controls ----------
pub fn button(ui: &mut Ui, label: &str, kind: Kind, width: f32, enabled: bool, height: f32) -> bool {
    let p = palette(ui.ctx());
    let fsize = if height >= 44.0 { 16.0 } else { size::BODY };
    let lsize = text_size(ui, label, Font::Semibold, fsize, None);
    let (rect, resp) = ui.allocate_exact_size(vec2(width.max(lsize.x + 40.0), height), if enabled { Sense::click() } else { Sense::hover() });
    let hovered = enabled && resp.hovered();
    if hovered { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
    let hover = anim_bool(ui.ctx(), resp.id.with("h"), hovered, 0.08);
    let press = anim_bool(ui.ctx(), resp.id.with("p"), enabled && resp.is_pointer_button_down_on(), 0.05);
    let (fill, ink) = match kind {
        Kind::Primary => (mix(p.primary, p.primary_hover, hover), p.primary_text),
        Kind::Secondary => (mix(p.control, p.control_hover, hover), p.text),
        Kind::Ghost => (fade(p.control, hover), p.text),
    };
    let alpha = if enabled { 1.0 } else { 0.4 };
    let shown = press_shrink(rect, press);
    ui.painter().rect_filled(shown, shown.height() * 0.5, fade(fill, alpha));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, label, font(Font::Semibold, fsize), fade(ink, alpha));
    enabled && resp.clicked()
}

pub fn toggle(ui: &mut Ui, value: &mut bool) -> bool {
    let p = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(42.0, 24.0), Sense::click());
    if resp.hovered() { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
    if resp.clicked() { *value = !*value; }
    let on = ease_out(anim_bool(ui.ctx(), resp.id.with("on"), *value, 0.14));
    let hover = anim_bool(ui.ctx(), resp.id.with("h"), resp.hovered(), 0.08);
    let off_track = mix(p.control_active, p.tertiary, hover * 0.35);
    ui.painter().rect_filled(rect, 12.0, mix(off_track, p.primary, on));
    let radius = 12.0 - 2.0;
    let center = pos2(rect.min.x + 12.0 + (42.0 - 24.0) * on, rect.center().y);
    ui.painter().circle_filled(center + vec2(0.0, 1.0), radius, ca(0, 0, 0, 30));
    let knob_off = if p.background == DARK.background { p.text } else { Color32::WHITE };
    ui.painter().circle_filled(center, radius, mix(knob_off, p.background, on));
    resp.clicked()
}

pub fn chevron(ui: &Ui, center: Pos2, s: f32, ink: Color32, down: bool) {
    let h = s * 0.5;
    let (a, b, c) = if down { (vec2(-h, -h * 0.5), vec2(0.0, h * 0.5), vec2(h, -h * 0.5)) } else { (vec2(-h, h * 0.5), vec2(0.0, -h * 0.5), vec2(h, h * 0.5)) };
    ui.painter().add(Shape::line(vec![center + a, center + b, center + c], Stroke::new(1.6_f32, ink)));
}

pub fn dropdown(ui: &mut Ui, id: &str, labels: &[String], index: &mut usize, width: f32) -> bool {
    let p = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    let popup_id = ui.make_persistent_id(id).with("popup");
    let open = egui::Popup::is_id_open(ui.ctx(), popup_id);
    if resp.hovered() { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
    let hover = anim_bool(ui.ctx(), resp.id.with("h"), resp.hovered() || open, 0.08);
    ui.painter().rect_filled(rect, 10.0, mix(p.control, p.control_hover, hover));
    let current = labels.get(*index).map(String::as_str).unwrap_or("");
    let clip = ui.painter().with_clip_rect(rect.shrink2(vec2(8.0, 0.0)).intersect(ui.clip_rect()));
    clip.text(pos2(rect.min.x + 14.0, rect.center().y), Align2::LEFT_CENTER, current, font(Font::Regular, size::BODY), p.text);
    chevron(ui, pos2(rect.max.x - 18.0, rect.center().y), 8.0, p.secondary, true);
    let mut changed = false;
    let frame = egui::Frame::new().fill(p.popup).corner_radius(12).inner_margin(6).stroke(Stroke::new(1.0_f32, p.separator))
        .shadow(egui::epaint::Shadow { offset: [0, 6], blur: 24, spread: 0, color: ca(0, 0, 0, 40) });
    egui::Popup::from_toggle_button_response(&resp).id(popup_id).gap(6.0).width(width).frame(frame)
        .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
        .show(|ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            egui::ScrollArea::vertical().max_height(320.0).show(ui, |ui| {
                for (i, l) in labels.iter().enumerate() {
                    if list_item(ui, l, i == *index, width - 12.0) {
                        *index = i; changed = true;
                        egui::Popup::close_id(ui.ctx(), popup_id);
                    }
                }
            });
        });
    changed
}

pub fn nav_item(ui: &mut Ui, label: &str, active: bool, width: f32) -> bool {
    let p = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 36.0), Sense::click());
    if resp.hovered() { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
    let hover = anim_bool(ui.ctx(), resp.id.with("h"), resp.hovered() && !active, 0.08);
    let on = anim_bool(ui.ctx(), resp.id.with("on"), active, 0.12);
    ui.painter().rect_filled(rect, 10.0, fade(p.control, hover * 0.6));
    let f = if on > 0.5 { Font::Semibold } else { Font::Regular };
    ui.painter().text(pos2(rect.min.x + 14.0, rect.center().y), Align2::LEFT_CENTER, label, font(f, size::BODY), mix(p.secondary, p.text, on));
    resp.clicked()
}

pub fn list_item(ui: &mut Ui, label: &str, active: bool, width: f32) -> bool {
    let p = palette(ui.ctx());
    let (rect, resp) = ui.allocate_exact_size(vec2(width, 34.0), Sense::click());
    if resp.hovered() { ui.ctx().set_cursor_icon(CursorIcon::PointingHand); }
    let hover = anim_bool(ui.ctx(), resp.id.with("h"), resp.hovered(), 0.08);
    let on = anim_bool(ui.ctx(), resp.id.with("on"), active, 0.12);
    ui.painter().rect_filled(rect, 8.0, mix(fade(p.control, hover * 0.7), p.control_hover, on));
    let f = if active { Font::Semibold } else { Font::Regular };
    ui.painter().text(pos2(rect.min.x + 12.0, rect.center().y), Align2::LEFT_CENTER, label, font(f, size::BODY), p.text);
    resp.clicked()
}

/// A keyboard key drawn as a little cap; returns its width.
pub fn keycap(ui: &mut Ui, key: &str) -> f32 {
    let p = palette(ui.ctx());
    let l = text_size(ui, key, Font::Semibold, size::SMALL, None);
    let (rect, _) = ui.allocate_exact_size(vec2((l.x + 14.0).max(30.0), 24.0), Sense::hover());
    ui.painter().rect_filled(rect, 6.0, p.control);
    ui.painter().line_segment([pos2(rect.min.x + 3.0, rect.max.y - 0.5), pos2(rect.max.x - 3.0, rect.max.y - 0.5)], Stroke::new(1.5_f32, p.separator));
    ui.painter().text(rect.center(), Align2::CENTER_CENTER, key, font(Font::Semibold, size::SMALL), p.text);
    rect.width()
}

pub fn progress_bar(ui: &mut Ui, id: Id, fraction: f32, width: f32, height: f32) {
    let p = palette(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let shown = animate(ui.ctx(), id, fraction.clamp(0.0, 1.0), 0.5);
    ui.painter().rect_filled(rect, height * 0.5, p.control);
    if shown > 0.001 {
        let filled = height.max(width * shown);
        ui.painter().rect_filled(Rect::from_min_size(rect.min, vec2(filled, height)), height * 0.5, p.accent);
    }
}

pub fn spinner(ui: &Ui, center: Pos2, radius: f32, ink: Color32) {
    let t = ui.input(|i| i.time) as f32;
    let start = t * 5.5;
    let pts: Vec<Pos2> = (0..=24).map(|i| { let a = start + 4.2 * i as f32 / 24.0; center + vec2(a.cos(), a.sin()) * radius }).collect();
    ui.painter().add(Shape::line(pts, Stroke::new(1.8_f32, ink)));
    ui.ctx().request_repaint();
}

pub fn check_mark(ui: &Ui, center: Pos2, s: f32, ink: Color32) {
    let h = s * 0.5;
    ui.painter().add(Shape::line(vec![center + vec2(-h, 0.0), center + vec2(-h * 0.3, h * 0.65), center + vec2(h, -h * 0.6)], Stroke::new(1.8_f32, ink)));
}
pub fn cross_mark(ui: &Ui, center: Pos2, s: f32, ink: Color32) {
    let h = s * 0.45;
    ui.painter().line_segment([center + vec2(-h, -h), center + vec2(h, h)], Stroke::new(1.8_f32, ink));
    ui.painter().line_segment([center + vec2(-h, h), center + vec2(h, -h)], Stroke::new(1.8_f32, ink));
}

/// Fill a convex polygon with a per-point colour and a 1 px anti-aliased edge.
fn fill_convex(ui: &Ui, pts: &[Pos2], color: impl Fn(Pos2) -> Color32) {
    let n = pts.len();
    if n < 3 { return; }
    let center = pts.iter().fold(Vec2::ZERO, |a, p| a + p.to_vec2()) / n as f32;
    let mut mesh = Mesh::default();
    let v = |m: &mut Mesh, p: Pos2, c: Color32| { m.vertices.push(Vertex { pos: p, uv: egui::epaint::WHITE_UV, color: c }); m.vertices.len() as u32 - 1 };
    let ci = v(&mut mesh, center.to_pos2(), color(center.to_pos2()));
    let feather = 1.0 / ui.ctx().pixels_per_point();
    let mut inner = Vec::with_capacity(n);
    let mut outer = Vec::with_capacity(n);
    for i in 0..n {
        let (prev, cur, next) = (pts[(i + n - 1) % n], pts[i], pts[(i + 1) % n]);
        let normal = ((cur - prev).normalized().rot90() + (next - cur).normalized().rot90()).normalized();
        let out = if (cur + normal - center.to_pos2()).length() > (cur - center.to_pos2()).length() { normal } else { -normal };
        let col = color(cur);
        inner.push(v(&mut mesh, cur - out * feather * 0.5, col));
        outer.push(v(&mut mesh, cur + out * feather * 0.5, Color32::TRANSPARENT));
    }
    for i in 0..n {
        let j = (i + 1) % n;
        mesh.add_triangle(ci, inner[i], inner[j]);
        mesh.add_triangle(inner[i], outer[i], outer[j]);
        mesh.add_triangle(inner[i], outer[j], inner[j]);
    }
    ui.painter().add(Shape::mesh(mesh));
}

fn rounded_square(center: Pos2, half: f32, corner: f32, angle: f32) -> Vec<Pos2> {
    let (s, c) = angle.sin_cos();
    let place = |x: f32, y: f32| pos2(center.x + x * c - y * s, center.y + x * s + y * c);
    let k = half - corner;
    let corners = [(k, k, 0.0f32), (-k, k, 0.5), (-k, -k, 1.0), (k, -k, 1.5)];
    let mut pts = Vec::new();
    for (cx, cy, start) in corners {
        for step in 0..=6 {
            let a = (start + step as f32 / 12.0) * std::f32::consts::PI;
            pts.push(place(cx + a.cos() * corner, cy + a.sin() * corner));
        }
    }
    pts
}

/// The launcher's logo: a pink rounded tile with a tilted white die showing three.
pub fn die(ui: &Ui, origin: Pos2, s: f32) {
    let rect = Rect::from_min_size(origin, vec2(s, s));
    let (top, bottom) = (c(255, 111, 180), c(212, 20, 111));
    fill_convex(ui, &rounded_square(rect.center(), s * 0.5, s * 0.215, 0.0), |p| mix(top, bottom, (p.y - rect.min.y) / s));
    let angle = 14f32.to_radians();
    let half = s * 0.285;
    let corner = half * 0.48;
    fill_convex(ui, &rounded_square(rect.center() + vec2(0.0, s * 0.035), half, corner, angle), |_| ca(106, 0, 53, 70));
    fill_convex(ui, &rounded_square(rect.center(), half, corner, angle), |_| c(250, 249, 252));
    let spacing = half * 0.52;
    let (sn, cs) = angle.sin_cos();
    for i in -1..=1 {
        let (x, y) = (spacing * i as f32, spacing * i as f32);
        ui.painter().circle_filled(pos2(rect.center().x + x * cs - y * sn, rect.center().y + x * sn + y * cs), s * 0.063, c(27, 27, 38));
    }
}

// ---------- layout ----------
/// Rounded surface behind `content`, `width` wide.
pub fn card<R>(ui: &mut Ui, width: f32, content: impl FnOnce(&mut Ui) -> R) -> R {
    let p = palette(ui.ctx());
    let bg = ui.painter().add(Shape::Noop);
    let top = ui.cursor().min;
    let r = ui.scope(|ui| { ui.set_width(width); content(ui) }).inner;
    let rect = Rect::from_min_max(top, pos2(top.x + width, ui.cursor().min.y));
    ui.painter().set(bg, Shape::rect_filled(rect, 14.0, p.surface));
    r
}

/// One settings line: label and detail on the left, a control on the right (C++ `row`).
pub fn row(ui: &mut Ui, width: f32, label: &str, detail: &str, cw: f32, ch: f32, control: impl FnOnce(&mut Ui)) {
    let p = palette(ui.ctx());
    let pad = 18.0;
    let tw = width - pad * 2.0 - cw - 24.0;
    let lg = ui.painter().layout(label.to_string(), font(Font::Regular, size::BODY), p.text, tw);
    let dg = (!detail.is_empty()).then(|| ui.painter().layout(detail.to_string(), font(Font::Regular, size::DETAIL), p.secondary, tw));
    let content = lg.size().y + dg.as_ref().map(|g| 3.0 + g.size().y).unwrap_or(0.0);
    let height = (content + 28.0).max(ch + 26.0);
    let top = ui.cursor().min;
    let rect = Rect::from_min_size(top, vec2(width, height));
    ui.allocate_rect(rect, Sense::hover());
    let ty = (top.y + (height - content) * 0.5).round();
    let lh = lg.size().y;
    ui.painter().galley(pos2(top.x + pad, ty), lg, p.text);
    if let Some(g) = dg { ui.painter().galley(pos2(top.x + pad, ty + lh + 3.0), g, p.secondary); }
    let crect = Rect::from_min_size(pos2(top.x + width - pad - cw, (top.y + (height - ch) * 0.5).round()), vec2(cw, ch));
    // a child Ui so the control does not move the parent's cursor back up
    let mut child = ui.new_child(UiBuilder::new().max_rect(crect).layout(egui::Layout::left_to_right(egui::Align::Center)));
    control(&mut child);
}

pub fn inset_separator(ui: &mut Ui, width: f32) {
    let p = palette(ui.ctx());
    let (rect, _) = ui.allocate_exact_size(vec2(width, 1.0), Sense::hover());
    ui.painter().rect_filled(Rect::from_min_max(pos2(rect.min.x + 18.0, rect.min.y), pos2(rect.max.x - 18.0, rect.max.y)), 0.0, p.separator);
}

/// Section title above a card.
pub fn group_title(ui: &mut Ui, title: &str) {
    let p = palette(ui.ctx());
    text(ui, title, Font::Semibold, size::DETAIL, p.secondary, None);
    gap(ui, 10.0);
}

#[allow(dead_code)]
pub fn link(ui: &mut Ui, label: impl Into<WidgetText>, url: &str) { ui.hyperlink_to(label, url); }

// ---------- window chrome (titlebar.cpp) ----------
pub const TITLEBAR: f32 = 32.0;
const BUTTON_W: f32 = 46.0;
const RESIZE: f32 = 6.0;

/// Borderless window: drag area, minimise / maximise / close buttons, resize edges.
pub fn window_chrome(ctx: &egui::Context) {
    let p = palette(ctx);
    let screen = ctx.content_rect();
    let maximized = ctx.input(|i| i.viewport().maximized.unwrap_or(false));
    egui::Area::new(Id::new("titlebar")).order(egui::Order::Foreground).fixed_pos(screen.min).interactable(true).show(ctx, |ui| {
        let bar = Rect::from_min_size(screen.min, vec2(screen.width(), TITLEBAR));
        let drag = Rect::from_min_max(bar.min, pos2(bar.max.x - BUTTON_W * 3.0, bar.max.y));
        let resp = ui.interact(drag, Id::new("titlebar-drag"), Sense::click_and_drag());
        if resp.double_clicked() { ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized)); }
        else if resp.drag_started_by(egui::PointerButton::Primary) { ctx.send_viewport_cmd(egui::ViewportCommand::StartDrag); }
        for i in 0..3 {
            let r = Rect::from_min_size(pos2(bar.max.x - BUTTON_W * (3 - i) as f32, bar.min.y), vec2(BUTTON_W, TITLEBAR));
            let b = ui.interact(r, Id::new(("window-button", i)), Sense::click());
            let t = ctx.animate_bool_with_time(b.id.with("h"), b.hovered(), 0.06);
            let close = i == 2;
            let fill = if close { fade(c(232, 17, 35), t) } else { fade(p.control_hover, t) };
            ui.painter().rect_filled(r, 0.0, fill);
            let col = mix(p.secondary, if close { Color32::WHITE } else { p.text }, t);
            let cc = r.center();
            let h = 5.0;
            let st = Stroke::new(1.0_f32, col);
            match i {
                0 => { ui.painter().line_segment([cc + vec2(-h, 0.0), cc + vec2(h, 0.0)], st); }
                1 if !maximized => { ui.painter().rect_stroke(Rect::from_center_size(cc, vec2(2.0 * h, 2.0 * h)), 0.0, st, StrokeKind::Inside); }
                1 => {
                    let s = 2.0;
                    ui.painter().rect_stroke(Rect::from_min_max(cc + vec2(-h, -h + s), cc + vec2(h - s, h)), 0.0, st, StrokeKind::Inside);
                    ui.painter().line_segment([cc + vec2(-h + s, -h), cc + vec2(h, -h)], st);
                    ui.painter().line_segment([cc + vec2(h, -h), cc + vec2(h, h - s)], st);
                }
                _ => {
                    ui.painter().line_segment([cc + vec2(-h, -h), cc + vec2(h, h)], st);
                    ui.painter().line_segment([cc + vec2(-h, h), cc + vec2(h, -h)], st);
                }
            }
            if b.clicked() {
                ctx.send_viewport_cmd(match i { 0 => egui::ViewportCommand::Minimized(true), 1 => egui::ViewportCommand::Maximized(!maximized), _ => egui::ViewportCommand::Close });
            }
        }
    });
    if maximized { return; }
    use egui::viewport::ResizeDirection as D;
    let Some(pos) = ctx.input(|i| i.pointer.hover_pos()) else { return };
    let (l, r, t, b) = (pos.x < screen.min.x + RESIZE, pos.x > screen.max.x - RESIZE, pos.y < screen.min.y + RESIZE, pos.y > screen.max.y - RESIZE);
    let dir = match (l, r, t, b) {
        (true, _, true, _) => Some((D::NorthWest, CursorIcon::ResizeNorthWest)),
        (_, true, true, _) => Some((D::NorthEast, CursorIcon::ResizeNorthEast)),
        (true, _, _, true) => Some((D::SouthWest, CursorIcon::ResizeSouthWest)),
        (_, true, _, true) => Some((D::SouthEast, CursorIcon::ResizeSouthEast)),
        (_, _, true, _) => Some((D::North, CursorIcon::ResizeNorth)),
        (_, _, _, true) => Some((D::South, CursorIcon::ResizeSouth)),
        (true, ..) => Some((D::West, CursorIcon::ResizeWest)),
        (_, true, ..) => Some((D::East, CursorIcon::ResizeEast)),
        _ => None,
    };
    if let Some((d, icon)) = dir {
        ctx.set_cursor_icon(icon);
        if ctx.input(|i| i.pointer.primary_pressed()) { ctx.send_viewport_cmd(egui::ViewportCommand::BeginResize(d)); }
    }
}
