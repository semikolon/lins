export interface GraphNode {
  id: number;
  labels: string[];
  properties: Record<string, PropertyValue>;
}

export interface GraphEdge {
  id: number;
  relationship_type: string;
  source_id: number;
  target_id: number;
  properties: Record<string, PropertyValue>;
}

export type PropertyValue =
  | string
  | number
  | boolean
  | null
  | PropertyValue[]
  | { [key: string]: PropertyValue };

export interface QueryResult {
  nodes: GraphNode[];
  edges: GraphEdge[];
  query_time_ms: number;
  source_graph: string;
}

export interface GraphSchema {
  graph_name: string;
  labels: string[];
  relationship_types: string[];
  property_keys: string[];
  node_count: number;
  edge_count: number;
}

export interface GraphInfo {
  name: string;
  node_count: number;
  edge_count: number;
}

export interface Suggestion {
  suggestion_type: 'label' | 'relationship_type' | 'property_value' | 'property_key';
  value: string;
  cypher: string;
  count?: number;
}

export interface LabelStyle {
  color?: string | null;
  /** The property that names a node: one key, or several tried in order. */
  caption?: string | string[] | null;
  size?: number | string | null;
  /** The properties shown when the pointer rests on a node's card, in order. */
  tooltip?: string[] | null;
}

export interface RelationshipStyle {
  color?: string | null;
  label?: string | null;
  style?: string | null;
}

export interface NodeHideRule {
  label: string;
  key?: string | null;
  values: string[];
}

/**
 * One way of laying a graph out as cards in lanes. A lane is a node (`lanes` and `member`: a card
 * holds a link of that kind to its lane) or a value (`cards` and `lane_property`).
 */
export interface Arrangement {
  name: string;
  lanes: NodeHideRule[];
  member?: string | null;
  cards: NodeHideRule[];
  lane_property?: string | null;
  sort?: string | null;
  ascending: boolean;
  lane_sort?: string | null;
  lane_ascending: boolean;
  per_lane?: number | null;
}

/** How full a card is drawn: `done` out of `of`, or what is not `left` out of `of`. */
export interface FillRule {
  of: string;
  left?: string | null;
  done?: string | null;
}

export interface Fill {
  done: number;
  of: number;
}

/** One lane of an arrangement: a node or a value, how many cards it holds, and the first of them. */
export interface Lane {
  key: string;
  node?: GraphNode | null;
  value?: string | null;
  total: number;
  fill?: Fill | null;
  cards: GraphNode[];
}

/** An arrangement as the server returns it: lanes in order, and the links among the cards shown. */
export interface Board {
  name: string;
  index: number;
  by: 'nodes' | 'values';
  property?: string | null;
  lanes: Lane[];
  links: GraphEdge[];
  card_count: number;
  shown_count: number;
  lanes_cut: boolean;
  total_node_count: number;
  query_time_ms: number;
}

/** What the middle of the page shows: one arrangement, one node and its ties, or the free picture. */
export type Mode = { kind: 'free' } | { kind: 'lanes'; index: number } | { kind: 'scene' };

/** How one graph is shown: the server's config with its overlays laid on top. */
export interface GraphView {
  graph: string;
  styling: {
    labels: Record<string, LabelStyle>;
    relationships: Record<string, RelationshipStyle>;
  };
  display: {
    properties: { show: string[]; hide: string[] };
    nodes: { hide: NodeHideRule[]; first?: NodeHideRule[]; max_nodes?: number | null; overview?: number | null; neighbours?: number | null };
    relationships: { hide: string[] };
    arrangements?: Arrangement[];
    fill?: FillRule | null;
  };
}

/** A view's nodes and links, with what it left out. */
export interface ViewResult extends QueryResult {
  total_node_count: number;
  hidden_node_count: number;
  mode: 'all' | 'overview' | 'first' | 'around';
  truncated: boolean;
  center?: number;
  neighbour_count?: number;
}

export interface ViewInfo {
  mode: 'all' | 'overview' | 'first' | 'around' | 'query';
  total: number;
  hidden: number;
  truncated: boolean;
}
