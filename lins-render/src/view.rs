//! Draws a placed scene with egui, and pans and zooms it.
//!
//! The one question this file was written to answer is how text looks while a scene is zoomed.
//! egui lays text out at a size and keeps the result as a picture of triangles; a zoomed layer
//! stretches that picture. So there are three ways to draw the text of a zoomed scene, and
//! [`TextMode`] picks one:
//!
//! - **Scaled**: set the text once, at full size, and stretch it. This is what egui's own zoom
//!   container does. Cheap, and soft at every zoom but 100%.
//! - **SharpAtRest**: stretch while the zoom is changing, and set the text again at the new size
//!   a moment after it stops.
//! - **SharpAlways**: set the text again at every frame of a zoom. Sharp throughout, and every
//!   frame of a zoom pays for new glyphs.
//!
//! Lines are broken once, at full size, and stay as they are at every zoom, so a card never
//! changes shape under the reader.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use egui::epaint::text::{LayoutJob, TextFormat};
use egui::epaint::{CornerRadius, CubicBezierShape, Shadow};
use egui::{
    pos2, vec2, Align, Color32, Context, CursorIcon, FontData, FontDefinitions, FontFamily, FontId, LayerId, Painter,
    PointerButton, Pos2, Rect, Sense, Shape, Stroke, StrokeKind, Ui,
};
use egui::emath::TSTransform;

use crate::scene::{self, Heights, Layout, Role, Scene};

pub const ZOOM_MIN: f32 = 0.2;
pub const ZOOM_MAX: f32 = 3.0;
/// A scene never opens with its text smaller than this share of full size.
pub const HOME_ZOOM_MIN: f32 = 0.75;
/// How long the zoom must hold still before text is set again, in [`TextMode::SharpAtRest`].
const SETTLE_SECONDS: f64 = 0.18;

const BODY: &str = "lins-body";
const STRONG: &str = "lins-strong";

#[cfg(lins_fonts)]
const FACES: &[(&str, &str, &[u8])] = &[
    (BODY, "Manrope-Medium", include_bytes!("../assets/fonts/Manrope-Medium.ttf")),
    (STRONG, "Manrope-SemiBold", include_bytes!("../assets/fonts/Manrope-SemiBold.ttf")),
];
#[cfg(not(lins_fonts))]
const FACES: &[(&str, &str, &[u8])] = &[];

/// Give egui the page's own faces, with egui's built-in ones behind them for any character the
/// page's faces lack. Returns false when this build carries no faces of its own.
pub fn install_fonts(ctx: &Context) -> bool {
    let mut fonts = FontDefinitions::default();
    let behind = fonts.families.get(&FontFamily::Proportional).cloned().unwrap_or_default();
    for family in [BODY, STRONG] {
        fonts.families.insert(FontFamily::Name(family.into()), behind.clone());
    }
    for (family, face, bytes) in FACES {
        fonts.font_data.insert((*face).to_owned(), Arc::new(FontData::from_static(bytes)));
        fonts.families.entry(FontFamily::Name((*family).into())).or_default().insert(0, (*face).to_owned());
    }
    ctx.set_fonts(fonts);
    !FACES.is_empty()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextMode {
    Scaled,
    SharpAtRest,
    SharpAlways,
}

impl TextMode {
    pub fn from_number(n: u8) -> Self {
        match n {
            0 => Self::Scaled,
            2 => Self::SharpAlways,
            _ => Self::SharpAtRest,
        }
    }
}

struct Palette {
    card: Color32,
    card_hover: Color32,
    card_focus: Color32,
    text: Color32,
    muted: Color32,
}

fn colour(hex: &str) -> Color32 {
    Color32::from_hex(hex).unwrap_or(Color32::from_rgb(255, 0, 255))
}

fn faded(c: Color32, alpha: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), (alpha.clamp(0.0, 1.0) * 255.0).round() as u8)
}

/// A placed scene with its text broken into lines.
struct Laid {
    layout: Layout,
    /// The lines of each item, in the order of `layout.items`.
    lines: Vec<Vec<String>>,
    palette: Palette,
    /// The same, as JSON, for whoever wants to read the scene instead of looking at it.
    json: String,
    /// Names that did not fit in `max_lines` and were cut.
    cut: usize,
}

pub struct SceneView {
    scene: Option<Scene>,
    laid: Option<Laid>,
    /// The pixel density the text was measured at. A change means measuring again.
    laid_density: f32,
    /// Where a point of the scene is on the canvas: `canvas = translation + scaling * scene`.
    to_canvas: TSTransform,
    text_mode: TextMode,
    /// The zoom the text is set at. Equal to the zoom itself when the text is sharp.
    text_zoom: f32,
    zoom_seen: f32,
    zoom_changed_at: f64,
    fit_wanted: bool,
    home_wanted: bool,
    picked: Option<String>,
    frames: u64,
    /// Counts up each time the scene is placed again.
    placed: u64,
}

impl Default for SceneView {
    fn default() -> Self {
        Self {
            scene: None,
            laid: None,
            laid_density: 0.0,
            to_canvas: TSTransform::IDENTITY,
            text_mode: TextMode::SharpAtRest,
            text_zoom: 1.0,
            zoom_seen: 1.0,
            zoom_changed_at: f64::NEG_INFINITY,
            fit_wanted: false,
            home_wanted: false,
            picked: None,
            frames: 0,
            placed: 0,
        }
    }
}

fn format(family: &str, size: f32, line: f32, color: Color32) -> TextFormat {
    TextFormat {
        font_id: FontId::new(size, FontFamily::Name(family.into())),
        line_height: Some(line),
        color,
        // A line taller than its letters has the spare room half above and half below, as in
        // a web page.
        valign: Align::Center,
        ..Default::default()
    }
}

/// How wide a piece of text is when set on one line.
fn width_of(painter: &Painter, text: &str, family: &str, size: f32, line: f32) -> f32 {
    let mut job = LayoutJob::single_section(text.to_owned(), format(family, size, line, Color32::WHITE));
    job.wrap.max_width = f32::INFINITY;
    painter.layout_job(job).size().x
}

/// The pieces of a name between which a line may be broken, each with whether a space stands
/// before it. A line may break at a space, and after a hyphen or dash inside a word, as in a
/// web page.
fn pieces(text: &str) -> Vec<(String, bool)> {
    let mut out = Vec::new();
    for word in text.split_whitespace() {
        let mut piece = String::new();
        let mut spaced = true;
        let mut letters = word.chars().peekable();
        while let Some(ch) = letters.next() {
            piece.push(ch);
            let breaks_after = matches!(ch, '-' | '\u{2013}' | '\u{2014}' | '/');
            let inside = piece.chars().count() > 1 && letters.peek().is_some_and(|next| next.is_alphabetic());
            if breaks_after && inside {
                out.push((std::mem::take(&mut piece), spaced));
                spaced = false;
            }
        }
        if !piece.is_empty() {
            out.push((piece, spaced));
        }
    }
    out
}

/// Break a name into lines at full size. Returns the lines and whether the name was cut.
///
/// The breaking is done here and not by egui's own wrapping, which counts the space after a
/// line's last word as part of the line and so breaks a word earlier than a web page does.
fn break_lines(painter: &Painter, text: &str, family: &str, size: f32, line: f32, width: f32, max: usize) -> (Vec<String>, bool) {
    let wide = |s: &str| width_of(painter, s, family, size, line);
    let mut lines: Vec<String> = Vec::new();
    let mut current = String::new();
    for (piece, spaced) in pieces(text) {
        let joined = if current.is_empty() {
            piece.clone()
        } else if spaced {
            format!("{current} {piece}")
        } else {
            format!("{current}{piece}")
        };
        if wide(&joined) <= width {
            current = joined;
            continue;
        }
        if !current.is_empty() {
            lines.push(std::mem::take(&mut current));
        }
        // The piece opens a line. One too long for a whole line is cut where it must be.
        let mut rest = piece;
        while wide(&rest) > width {
            let mut cut = 0;
            for (i, ch) in rest.char_indices() {
                let end = i + ch.len_utf8();
                if wide(&rest[..end]) > width {
                    break;
                }
                cut = end;
            }
            if cut == 0 {
                cut = rest.chars().next().map_or(rest.len(), char::len_utf8);
            }
            lines.push(rest[..cut].to_owned());
            rest = rest[cut..].to_owned();
        }
        current = rest;
    }
    if !current.is_empty() {
        lines.push(current);
    }
    let max = max.max(1);
    let cut = lines.len() > max;
    if cut {
        lines.truncate(max);
        let last = lines.last_mut().expect("at least one line");
        let mut ending = format!("{last}…");
        while wide(&ending) > width && last.pop().is_some() {
            ending = format!("{}…", last.trim_end());
        }
        *last = ending;
    }
    (lines, cut)
}

impl SceneView {
    pub fn set_scene(&mut self, scene: Scene) {
        self.scene = Some(scene);
        self.laid = None;
        self.fit_wanted = false;
        self.home_wanted = true;
    }

    pub fn set_text_mode(&mut self, mode: TextMode) {
        self.text_mode = mode;
    }

    pub fn text_mode(&self) -> TextMode {
        self.text_mode
    }

    /// Put the scene's own (0, 0) at `(x, y)` on the canvas, at this zoom.
    pub fn set_view(&mut self, x: f32, y: f32, zoom: f32) {
        self.to_canvas = TSTransform::new(vec2(x, y), zoom.clamp(ZOOM_MIN, ZOOM_MAX));
        self.fit_wanted = false;
        self.home_wanted = false;
    }

    pub fn fit_next_frame(&mut self) {
        self.fit_wanted = true;
        self.home_wanted = false;
    }

    /// Where the scene's (0, 0) is on the canvas, and the zoom.
    pub fn view(&self) -> [f32; 3] {
        [self.to_canvas.translation.x, self.to_canvas.translation.y, self.to_canvas.scaling]
    }

    pub fn text_zoom(&self) -> f32 {
        self.text_zoom
    }

    pub fn frames(&self) -> u64 {
        self.frames
    }

    pub fn placed(&self) -> u64 {
        self.placed
    }

    pub fn cut(&self) -> usize {
        self.laid.as_ref().map_or(0, |l| l.cut)
    }

    /// The placed scene as JSON: every box with its text, and the links.
    pub fn json(&self) -> &str {
        self.laid.as_ref().map_or("null", |l| l.json.as_str())
    }

    pub fn canvas_colour(&self) -> Color32 {
        self.scene.as_ref().map_or(Color32::from_rgb(42, 48, 64), |s| colour(&s.style.canvas))
    }

    /// The node a click landed on since this was last asked.
    pub fn take_picked(&mut self) -> Option<String> {
        self.picked.take()
    }

    fn place(&mut self, painter: &Painter) {
        let Some(scene) = &self.scene else { return };
        let st = &scene.style;
        let card_width = st.card_w - st.stripe - 2.0 * st.pad_x;
        let focus_width = st.focus_w - st.stripe - 2.0 * st.pad_x;
        let mut cut = 0;

        let (focus_lines, focus_cut) =
            break_lines(painter, &scene.focus.name, STRONG, st.focus_font, st.focus_line, focus_width, st.max_lines + 2);
        cut += usize::from(focus_cut);

        let mut lines_of: HashMap<(i8, String), Vec<String>> = HashMap::new();
        let mut measure = |side: i8, groups: &[scene::Group]| -> Vec<Vec<f32>> {
            groups
                .iter()
                .map(|group| {
                    group
                        .cards
                        .iter()
                        .map(|card| {
                            let (lines, was_cut) = break_lines(painter, &card.name, BODY, st.font, st.line, card_width, st.max_lines);
                            cut += usize::from(was_cut);
                            let height = st.pad_y * 2.0 + lines.len().max(1) as f32 * st.line;
                            lines_of.insert((side, card.id.clone()), lines);
                            height
                        })
                        .collect()
                })
                .collect()
        };
        let heights = Heights {
            focus: st.pad_y * 2.0 + focus_lines.len().max(1) as f32 * st.focus_line,
            left: measure(-1, &scene.left),
            right: measure(1, &scene.right),
        };

        let layout = scene::layout(scene, &heights);
        let lines = layout
            .items
            .iter()
            .map(|item| match item.role {
                Role::Focus => focus_lines.clone(),
                Role::Card => lines_of.get(&(item.side, item.id.clone())).cloned().unwrap_or_default(),
                Role::Header | Role::Stub => vec![item.text.clone()],
            })
            .collect();
        let json = serde_json::to_string(&layout).unwrap_or_else(|_| "null".into());
        let palette = Palette {
            card: colour(&st.card),
            card_hover: colour(&st.card_hover),
            card_focus: colour(&st.card_focus),
            text: colour(&st.text),
            muted: colour(&st.muted),
        };
        self.laid = Some(Laid { layout, lines, palette, json, cut });
        self.placed += 1;
    }

    fn fit(&mut self, canvas: Rect) {
        let Some(laid) = &self.laid else { return };
        let [left, top, right, bottom] = laid.layout.bounds;
        let (w, h) = ((right - left).max(1.0), (bottom - top).max(1.0));
        let zoom = (canvas.width() / w).min(canvas.height() / h).clamp(ZOOM_MIN, 1.0);
        let middle = vec2((left + right) / 2.0, (top + bottom) / 2.0);
        self.to_canvas = TSTransform::new(canvas.center().to_vec2() - zoom * middle, zoom);
        // A fit is a jump, not a zoom somebody is in the middle of: the text is set at once.
        self.text_zoom = zoom;
        self.zoom_seen = zoom;
    }

    /// Where a scene opens: as wide as the canvas when that keeps the text at three quarters
    /// of its size or more, with the middle node half way down. The top and bottom of a tall
    /// side are then off the canvas, a drag away.
    fn home(&mut self, canvas: Rect) {
        let Some(laid) = &self.laid else { return };
        let [left, _, right, _] = laid.layout.bounds;
        let zoom = (canvas.width() / (right - left).max(1.0)).clamp(HOME_ZOOM_MIN, 1.0);
        let middle = vec2((left + right) / 2.0, 0.0);
        self.to_canvas = TSTransform::new(canvas.center().to_vec2() - zoom * middle, zoom);
        self.text_zoom = zoom;
        self.zoom_seen = zoom;
    }

    /// The card, middle node or stub under a point of the scene.
    fn under(&self, at: Pos2) -> Option<usize> {
        let laid = self.laid.as_ref()?;
        laid.layout.items.iter().position(|item| {
            item.role != Role::Header && Rect::from_min_size(pos2(item.x, item.y), vec2(item.w, item.h)).contains(at)
        })
    }

    /// Draw the scene over the whole of `ui`, and take the pointer.
    pub fn ui(&mut self, ui: &mut Ui) {
        let canvas = ui.max_rect();
        let ctx = ui.ctx().clone();
        let density = ctx.pixels_per_point();
        if self.scene.is_some() && (self.laid.is_none() || density != self.laid_density) {
            self.place(ui.painter());
            self.laid_density = density;
        }
        if self.laid.is_some() && canvas.width() > 1.0 && canvas.height() > 1.0 {
            if std::mem::take(&mut self.home_wanted) {
                self.home(canvas);
            }
            if std::mem::take(&mut self.fit_wanted) {
                self.fit(canvas);
            }
        }
        ui.painter().rect_filled(canvas, 0.0, self.canvas_colour());
        self.frames += 1;

        let response = ui.interact(canvas, ui.id().with("lins-scene"), Sense::click_and_drag());
        if response.dragged_by(PointerButton::Primary) {
            self.to_canvas.translation += response.drag_delta();
        }
        // This view is the whole canvas, so every scroll and pinch that reaches egui is meant
        // for it. (Inside a larger egui window this would ask first whether the pointer is
        // over the scene.) A wheel can turn before the pointer has ever moved, and then there
        // is no pointer to zoom about: the middle of the canvas stands in.
        let pointer = ui.input(|i| i.pointer.latest_pos());
        let (zoom_by, scroll) = ui.input(|i| (i.zoom_delta(), i.smooth_scroll_delta));
        if zoom_by != 1.0 {
            let about = pointer.unwrap_or(canvas.center()).to_vec2();
            let before = self.to_canvas.scaling;
            let after = (before * zoom_by).clamp(ZOOM_MIN, ZOOM_MAX);
            // The point of the scene under the pointer stays under the pointer.
            self.to_canvas.translation = about - (after / before) * (about - self.to_canvas.translation);
            self.to_canvas.scaling = after;
        }
        self.to_canvas.translation += scroll;
        let hovered = pointer.filter(|_| response.contains_pointer()).and_then(|at| self.under(self.to_canvas.inverse() * at));
        if response.clicked() {
            if let (Some(at), Some(laid)) = (hovered, &self.laid) {
                let item = &laid.layout.items[at];
                if item.role == Role::Card {
                    self.picked = Some(item.id.clone());
                }
            }
        }
        if response.dragged() {
            ctx.set_cursor_icon(CursorIcon::Grabbing);
        } else if hovered.is_some() {
            ctx.set_cursor_icon(CursorIcon::PointingHand);
        }

        // Which zoom the text is set at.
        let zoom = self.to_canvas.scaling;
        let now = ui.input(|i| i.time);
        if zoom != self.zoom_seen {
            self.zoom_seen = zoom;
            self.zoom_changed_at = now;
        }
        match self.text_mode {
            TextMode::Scaled => self.text_zoom = 1.0,
            TextMode::SharpAlways => self.text_zoom = zoom,
            TextMode::SharpAtRest => {
                if self.text_zoom != zoom {
                    if now - self.zoom_changed_at >= SETTLE_SECONDS {
                        self.text_zoom = zoom;
                    } else {
                        // Nothing else may ask for a frame once the zoom stops, so ask for
                        // the one that sets the text.
                        ctx.request_repaint_after(Duration::from_secs_f64(SETTLE_SECONDS));
                    }
                }
            }
        }

        // The scene is drawn at the size its text is set at, on a layer of its own, and the
        // layer is stretched by whatever is left of the zoom.
        let drawn_at = self.text_zoom;
        let layer = LayerId::new(ui.layer_id().order, ui.id().with("lins-scene-layer"));
        let stretch = TSTransform::new(self.to_canvas.translation, zoom / drawn_at);
        ctx.set_sublayer(ui.layer_id(), layer);
        ctx.set_transform_layer(layer, stretch);
        let painter = Painter::new(ctx.clone(), layer, stretch.inverse() * canvas);
        self.paint(&painter, drawn_at, hovered);
    }

    fn paint(&self, painter: &Painter, k: f32, hovered: Option<usize>) {
        let (Some(scene), Some(laid)) = (&self.scene, &self.laid) else { return };
        let st = &scene.style;
        let pal = &laid.palette;
        let at = |p: [f32; 2]| pos2(p[0] * k, p[1] * k);
        let corner = |r: f32| (r * k).round().clamp(0.0, 255.0) as u8;

        for link in &laid.layout.links {
            let c = faded(colour(&link.color), if link.arc { st.arc_alpha } else { st.link_alpha });
            painter.add(CubicBezierShape::from_points_stroke(
                link.p.map(at),
                false,
                Color32::TRANSPARENT,
                Stroke::new(st.link_w * k, c),
            ));
            let tip = at(link.tip);
            let back = tip.x - link.dir * st.arrow_len * k;
            let half = st.arrow_half * k;
            painter.add(Shape::convex_polygon(vec![tip, pos2(back, tip.y - half), pos2(back, tip.y + half)], c, Stroke::NONE));
        }

        for (i, item) in laid.layout.items.iter().enumerate() {
            let rect = Rect::from_min_size(pos2(item.x * k, item.y * k), vec2(item.w * k, item.h * k));
            let lines = &laid.lines[i];
            match item.role {
                Role::Focus | Role::Card => {
                    let focus = item.role == Role::Focus;
                    let kind = colour(&item.color);
                    let r = corner(st.radius);
                    let shadow = Shadow {
                        offset: [0, (2.0 * k).round().clamp(0.0, 127.0) as i8],
                        blur: corner(6.0),
                        spread: 0,
                        color: Color32::from_black_alpha(72),
                    };
                    painter.add(shadow.as_shape(rect, r));
                    // The whole card in its kind's colour, then the body over all of it but
                    // a strip on the left: the coloured edge follows the card's round corner.
                    painter.rect_filled(rect, r, kind);
                    let body = Rect::from_min_max(pos2(rect.min.x + st.stripe * k, rect.min.y), rect.max);
                    let fill = if focus {
                        pal.card_focus
                    } else if hovered == Some(i) {
                        pal.card_hover
                    } else {
                        pal.card
                    };
                    painter.rect_filled(body, CornerRadius { nw: 0, sw: 0, ne: r, se: r }, fill);
                    if focus {
                        painter.rect_stroke(rect.expand(3.0 * k), corner(st.radius + 3.0), Stroke::new(2.0 * k, kind), StrokeKind::Middle);
                    }
                    let (family, size, line) = if focus { (STRONG, st.focus_font, st.focus_line) } else { (BODY, st.font, st.line) };
                    let left = rect.min.x + (st.stripe + st.pad_x) * k;
                    let top = rect.min.y + st.pad_y * k;
                    for (n, text) in lines.iter().enumerate() {
                        let galley = self.line(painter, text, family, size * k, line * k, pal.text);
                        painter.galley(pos2(left, top + n as f32 * line * k), galley, pal.text);
                    }
                }
                Role::Header => {
                    let c = colour(&item.color);
                    let galley = self.line(painter, &item.text, STRONG, st.small_font * k, st.small_line * k, c);
                    painter.galley(pos2(rect.min.x + 2.0 * k, rect.center().y - st.small_line * k / 2.0), galley, c);
                }
                Role::Stub => {
                    let c = colour(&item.color);
                    let fill = if hovered == Some(i) { faded(c, 0.16) } else { faded(c, 0.07) };
                    painter.rect(rect, corner(st.radius), fill, Stroke::new(k, faded(c, 0.55)), StrokeKind::Inside);
                    let galley = self.line(painter, &item.text, BODY, st.small_font * k, st.small_line * k, pal.muted);
                    let x = rect.center().x - galley.size().x / 2.0;
                    painter.galley(pos2(x, rect.center().y - st.small_line * k / 2.0), galley, pal.muted);
                }
            }
        }
    }

    /// One line of text, set at this size. egui keeps what it set last frame, so a scene that
    /// is not being zoomed sets nothing new.
    fn line(&self, painter: &Painter, text: &str, family: &str, size: f32, line: f32, color: Color32) -> Arc<egui::Galley> {
        let mut job = LayoutJob::single_section(text.to_owned(), format(family, size, line, color));
        job.wrap.max_width = f32::INFINITY;
        painter.layout_job(job)
    }
}

#[cfg(test)]
mod tests {
    use super::pieces;

    fn texts(text: &str) -> Vec<(&'static str, bool)> {
        pieces(text).into_iter().map(|(piece, spaced)| (&*piece.leak(), spaced)).collect()
    }

    #[test]
    fn a_line_may_break_at_a_space_and_after_a_hyphen_inside_a_word() {
        assert_eq!(
            texts("sugar-free self-raising flour"),
            vec![("sugar-", true), ("free", false), ("self-", true), ("raising", false), ("flour", true)]
        );
    }

    #[test]
    fn a_hyphen_at_a_words_edge_or_before_a_figure_is_no_place_to_break() {
        assert_eq!(texts("-x a- GPT-4 well-known"), vec![("-x", true), ("a-", true), ("GPT-4", true), ("well-", true), ("known", false)]);
    }

    #[test]
    fn a_dash_between_words_breaks_after_it() {
        assert_eq!(texts("language\u{2014}language"), vec![("language\u{2014}", true), ("language", false)]);
        assert_eq!(texts("  spaced   out  "), vec![("spaced", true), ("out", true)]);
    }
}
