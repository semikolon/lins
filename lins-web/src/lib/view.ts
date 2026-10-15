import type { Fill, GraphNode, GraphView, PropertyValue } from './types';

/** The property names tried as a node's name when its label has no caption. */
const NAME_KEYS = ['name', 'title', 'short_title', 'summary', 'label', 'text'];

const PALETTE = [
  '#2d6a4f', '#7c3aed', '#b45309', '#0e7490', '#be185d',
  '#4338ca', '#059669', '#d97706', '#7c2d12', '#1d4ed8',
  '#9333ea', '#dc2626', '#0284c7', '#65a30d', '#c026d3',
];

export const DEFAULT_EDGE_COLOR = '#4a5568';

function captionKeys(node: GraphNode, view: GraphView | null): string[] {
  for (const label of node.labels) {
    const caption = view?.styling.labels[label]?.caption;
    if (typeof caption === 'string') return [caption];
    if (Array.isArray(caption) && caption.length > 0) return caption;
  }
  return NAME_KEYS;
}

/** A node's name in full: its caption property, or failing that its label. */
export function nodeFullName(node: GraphNode, view: GraphView | null): string {
  for (const key of captionKeys(node, view)) {
    const value = node.properties[key];
    if (typeof value === 'string' && value.trim() !== '') return value.trim();
    if (typeof value === 'number') return String(value);
  }
  return node.labels[0] ?? `Node ${node.id}`;
}

/**
 * What a node's card says when the pointer rests on it: each property its label's `tooltip` names
 * that the node has, in order, every line of it a part of its own. A value that only repeats the
 * node's name is left out. Nothing when the label names none.
 */
export function tooltipLines(node: GraphNode, view: GraphView | null): string[] {
  const keys = node.labels.map((label) => view?.styling.labels[label]?.tooltip).find((k) => k && k.length > 0);
  if (!keys) return [];
  const name = nodeFullName(node, view).trim();
  const lines: string[] = [];
  for (const key of keys) {
    const value = node.properties[key];
    const text = typeof value === 'number' ? String(value) : typeof value === 'string' ? value.trim() : '';
    if (text === '' || text === name) continue;
    lines.push(...text.split('\n').map((line) => line.trim()).filter((line) => line !== ''));
  }
  return lines;
}

/** A node's name cut to fit beside a dot on the canvas. */
export function nodeName(node: GraphNode, view: GraphView | null, max = 64): string {
  const full = nodeFullName(node, view).replace(/\s+/g, ' ');
  if (full.length <= max) return full;
  const cut = full.slice(0, max);
  const lastSpace = cut.lastIndexOf(' ');
  return `${lastSpace > max * 0.6 ? cut.slice(0, lastSpace) : cut}…`;
}

export function labelColor(label: string, view: GraphView | null, labels: string[]): string {
  const configured = view?.styling.labels[label]?.color;
  if (configured) return configured;
  const index = labels.indexOf(label);
  return PALETTE[index >= 0 ? index % PALETTE.length : 0];
}

export function relColor(type: string, view: GraphView | null): string {
  return view?.styling.relationships[type]?.color ?? DEFAULT_EDGE_COLOR;
}

function asNumber(value: PropertyValue | undefined): number | null {
  if (typeof value === 'number' && Number.isFinite(value)) return value;
  if (typeof value === 'string' && value.trim() !== '' && Number.isFinite(Number(value))) return Number(value);
  return null;
}

/**
 * How much of a node is done, from the two numbers the view's fill rule names. Nothing when the
 * view has no rule, the node lacks a number, or there is nothing in all. The server's rule for a
 * lane (lins-core, `lanes::Fill`) is this one.
 */
export function fillOf(node: GraphNode, view: GraphView | null): Fill | null {
  const rule = view?.display.fill;
  if (!rule) return null;
  const of = asNumber(node.properties[rule.of]);
  if (of === null || of <= 0) return null;
  const done = rule.done ? asNumber(node.properties[rule.done]) : null;
  const left = rule.left ? asNumber(node.properties[rule.left]) : null;
  const value = done !== null ? done : left !== null ? of - left : null;
  if (value === null) return null;
  return { done: Math.min(of, Math.max(0, value)), of };
}

/** A fill as a whole percent, for a width. */
export function fillPercent(fill: Fill | null | undefined): number | null {
  return fill && fill.of > 0 ? Math.round((fill.done / fill.of) * 100) : null;
}

/** "BELONGS_TO" as words, unless the view gives the kind a name of its own. */
export function relWords(type: string, view: GraphView | null): string {
  return view?.styling.relationships[type]?.label ?? type.toLowerCase().replaceAll('_', ' ');
}

/** Should this property be listed? Vectors are taken out by the server; this is the view's own list. */
export function shownProperty(key: string, view: GraphView | null): boolean {
  if (view?.display.properties.hide.includes(key)) return false;
  return !key.endsWith('_embedding');
}

/** A line saying what the view hides, for the status bar. Empty when it hides nothing. */
export function hiddenSummary(view: GraphView | null): string {
  if (!view) return '';
  const parts: string[] = [];
  for (const rule of view.display.nodes.hide) {
    parts.push(rule.key && rule.values.length > 0 ? `${rule.label} (${rule.values.join(', ')})` : rule.label);
  }
  parts.push(...view.display.relationships.hide);
  return parts.join(' · ');
}
