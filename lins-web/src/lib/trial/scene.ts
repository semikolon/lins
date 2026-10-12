/**
 * One scene, for the trial of two ways to draw it.
 *
 * `buildScene` turns a node and its neighbours into a scene: the node in the middle, what points
 * at it grouped on the left, what it points at grouped on the right. `layout` places a scene.
 *
 * The scene is handed to both drawings: the browser's (HTML and SVG, in routes/trial) and the
 * Rust one (lins-render). Each measures its own text and then places by the same rules. The
 * rules here are the rules of lins-render/src/scene.rs, line for line; a change to one is a
 * change to the other, and the page checks them against each other when the Rust view is loaded.
 */

import type { GraphEdge, GraphNode, GraphView, ViewResult } from '$lib/types';
import { labelColor, nodeFullName, relColor } from '$lib/view';

/**
 * Every size and colour, in CSS pixels. Both drawings read these and nothing else, so the two
 * pictures differ only in who drew them. The values are the display proposal's starting ones
 * (docs/display_proposal_2026_10_10.md § 5.3, § 5.4, § 5.7).
 */
export const STYLE = {
  canvas: '#2a3040',
  card: '#3b445a',
  card_hover: '#465069',
  card_focus: '#434d68',
  text: '#eef1f6',
  muted: '#a9b3c6',

  card_w: 168,
  focus_w: 240,
  pad_x: 10,
  pad_y: 8,
  radius: 7,
  stripe: 4,

  font: 14,
  line: 19,
  focus_font: 16,
  focus_line: 22,
  small_font: 12,
  small_line: 16,
  max_lines: 4,

  header_h: 16,
  stub_h: 28,
  gap_y: 20,
  header_gap: 8,
  group_gap: 28,
  col_gap: 96,
  wrap_gap: 44,
  max_col_h: 860,

  link_w: 2,
  link_alpha: 0.8,
  arc_alpha: 0.5,
  arrow_len: 8,
  arrow_half: 4,
  fan_step: 6,
  fan_inset: 8,
  focus_gap: 5,
  arc_min: 18,
  arc_slope: 0.18,
  arc_max: 70,
  margin: 24,
};

export type Style = typeof STYLE;

export interface SceneNode {
  id: string;
  name: string;
  color: string;
}

/** The neighbours joined to the middle node by one kind of link, on one side. */
export interface SceneGroup {
  kind: string;
  color: string;
  header_text: string;
  continued_text: string;
  stub_text: string | null;
  cards: SceneNode[];
}

export interface SceneArc {
  from: string;
  to: string;
  color: string;
}

export interface Scene {
  style: Style;
  focus: SceneNode;
  left: SceneGroup[];
  right: SceneGroup[];
  arcs: SceneArc[];
}

/** What a scene shows and leaves out, for the line under the picture. */
export interface SceneFacts {
  /** Neighbours the server returned. */
  neighbours: number;
  shown: number;
  /** True when the server itself cut the neighbourhood short. */
  truncated: boolean;
  /** Links to the middle node beyond a neighbour's first, which the scene does not draw. */
  second_links: number;
  /** Links among the neighbours shown. */
  arcs: number;
}

export interface Heights {
  focus: number;
  left: number[][];
  right: number[][];
}

export type Role = 'focus' | 'card' | 'header' | 'stub';
export type Point = [number, number];

export interface Item {
  role: Role;
  id: string;
  text: string;
  color: string;
  link: string;
  side: -1 | 0 | 1;
  col: number;
  x: number;
  y: number;
  w: number;
  h: number;
}

export interface Link {
  p: [Point, Point, Point, Point];
  tip: Point;
  dir: number;
  color: string;
  arc: boolean;
}

export interface Layout {
  items: Item[];
  links: Link[];
  bounds: [number, number, number, number];
  arcs_left_out: number;
}

/** "DECOMPOSES_INTO" as words, unless the view gives the kind a name of its own. */
function kindWords(kind: string, view: GraphView | null): string {
  return view?.styling.relationships[kind]?.label ?? kind.toLowerCase().replaceAll('_', ' ');
}

interface Member {
  node: GraphNode;
  name: string;
  /** Links to the other neighbours: a neighbour that ties others together says the most. */
  ties: number;
  /** A second number to rank by, where the caller gives one; larger comes first. */
  weight: number;
}

function byRank(a: Member, b: Member): number {
  if (a.ties !== b.ties) return b.ties - a.ties;
  if (a.weight !== b.weight) return b.weight - a.weight;
  // Numbers inside a name count as numbers, so that "row 2" comes before "row 10".
  const byName = a.name.localeCompare(b.name, 'en', { numeric: true });
  return byName !== 0 ? byName : a.node.id - b.node.id;
}

/** How a scene chooses among a node's neighbours, beyond one number for all of them. */
export interface SceneChoice {
  /**
   * Show up to this many neighbours in each group (a side and a kind of link), by that group's
   * key, `-1:KIND` or `1:KIND`; a group not named shows `budget`. With this, `budget` is a
   * number for each group and no longer for the whole scene, so no kind of link is crowded out
   * by another.
   */
  perGroup?: Record<string, number>;
  /** A second number to rank a neighbour by, after its links to the other neighbours. */
  weight?: (node: GraphNode) => number;
}

/**
 * Build the scene around `result.center`, showing at most `budget` neighbours.
 *
 * Which neighbours: those with the most links to the other neighbours, then by name. The
 * proposal's ranking (§ 5.2) also weighs the search words and the kind of link and is computed
 * by the server; that is not built, and this is the part of it a page can compute.
 *
 * A neighbour joined to the middle node by several links stands once, in the group of its first
 * link (kinds in alphabetical order, the left side before the right).
 */
export function buildScene(
  result: ViewResult,
  view: GraphView | null,
  budget: number,
  choice: SceneChoice = {},
): { scene: Scene; facts: SceneFacts } {
  const centre = result.center ?? result.nodes[0]?.id;
  const nodes = new Map<number, GraphNode>(result.nodes.map((n) => [n.id, n]));
  const middle = nodes.get(centre);
  if (!middle) throw new Error('The node the scene was opened from is not among the nodes returned.');
  const labels = [...new Set(result.nodes.flatMap((n) => n.labels))].sort();
  const asCard = (node: GraphNode): SceneNode => ({
    id: String(node.id),
    name: nodeFullName(node, view),
    color: labelColor(node.labels[0] ?? '', view, labels),
  });

  const places = new Map<number, { side: -1 | 1; kind: string }[]>();
  const among: GraphEdge[] = [];
  const ties = new Map<number, number>();
  for (const edge of result.edges) {
    const { source_id: from, target_id: to } = edge;
    if (from === to) continue;
    if (to === centre || from === centre) {
      const other = to === centre ? from : to;
      if (!nodes.has(other)) continue;
      const list = places.get(other) ?? [];
      list.push({ side: to === centre ? -1 : 1, kind: edge.relationship_type });
      places.set(other, list);
    } else if (nodes.has(from) && nodes.has(to)) {
      among.push(edge);
      ties.set(from, (ties.get(from) ?? 0) + 1);
      ties.set(to, (ties.get(to) ?? 0) + 1);
    }
  }

  let secondLinks = 0;
  const groups = new Map<string, { side: -1 | 1; kind: string; members: Member[] }>();
  const all: Member[] = [];
  for (const [id, list] of places) {
    list.sort((a, b) => (a.kind !== b.kind ? (a.kind < b.kind ? -1 : 1) : a.side - b.side));
    secondLinks += list.length - 1;
    const { side, kind } = list[0];
    const node = nodes.get(id)!;
    const member: Member = { node, name: nodeFullName(node, view), ties: ties.get(id) ?? 0, weight: choice.weight?.(node) ?? 0 };
    const key = `${side}:${kind}`;
    const group = groups.get(key) ?? { side, kind, members: [] };
    group.members.push(member);
    groups.set(key, group);
    all.push(member);
  }

  all.sort(byRank);
  const shown = new Set<number>();
  if (choice.perGroup) {
    for (const [key, group] of groups) {
      const most = Math.max(0, choice.perGroup[key] ?? budget);
      for (const member of [...group.members].sort(byRank).slice(0, most)) shown.add(member.node.id);
    }
  } else {
    for (const member of all.slice(0, Math.max(0, budget))) shown.add(member.node.id);
  }

  const side = (which: -1 | 1): SceneGroup[] =>
    [...groups.values()]
      .filter((g) => g.side === which)
      .sort((a, b) => (a.kind < b.kind ? -1 : 1))
      .map((g) => {
        const cards = g.members.filter((m) => shown.has(m.node.id)).sort(byRank);
        const total = g.members.length;
        const words = kindWords(g.kind, view);
        const phrase = which < 0 ? `${words} this` : `this ${words}`;
        const rest = total - cards.length;
        return {
          kind: g.kind,
          color: relColor(g.kind, view),
          header_text: rest > 0 ? `${phrase} · ${cards.length} of ${total}` : `${phrase} · ${total}`,
          continued_text: `${phrase} · continued`,
          stub_text: rest > 0 ? (cards.length > 0 ? `+${rest} more` : `+${rest} ${phrase}`) : null,
          cards: cards.map((m) => asCard(m.node)),
        };
      });

  const seen = new Set<string>();
  const arcs: SceneArc[] = [];
  for (const edge of among) {
    if (!shown.has(edge.source_id) || !shown.has(edge.target_id)) continue;
    const key = `${edge.source_id}>${edge.target_id}:${edge.relationship_type}`;
    if (seen.has(key)) continue;
    seen.add(key);
    arcs.push({ from: String(edge.source_id), to: String(edge.target_id), color: relColor(edge.relationship_type, view) });
  }

  return {
    scene: { style: STYLE, focus: asCard(middle), left: side(-1), right: side(1), arcs },
    facts: {
      neighbours: all.length,
      shown: shown.size,
      truncated: result.truncated,
      second_links: secondLinks,
      arcs: arcs.length,
    },
  };
}

interface Entry {
  role: Role;
  id: string;
  text: string;
  color: string;
  link: string;
  h: number;
  starts_group: boolean;
}

function entriesOf(group: SceneGroup, heights: number[], st: Style): Entry[] {
  const out: Entry[] = [];
  if (group.cards.length > 0) {
    out.push({ role: 'header', id: group.kind, text: group.header_text, color: group.color, link: '', h: st.header_h, starts_group: true });
  }
  group.cards.forEach((card, i) => {
    out.push({
      role: 'card',
      id: card.id,
      text: card.name,
      color: card.color,
      link: group.color,
      h: heights[i] ?? st.pad_y * 2 + st.line,
      starts_group: false,
    });
  });
  if (group.stub_text !== null) {
    out.push({ role: 'stub', id: group.kind, text: group.stub_text, color: group.color, link: '', h: st.stub_h, starts_group: out.length === 0 });
  }
  return out;
}

function gapBefore(above: Role | undefined, entry: Entry, st: Style): number {
  if (above === undefined) return 0;
  if (above === 'header') return st.header_gap;
  return entry.starts_group ? st.group_gap : st.gap_y;
}

/** One side as columns, nearest the middle first, each entry with its distance from the top. */
function flow(groups: SceneGroup[], heights: number[][], st: Style): [Entry, number][][] {
  const cols: [Entry, number][][] = [[]];
  let y = 0;
  groups.forEach((group, g) => {
    const entries = entriesOf(group, heights[g] ?? [], st);
    entries.forEach((entry, i) => {
      let col = cols[cols.length - 1];
      let above = col[col.length - 1]?.[0].role;
      let need = gapBefore(above, entry, st) + entry.h;
      const next = entries[i + 1];
      if (entry.role === 'header' && next) need += st.header_gap + next.h;
      if (above !== undefined && y + need > st.max_col_h) {
        col = [];
        cols.push(col);
        y = 0;
        if (!entry.starts_group) {
          col.push([
            { role: 'header', id: group.kind, text: group.continued_text, color: group.color, link: '', h: st.header_h, starts_group: true },
            0,
          ]);
          y = st.header_h;
        }
      }
      above = col[col.length - 1]?.[0].role;
      const top = y + gapBefore(above, entry, st);
      y = top + entry.h;
      col.push([entry, top]);
    });
  });
  return cols.filter((col) => col.length > 0);
}

/** Place a scene, given how tall its text came out. */
export function layout(scene: Scene, heights: Heights): Layout {
  const st = scene.style;
  const fw = st.focus_w;
  const fh = heights.focus;
  const items: Item[] = [
    { role: 'focus', id: scene.focus.id, text: scene.focus.name, color: scene.focus.color, link: '', side: 0, col: 0, x: -fw / 2, y: -fh / 2, w: fw, h: fh },
  ];

  const sides: [-1 | 1, SceneGroup[], number[][]][] = [
    [-1, scene.left, heights.left],
    [1, scene.right, heights.right],
  ];
  for (const [side, groups, hs] of sides) {
    flow(groups, hs, st).forEach((col, k) => {
      const last = col[col.length - 1];
      const height = last[1] + last[0].h;
      const out = k * (st.card_w + st.wrap_gap);
      const x = side < 0 ? -fw / 2 - st.col_gap - st.card_w - out : fw / 2 + st.col_gap + out;
      for (const [entry, top] of col) {
        items.push({ role: entry.role, id: entry.id, text: entry.text, color: entry.color, link: entry.link, side, col: k, x, y: top - height / 2, w: st.card_w, h: entry.h });
      }
    });
  }

  const midY = (item: Item) => item.y + item.h / 2;
  const links: Link[] = [];
  for (const side of [-1, 1] as const) {
    const cards = items.filter((i) => i.role === 'card' && i.side === side).sort((a, b) => midY(a) - midY(b) || a.col - b.col);
    const n = cards.length;
    const room = Math.max(0, fh - 2 * st.fan_inset);
    const step = n > 1 ? Math.min(room / (n - 1), st.fan_step) : 0;
    cards.forEach((card, i) => {
      const fy = (i - (n - 1) / 2) * step;
      const edge = fw / 2 + st.focus_gap;
      const from: Point = side < 0 ? [card.x + card.w, midY(card)] : [edge, fy];
      const tip: Point = side < 0 ? [-edge, fy] : [card.x, midY(card)];
      const end: Point = [tip[0] - st.arrow_len, tip[1]];
      const dx = (end[0] - from[0]) / 2;
      links.push({ p: [from, [from[0] + dx, from[1]], [end[0] - dx, end[1]], end], tip, dir: 1, color: card.link, arc: false });
    });
  }

  const byId = new Map<string, Item>();
  const outermost = new Map<number, number>();
  for (const item of items) {
    if (item.role !== 'card') continue;
    if (!byId.has(item.id)) byId.set(item.id, item);
    outermost.set(item.side, Math.max(outermost.get(item.side) ?? 0, item.col));
  }
  let arcsLeftOut = 0;
  for (const arc of scene.arcs) {
    const a = byId.get(arc.from);
    const b = byId.get(arc.to);
    if (!a || !b || a.side !== b.side || a.col !== b.col || a.id === b.id) {
      arcsLeftOut += 1;
      continue;
    }
    const s = a.side;
    const x = s < 0 ? a.x : a.x + a.w;
    const ya = midY(a);
    const yb = midY(b);
    const room = a.col < (outermost.get(s) ?? 0) ? st.wrap_gap * 0.85 : st.arc_max;
    const bulge = Math.min(st.arc_min + Math.abs(yb - ya) * st.arc_slope, room);
    // A curve whose two middle points stand `c` out reaches three quarters of `c`.
    const c = (bulge * 4) / 3;
    const end: Point = [x + s * st.arrow_len, yb];
    links.push({ p: [[x, ya], [x + s * c, ya], [end[0] + s * c, yb], end], tip: [x, yb], dir: -s, color: arc.color, arc: true });
  }

  let [left, top, right, bottom] = [Infinity, Infinity, -Infinity, -Infinity];
  const cover = (x: number, y: number) => {
    left = Math.min(left, x);
    top = Math.min(top, y);
    right = Math.max(right, x);
    bottom = Math.max(bottom, y);
  };
  for (const item of items) {
    cover(item.x, item.y);
    cover(item.x + item.w, item.y + item.h);
  }
  for (const link of links) {
    if (link.arc) cover(link.p[0][0] + (link.p[1][0] - link.p[0][0]) * 0.75, link.p[0][1]);
  }

  return {
    items,
    links,
    bounds: [left - st.margin, top - st.margin, right + st.margin, bottom + st.margin],
    arcs_left_out: arcsLeftOut,
  };
}

/** The largest difference between two placings of one scene, in pixels, and where it is. */
export function largestDifference(a: Layout, b: Layout): { px: number; where: string } {
  let worst = { px: 0, where: 'nowhere' };
  const note = (px: number, where: string) => {
    if (px > worst.px) worst = { px, where };
  };
  if (a.items.length !== b.items.length) return { px: Infinity, where: `${a.items.length} items against ${b.items.length}` };
  if (a.links.length !== b.links.length) return { px: Infinity, where: `${a.links.length} links against ${b.links.length}` };
  a.items.forEach((item, i) => {
    const other = b.items[i];
    if (item.role !== other.role || item.id !== other.id) return note(Infinity, `item ${i} is ${item.role} ${item.id} against ${other.role} ${other.id}`);
    for (const key of ['x', 'y', 'w', 'h'] as const) note(Math.abs(item[key] - other[key]), `${item.role} "${item.text.slice(0, 24)}" ${key}`);
  });
  a.links.forEach((link, i) => {
    link.p.forEach((point, k) => {
      note(Math.abs(point[0] - b.links[i].p[k][0]), `link ${i} point ${k} x`);
      note(Math.abs(point[1] - b.links[i].p[k][1]), `link ${i} point ${k} y`);
    });
  });
  a.bounds.forEach((v, i) => note(Math.abs(v - b.bounds[i]), `bounds ${i}`));
  return worst;
}
