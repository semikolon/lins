//! What a scene is and where its parts go. No drawing, no web.
//!
//! The page builds the scene (which node is in the middle, which neighbours are shown, every
//! size and colour) and hands it over as JSON. This file places it. The browser's own drawing of
//! the same scene places it by the same rules, in `lins-web/src/lib/trial/scene.ts`: a change to
//! one is a change to the other, and the page can ask [`layout`] for its answer to check.
//!
//! The rules, in the order they are applied:
//!
//! 1. The node the scene was opened from sits in the middle, at (0, 0).
//! 2. What points at it is on the left, what it points at on the right.
//! 3. A side is a list of groups, one per kind of link. A group is a heading, its cards, and a
//!    stub for the neighbours not shown.
//! 4. A side runs down one column until the column would pass `max_col_h`; it then goes on in a
//!    further column, one step out. A heading is never the last thing in a column, and a group
//!    that goes on in a new column gets its heading again.
//! 5. Every column is centred on the middle node.
//! 6. One link per card, between the card and the middle node, fanned along the node's edge in
//!    the order the cards stand. A link between two neighbours is an arc on the outside of
//!    their column, and is drawn only when both stand in the same column.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Every size and colour of a scene. The page owns these; nothing here has a fallback, so a
/// number missing from the page is an error the page sees.
#[derive(Clone, Debug, Deserialize)]
pub struct Style {
    pub canvas: String,
    pub card: String,
    pub card_hover: String,
    pub card_focus: String,
    pub text: String,
    pub muted: String,

    pub card_w: f32,
    pub focus_w: f32,
    pub pad_x: f32,
    pub pad_y: f32,
    pub radius: f32,
    /// The coloured edge of a card, saying its kind.
    pub stripe: f32,

    pub font: f32,
    pub line: f32,
    pub focus_font: f32,
    pub focus_line: f32,
    pub small_font: f32,
    pub small_line: f32,
    /// A name longer than this many lines is cut with an ellipsis.
    pub max_lines: usize,

    pub header_h: f32,
    pub stub_h: f32,
    pub gap_y: f32,
    pub header_gap: f32,
    pub group_gap: f32,
    pub col_gap: f32,
    pub wrap_gap: f32,
    pub max_col_h: f32,

    pub link_w: f32,
    pub link_alpha: f32,
    pub arc_alpha: f32,
    pub arrow_len: f32,
    pub arrow_half: f32,
    /// The most that two links' ends are apart on the middle node's edge.
    pub fan_step: f32,
    pub fan_inset: f32,
    /// Links stop this far outside the middle node, clear of its ring.
    pub focus_gap: f32,
    pub arc_min: f32,
    pub arc_slope: f32,
    pub arc_max: f32,
    pub margin: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Node {
    pub id: String,
    pub name: String,
    pub color: String,
}

/// The neighbours joined to the middle node by one kind of link, on one side.
#[derive(Clone, Debug, Deserialize)]
pub struct Group {
    pub kind: String,
    pub color: String,
    /// "supports this · 5 of 21"
    pub header_text: String,
    /// The heading again, where the group goes on in a further column.
    pub continued_text: String,
    /// "+16 more", when some of the group is not shown.
    pub stub_text: Option<String>,
    pub cards: Vec<Node>,
}

/// A link between two neighbours.
#[derive(Clone, Debug, Deserialize)]
pub struct Arc {
    pub from: String,
    pub to: String,
    pub color: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Scene {
    pub style: Style,
    pub focus: Node,
    pub left: Vec<Group>,
    pub right: Vec<Group>,
    pub arcs: Vec<Arc>,
}

/// How tall each box of text came out. Whoever draws the scene measures its own text.
#[derive(Clone, Debug, Deserialize)]
pub struct Heights {
    pub focus: f32,
    /// One list per group, one height per card.
    pub left: Vec<Vec<f32>>,
    pub right: Vec<Vec<f32>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Focus,
    Card,
    Header,
    Stub,
}

/// One placed thing. `side` is -1 on the left, 0 in the middle, 1 on the right; `col` counts
/// outwards from the middle.
#[derive(Clone, Debug, Serialize)]
pub struct Item {
    pub role: Role,
    /// A node's id for a card, the kind of link for a heading or a stub.
    pub id: String,
    pub text: String,
    /// A card's own colour (its kind of node); a heading's or stub's kind of link.
    pub color: String,
    /// The colour of the kind of link that joins a card to the middle node.
    pub link: String,
    pub side: i8,
    pub col: usize,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl Item {
    fn mid_y(&self) -> f32 {
        self.y + self.h / 2.0
    }
}

/// A curve of four points, with an arrowhead at `tip` pointing right (`dir` 1) or left (-1).
#[derive(Clone, Debug, Serialize)]
pub struct Link {
    pub p: [[f32; 2]; 4],
    pub tip: [f32; 2],
    pub dir: f32,
    pub color: String,
    /// True for a link between two neighbours.
    pub arc: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Layout {
    pub items: Vec<Item>,
    pub links: Vec<Link>,
    /// Left, top, right, bottom of everything placed, with the margin.
    pub bounds: [f32; 4],
    /// Links between neighbours that are in the scene and not drawn, because their two cards
    /// do not stand in the same column.
    pub arcs_left_out: usize,
}

struct Entry {
    role: Role,
    id: String,
    text: String,
    color: String,
    link: String,
    h: f32,
    /// The first thing of its group.
    starts_group: bool,
}

fn entries_of(group: &Group, heights: &[f32], st: &Style) -> Vec<Entry> {
    let mut out = Vec::with_capacity(group.cards.len() + 2);
    if !group.cards.is_empty() {
        out.push(Entry {
            role: Role::Header,
            id: group.kind.clone(),
            text: group.header_text.clone(),
            color: group.color.clone(),
            link: String::new(),
            h: st.header_h,
            starts_group: true,
        });
    }
    for (i, card) in group.cards.iter().enumerate() {
        out.push(Entry {
            role: Role::Card,
            id: card.id.clone(),
            text: card.name.clone(),
            color: card.color.clone(),
            link: group.color.clone(),
            // A card nobody measured takes one line, so a wrong list shows as a wrong picture
            // and never as a crash.
            h: heights.get(i).copied().unwrap_or(st.pad_y * 2.0 + st.line),
            starts_group: false,
        });
    }
    if let Some(stub) = &group.stub_text {
        let alone = out.is_empty();
        out.push(Entry {
            role: Role::Stub,
            id: group.kind.clone(),
            text: stub.clone(),
            color: group.color.clone(),
            link: String::new(),
            h: st.stub_h,
            starts_group: alone,
        });
    }
    out
}

fn gap_before(above: Option<Role>, entry: &Entry, st: &Style) -> f32 {
    match above {
        None => 0.0,
        Some(Role::Header) => st.header_gap,
        Some(_) if entry.starts_group => st.group_gap,
        Some(_) => st.gap_y,
    }
}

/// One side as columns, nearest the middle first. Each entry comes with its distance from the
/// top of its column.
fn flow(groups: &[Group], heights: &[Vec<f32>], st: &Style) -> Vec<Vec<(Entry, f32)>> {
    let none: Vec<f32> = Vec::new();
    let mut cols: Vec<Vec<(Entry, f32)>> = vec![Vec::new()];
    let mut y = 0.0_f32;
    for (g, group) in groups.iter().enumerate() {
        let mut entries = entries_of(group, heights.get(g).unwrap_or(&none), st).into_iter().peekable();
        while let Some(entry) = entries.next() {
            let above = cols.last().and_then(|col| col.last()).map(|(e, _)| e.role);
            let mut need = gap_before(above, &entry, st) + entry.h;
            if entry.role == Role::Header {
                // A heading stays with the first thing under it.
                if let Some(next) = entries.peek() {
                    need += st.header_gap + next.h;
                }
            }
            if above.is_some() && y + need > st.max_col_h {
                cols.push(Vec::new());
                y = 0.0;
                if !entry.starts_group {
                    let again = Entry {
                        role: Role::Header,
                        id: group.kind.clone(),
                        text: group.continued_text.clone(),
                        color: group.color.clone(),
                        link: String::new(),
                        h: st.header_h,
                        starts_group: true,
                    };
                    y = again.h;
                    cols.last_mut().expect("just pushed").push((again, 0.0));
                }
            }
            let above = cols.last().and_then(|col| col.last()).map(|(e, _)| e.role);
            let top = y + gap_before(above, &entry, st);
            y = top + entry.h;
            cols.last_mut().expect("never empty").push((entry, top));
        }
    }
    cols.retain(|col| !col.is_empty());
    cols
}

/// Place a scene, given how tall its text came out.
pub fn layout(scene: &Scene, heights: &Heights) -> Layout {
    let st = &scene.style;
    let (fw, fh) = (st.focus_w, heights.focus);
    let mut items = vec![Item {
        role: Role::Focus,
        id: scene.focus.id.clone(),
        text: scene.focus.name.clone(),
        color: scene.focus.color.clone(),
        link: String::new(),
        side: 0,
        col: 0,
        x: -fw / 2.0,
        y: -fh / 2.0,
        w: fw,
        h: fh,
    }];

    for (side, groups, hs) in [(-1_i8, &scene.left, &heights.left), (1_i8, &scene.right, &heights.right)] {
        for (k, col) in flow(groups, hs, st).into_iter().enumerate() {
            let height = col.last().map(|(e, top)| top + e.h).unwrap_or(0.0);
            let out = k as f32 * (st.card_w + st.wrap_gap);
            let x = if side < 0 {
                -fw / 2.0 - st.col_gap - st.card_w - out
            } else {
                fw / 2.0 + st.col_gap + out
            };
            for (entry, top) in col {
                items.push(Item {
                    role: entry.role,
                    id: entry.id,
                    text: entry.text,
                    color: entry.color,
                    link: entry.link,
                    side,
                    col: k,
                    x,
                    y: top - height / 2.0,
                    w: st.card_w,
                    h: entry.h,
                });
            }
        }
    }

    let mut links = Vec::new();
    for side in [-1_i8, 1_i8] {
        let mut cards: Vec<&Item> = items.iter().filter(|i| i.role == Role::Card && i.side == side).collect();
        cards.sort_by(|a, b| a.mid_y().total_cmp(&b.mid_y()).then(a.col.cmp(&b.col)));
        let n = cards.len();
        let room = (fh - 2.0 * st.fan_inset).max(0.0);
        let step = if n > 1 { (room / (n - 1) as f32).min(st.fan_step) } else { 0.0 };
        for (i, card) in cards.iter().enumerate() {
            let fy = (i as f32 - (n as f32 - 1.0) / 2.0) * step;
            let edge = fw / 2.0 + st.focus_gap;
            let (from, tip) = if side < 0 {
                ([card.x + card.w, card.mid_y()], [-edge, fy])
            } else {
                ([edge, fy], [card.x, card.mid_y()])
            };
            let end = [tip[0] - st.arrow_len, tip[1]];
            let dx = (end[0] - from[0]) / 2.0;
            links.push(Link {
                p: [from, [from[0] + dx, from[1]], [end[0] - dx, end[1]], end],
                tip,
                dir: 1.0,
                color: card.link.clone(),
                arc: false,
            });
        }
    }

    let mut outermost: HashMap<i8, usize> = HashMap::new();
    let mut by_id: HashMap<&str, &Item> = HashMap::new();
    for item in items.iter().filter(|i| i.role == Role::Card) {
        by_id.entry(item.id.as_str()).or_insert(item);
        let far = outermost.entry(item.side).or_insert(0);
        *far = (*far).max(item.col);
    }
    let mut arcs_left_out = 0;
    for arc in &scene.arcs {
        let ends = (by_id.get(arc.from.as_str()), by_id.get(arc.to.as_str()));
        let (a, b) = match ends {
            (Some(a), Some(b)) if a.side == b.side && a.col == b.col && a.id != b.id => (a, b),
            _ => {
                arcs_left_out += 1;
                continue;
            }
        };
        let s = f32::from(a.side);
        let x = if a.side < 0 { a.x } else { a.x + a.w };
        let (ya, yb) = (a.mid_y(), b.mid_y());
        let room = if a.col < outermost[&a.side] { st.wrap_gap * 0.85 } else { st.arc_max };
        let bulge = (st.arc_min + (yb - ya).abs() * st.arc_slope).min(room);
        // A curve whose two middle points stand `c` out reaches three quarters of `c`.
        let c = bulge * 4.0 / 3.0;
        let end = [x + s * st.arrow_len, yb];
        links.push(Link {
            p: [[x, ya], [x + s * c, ya], [end[0] + s * c, yb], end],
            tip: [x, yb],
            dir: -s,
            color: arc.color.clone(),
            arc: true,
        });
    }

    let mut bounds = [f32::MAX, f32::MAX, f32::MIN, f32::MIN];
    let mut cover = |x: f32, y: f32| {
        bounds = [bounds[0].min(x), bounds[1].min(y), bounds[2].max(x), bounds[3].max(y)];
    };
    for item in &items {
        cover(item.x, item.y);
        cover(item.x + item.w, item.y + item.h);
    }
    for link in links.iter().filter(|l| l.arc) {
        // An arc reaches three quarters of the way to its middle points.
        let reach = link.p[0][0] + (link.p[1][0] - link.p[0][0]) * 0.75;
        cover(reach, link.p[0][1]);
    }
    let bounds = [bounds[0] - st.margin, bounds[1] - st.margin, bounds[2] + st.margin, bounds[3] + st.margin];

    Layout { items, links, bounds, arcs_left_out }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn style() -> Style {
        serde_json::from_value(serde_json::json!({
            "canvas": "#2a3040", "card": "#3b445a", "card_hover": "#465069", "card_focus": "#434d68",
            "text": "#eef1f6", "muted": "#a9b3c6",
            "card_w": 168.0, "focus_w": 240.0, "pad_x": 10.0, "pad_y": 8.0, "radius": 7.0, "stripe": 4.0,
            "font": 14.0, "line": 19.0, "focus_font": 16.0, "focus_line": 22.0,
            "small_font": 12.0, "small_line": 16.0, "max_lines": 4,
            "header_h": 16.0, "stub_h": 28.0, "gap_y": 20.0, "header_gap": 8.0, "group_gap": 28.0,
            "col_gap": 96.0, "wrap_gap": 44.0, "max_col_h": 400.0,
            "link_w": 2.0, "link_alpha": 0.8, "arc_alpha": 0.5, "arrow_len": 8.0, "arrow_half": 4.0,
            "fan_step": 6.0, "fan_inset": 8.0, "focus_gap": 5.0,
            "arc_min": 18.0, "arc_slope": 0.18, "arc_max": 70.0, "margin": 24.0
        }))
        .expect("the test style names every field")
    }

    fn node(id: &str) -> Node {
        Node { id: id.into(), name: format!("name of {id}"), color: "#8b7fd6".into() }
    }

    fn group(kind: &str, ids: &[&str], stub: Option<&str>) -> Group {
        Group {
            kind: kind.into(),
            color: "#3fae6a".into(),
            header_text: format!("{kind} this"),
            continued_text: format!("{kind} this · continued"),
            stub_text: stub.map(str::to_owned),
            cards: ids.iter().map(|id| node(id)).collect(),
        }
    }

    fn scene(left: Vec<Group>, right: Vec<Group>, arcs: Vec<Arc>) -> Scene {
        Scene { style: style(), focus: node("f"), left, right, arcs }
    }

    fn heights_for(scene: &Scene, card: f32) -> Heights {
        let of = |groups: &[Group]| groups.iter().map(|g| vec![card; g.cards.len()]).collect();
        Heights { focus: 60.0, left: of(&scene.left), right: of(&scene.right) }
    }

    fn cards_on(layout: &Layout, side: i8) -> Vec<&Item> {
        layout.items.iter().filter(|i| i.role == Role::Card && i.side == side).collect()
    }

    #[test]
    fn the_middle_node_is_at_the_origin_and_sides_stand_clear_of_it() {
        let s = scene(vec![group("SUPPORTS", &["a", "b"], None)], vec![group("ATTACKS", &["c"], None)], vec![]);
        let l = layout(&s, &heights_for(&s, 54.0));
        let focus = &l.items[0];
        assert_eq!((focus.x + focus.w / 2.0, focus.y + focus.h / 2.0), (0.0, 0.0));
        for card in cards_on(&l, -1) {
            assert_eq!(card.x + card.w, -120.0 - 96.0, "a left card ends one column gap short of the middle node");
        }
        for card in cards_on(&l, 1) {
            assert_eq!(card.x, 120.0 + 96.0);
        }
    }

    #[test]
    fn a_column_is_centred_on_the_middle_node() {
        let s = scene(vec![group("SUPPORTS", &["a", "b", "c"], Some("+4 more"))], vec![], vec![]);
        let l = layout(&s, &heights_for(&s, 54.0));
        let left: Vec<&Item> = l.items.iter().filter(|i| i.side == -1).collect();
        let top = left.first().unwrap().y;
        let bottom = left.last().map(|i| i.y + i.h).unwrap();
        assert!((top + bottom).abs() < 1e-3, "top {top} and bottom {bottom} are the same distance from the middle");
        assert_eq!(left.first().unwrap().role, Role::Header);
        assert_eq!(left.last().unwrap().role, Role::Stub);
    }

    #[test]
    fn a_tall_side_goes_on_in_a_further_column_and_says_what_it_continues() {
        let ids: Vec<String> = (0..9).map(|i| format!("n{i}")).collect();
        let ids: Vec<&str> = ids.iter().map(String::as_str).collect();
        let s = scene(vec![group("SUPPORTS", &ids, None)], vec![], vec![]);
        let l = layout(&s, &heights_for(&s, 54.0));
        let cols = l.items.iter().filter(|i| i.side == -1).map(|i| i.col).max().unwrap();
        assert!(cols >= 1, "nine cards of 54 with gaps of 20 do not fit in 400");
        for col in 0..=cols {
            let in_col: Vec<&Item> = l.items.iter().filter(|i| i.side == -1 && i.col == col).collect();
            assert_eq!(in_col[0].role, Role::Header, "column {col} opens on a heading");
            assert_ne!(in_col.last().unwrap().role, Role::Header, "column {col} does not end on a heading");
            let height = in_col.last().map(|i| i.y + i.h).unwrap() - in_col[0].y;
            assert!(height <= s.style.max_col_h + 1e-3, "column {col} is {height} tall");
            if col > 0 {
                assert!(in_col[0].text.ends_with("continued"));
                assert!(in_col[0].x < l.items.iter().find(|i| i.side == -1 && i.col == col - 1).unwrap().x);
            }
        }
        assert_eq!(cards_on(&l, -1).len(), 9, "no card is lost at a column break");
    }

    #[test]
    fn a_group_with_nothing_shown_is_a_stub_alone() {
        let s = scene(vec![group("SUPPORTS", &["a"], None), group("QUALIFIES", &[], Some("+17 qualifies this"))], vec![], vec![]);
        let l = layout(&s, &heights_for(&s, 54.0));
        let roles: Vec<Role> = l.items.iter().filter(|i| i.side == -1).map(|i| i.role).collect();
        assert_eq!(roles, vec![Role::Header, Role::Card, Role::Stub]);
        let (card, stub) = (&l.items[2], &l.items[3]);
        assert_eq!(stub.y - (card.y + card.h), s.style.group_gap, "a stub that is a group of its own is a group's distance away");
    }

    #[test]
    fn every_card_has_one_link_and_its_arrow_points_the_way_the_link_runs() {
        let s = scene(vec![group("SUPPORTS", &["a", "b", "c"], None)], vec![group("ATTACKS", &["d", "e"], None)], vec![]);
        let l = layout(&s, &heights_for(&s, 54.0));
        assert_eq!(l.links.len(), 5);
        for link in &l.links {
            assert!(link.p[0][0] < link.tip[0], "a link to or from the middle node runs left to right");
            assert_eq!(link.dir, 1.0);
            assert_eq!(link.p[3][0], link.tip[0] - s.style.arrow_len, "the line stops where the arrowhead begins");
        }
        let left_tips: Vec<f32> = l.links.iter().take(3).map(|k| k.tip[1]).collect();
        assert!(left_tips.windows(2).all(|w| w[0] < w[1]), "links arrive in the order their cards stand: {left_tips:?}");
        assert!((left_tips[0] + left_tips[2]).abs() < 1e-3, "and are spread evenly about the middle");
    }

    #[test]
    fn an_arc_is_drawn_only_between_cards_of_one_column_and_the_rest_are_counted() {
        let arc = |from: &str, to: &str| Arc { from: from.into(), to: to.into(), color: "#d9a441".into() };
        let s = scene(
            vec![group("SUPPORTS", &["a", "b"], None)],
            vec![group("ATTACKS", &["c"], None)],
            vec![arc("a", "b"), arc("a", "c"), arc("a", "nowhere"), arc("a", "a")],
        );
        let l = layout(&s, &heights_for(&s, 54.0));
        let arcs: Vec<&Link> = l.links.iter().filter(|k| k.arc).collect();
        assert_eq!(arcs.len(), 1);
        assert_eq!(l.arcs_left_out, 3);
        let a = l.items.iter().find(|i| i.id == "a").unwrap();
        assert_eq!(arcs[0].p[0][0], a.x, "on the left the arc leaves from the card's outer, left edge");
        assert!(arcs[0].p[1][0] < a.x, "and bulges away from the middle");
        assert_eq!(arcs[0].dir, 1.0, "its arrowhead points back in, to the right");
        assert!(l.bounds[0] < a.x - s.style.margin, "the bounds make room for the arc");
    }

    #[test]
    fn a_card_nobody_measured_still_gets_a_place() {
        let s = scene(vec![group("SUPPORTS", &["a", "b"], None)], vec![], vec![]);
        let l = layout(&s, &Heights { focus: 60.0, left: vec![], right: vec![] });
        assert_eq!(cards_on(&l, -1).len(), 2);
        assert!(cards_on(&l, -1).iter().all(|c| c.h == 8.0 * 2.0 + 19.0));
    }
}
