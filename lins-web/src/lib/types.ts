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
