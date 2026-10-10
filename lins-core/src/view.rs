//! What a view of a graph leaves out, and how the queries that build it are written.
//!
//! A view is the answer to "what should a person see first": not the vectors,
//! not the bookkeeping fields, and not the kinds of node the graph's own
//! settings say are scaffolding. Everything here is driven by `GraphView`,
//! so nothing about any one graph is written into the code.

use crate::config::{GraphView, NodeHideRule};
use crate::graph_types::{GraphEdge, GraphNode, PropertyValue, QueryResult};
use std::collections::{HashMap, HashSet};

/// Is this safe to write into a query as a label, a type or a property key?
pub fn is_ident(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// A reader's words as a Cypher string literal. Backslashes and quotes are
/// escaped and control characters dropped, so the words cannot end the literal.
pub fn cypher_str(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('\'');
    for c in text.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\'' => out.push_str("\\'"),
            c if c.is_control() => {}
            c => out.push(c),
        }
    }
    out.push('\'');
    out
}

/// A list of whole numbers as a Cypher list.
pub fn cypher_ids(ids: &[i64]) -> String {
    let parts: Vec<String> = ids.iter().map(i64::to_string).collect();
    format!("[{}]", parts.join(", "))
}

/// The condition a node must meet to be shown, written for the node bound to `alias`.
///
/// A rule with no key hides the whole label. A rule with a key hides the nodes
/// whose key holds one of the values; a node that lacks the key is shown.
/// A rule whose label or key is not a plain name is skipped, never interpolated.
pub fn node_visible(alias: &str, rules: &[NodeHideRule]) -> String {
    let mut parts = Vec::new();
    for rule in rules {
        if !is_ident(&rule.label) {
            continue;
        }
        let has_label = format!("{} IN labels({alias})", cypher_str(&rule.label));
        match &rule.key {
            None => parts.push(format!("NOT {has_label}")),
            Some(key) if is_ident(key) && !rule.values.is_empty() => {
                let values: Vec<String> = rule.values.iter().map(|v| cypher_str(v)).collect();
                parts.push(format!(
                    "NOT ({has_label} AND coalesce(toString({alias}.{key}), '') IN [{}])",
                    values.join(", ")
                ));
            }
            Some(_) => {}
        }
    }
    if parts.is_empty() {
        "true".to_string()
    } else {
        parts.join(" AND ")
    }
}

/// The condition a relationship must meet to be shown, for the one bound to `alias`.
pub fn rel_visible(alias: &str, hidden: &[String]) -> String {
    let types: Vec<String> = hidden.iter().filter(|t| is_ident(t)).map(|t| cypher_str(t)).collect();
    if types.is_empty() {
        "true".to_string()
    } else {
        format!("NOT type({alias}) IN [{}]", types.join(", "))
    }
}

/// Would this node be left out of the view? The same rule as `node_visible`, in code.
pub fn node_hidden(node: &GraphNode, rules: &[NodeHideRule]) -> bool {
    rules.iter().any(|rule| {
        node.labels.contains(&rule.label)
            && match &rule.key {
                None => true,
                Some(key) => match node.properties.get(key) {
                    Some(PropertyValue::String(s)) => rule.values.contains(s),
                    Some(PropertyValue::Null) | None => false,
                    Some(other) => rule.values.contains(&other.display_short()),
                },
            }
    })
}

fn strip_map(properties: &mut HashMap<String, PropertyValue>, hide: &HashSet<&str>) {
    properties.retain(|key, value| !hide.contains(key.as_str()) && value.is_vector().is_none());
}

/// Take out of a result what no person reads: vectors, wherever they are and
/// whatever they are called, and the properties the view hides.
pub fn strip(result: &mut QueryResult, view: &GraphView) {
    let hide: HashSet<&str> = view.display.properties.hide.iter().map(String::as_str).collect();
    for node in &mut result.nodes {
        strip_map(&mut node.properties, &hide);
    }
    for edge in &mut result.edges {
        strip_map(&mut edge.properties, &hide);
    }
}

/// The best-connected nodes, from the pairs a set of links joins.
/// Ties are broken by id so the same graph always opens on the same view.
pub fn best_connected(pairs: &[Vec<i64>], limit: usize) -> Vec<i64> {
    let mut degree: HashMap<i64, usize> = HashMap::new();
    for pair in pairs {
        if let [a, b] = pair[..] {
            *degree.entry(a).or_insert(0) += 1;
            *degree.entry(b).or_insert(0) += 1;
        }
    }
    let mut ranked: Vec<(i64, usize)> = degree.into_iter().collect();
    ranked.sort_by(|x, y| y.1.cmp(&x.1).then(x.0.cmp(&y.0)));
    ranked.into_iter().take(limit).map(|(id, _)| id).collect()
}

/// Take out the nodes no link in the result touches. A well-connected node whose
/// neighbours all fell outside an overview would otherwise sit in it alone.
pub fn drop_unjoined(result: &mut QueryResult) {
    let joined: HashSet<i64> = result.edges.iter().flat_map(|e| [e.source_id, e.target_id]).collect();
    result.nodes.retain(|n| joined.contains(&n.id));
}

/// Keep only the links whose two ends are both among the nodes.
pub fn links_among(edges: Vec<GraphEdge>, nodes: &[GraphNode]) -> Vec<GraphEdge> {
    let ids: HashSet<i64> = nodes.iter().map(|n| n.id).collect();
    edges
        .into_iter()
        .filter(|e| ids.contains(&e.source_id) && ids.contains(&e.target_id))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DisplayConfig, StylingConfig};

    fn rule(label: &str, key: Option<&str>, values: &[&str]) -> NodeHideRule {
        NodeHideRule {
            label: label.to_string(),
            key: key.map(str::to_string),
            values: values.iter().map(|v| v.to_string()).collect(),
        }
    }

    fn node(id: i64, label: &str, props: &[(&str, PropertyValue)]) -> GraphNode {
        GraphNode {
            id,
            labels: vec![label.to_string()],
            properties: props.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        }
    }

    #[test]
    fn a_readers_words_cannot_end_the_literal() {
        assert_eq!(cypher_str("plain"), "'plain'");
        assert_eq!(cypher_str("it's"), "'it\\'s'");
        assert_eq!(cypher_str("a\\'b"), "'a\\\\\\'b'");
        assert_eq!(cypher_str("line\nbreak"), "'linebreak'");
        assert_eq!(cypher_str("') DETACH DELETE n //"), "'\\') DETACH DELETE n //'");
    }

    #[test]
    fn only_plain_names_are_written_into_a_query() {
        assert!(is_ident("claim_kind"));
        assert!(is_ident("_x1"));
        assert!(!is_ident("1x"));
        assert!(!is_ident("a b"));
        assert!(!is_ident("x) DELETE"));
        assert!(!is_ident(""));
    }

    #[test]
    fn the_condition_hides_a_label_or_only_some_of_its_nodes() {
        assert_eq!(node_visible("n", &[]), "true");
        assert_eq!(node_visible("n", &[rule("Question", None, &[])]), "NOT 'Question' IN labels(n)");
        assert_eq!(
            node_visible("m", &[rule("Claim", Some("claim_kind"), &["cq_positive", "cq_negative"])]),
            "NOT ('Claim' IN labels(m) AND coalesce(toString(m.claim_kind), '') IN ['cq_positive', 'cq_negative'])"
        );
    }

    #[test]
    fn a_rule_that_is_not_a_plain_name_is_skipped_and_never_written() {
        let bad = [rule("Claim) DELETE", None, &[]), rule("Claim", Some("k) OR true"), &["x"])];
        assert_eq!(node_visible("n", &bad), "true");
        assert_eq!(rel_visible("r", &["OK_TYPE".into(), "bad type".into()]), "NOT type(r) IN ['OK_TYPE']");
        assert_eq!(rel_visible("r", &[]), "true");
    }

    #[test]
    fn the_rule_in_code_agrees_with_the_rule_in_the_query() {
        let rules = [rule("Claim", Some("claim_kind"), &["cq_positive"]), rule("Question", None, &[])];
        let s = |v: &str| PropertyValue::String(v.to_string());
        assert!(node_hidden(&node(1, "Claim", &[("claim_kind", s("cq_positive"))]), &rules));
        assert!(!node_hidden(&node(2, "Claim", &[("claim_kind", s("extracted"))]), &rules));
        assert!(!node_hidden(&node(3, "Claim", &[]), &rules), "a node that lacks the key is shown");
        assert!(node_hidden(&node(4, "Question", &[]), &rules));
        assert!(!node_hidden(&node(5, "Source", &[]), &rules));
    }

    #[test]
    fn vectors_go_whatever_they_are_called_and_hidden_keys_go() {
        let vector = PropertyValue::Array((0..16).map(|i| PropertyValue::Float(i as f64)).collect());
        let short_list = PropertyValue::Array(vec![PropertyValue::Float(1.0), PropertyValue::Float(2.0)]);
        let mut result = QueryResult {
            nodes: vec![node(
                1,
                "Claim",
                &[
                    ("embedding", vector.clone()),
                    ("anything_else", vector.clone()),
                    ("pair", short_list.clone()),
                    ("group_id", PropertyValue::String("g".into())),
                    ("text", PropertyValue::String("kept".into())),
                ],
            )],
            edges: vec![GraphEdge {
                id: 9,
                relationship_type: "SUPPORTS".into(),
                source_id: 1,
                target_id: 1,
                properties: [("fact_embedding".to_string(), vector), ("reasoning".to_string(), PropertyValue::String("kept".into()))]
                    .into_iter()
                    .collect(),
            }],
            query_time_ms: 0.0,
            source_graph: "g".into(),
        };
        let mut display = DisplayConfig::default();
        display.properties.hide = vec!["group_id".into()];
        let view = GraphView { graph: "g".into(), styling: StylingConfig::default(), display };

        strip(&mut result, &view);

        let mut kept: Vec<&str> = result.nodes[0].properties.keys().map(String::as_str).collect();
        kept.sort();
        assert_eq!(kept, vec!["pair", "text"]);
        assert_eq!(result.edges[0].properties.keys().collect::<Vec<_>>(), vec!["reasoning"]);
    }

    #[test]
    fn the_best_connected_come_first_and_ties_go_by_id() {
        let pairs = vec![vec![1, 2], vec![1, 3], vec![4, 1], vec![2, 3], vec![9, 8]];
        assert_eq!(best_connected(&pairs, 3), vec![1, 2, 3]);
        assert_eq!(best_connected(&pairs, 10), vec![1, 2, 3, 4, 8, 9]);
        assert!(best_connected(&[], 5).is_empty());
    }

    #[test]
    fn a_node_nothing_joins_is_taken_out_of_an_overview() {
        let mut result = QueryResult {
            nodes: vec![node(1, "A", &[]), node(2, "A", &[]), node(3, "A", &[])],
            edges: vec![GraphEdge {
                id: 1,
                relationship_type: "R".into(),
                source_id: 1,
                target_id: 2,
                properties: HashMap::new(),
            }],
            query_time_ms: 0.0,
            source_graph: "g".into(),
        };
        drop_unjoined(&mut result);
        assert_eq!(result.nodes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn a_link_with_an_end_outside_the_nodes_is_dropped() {
        let nodes = vec![node(1, "A", &[]), node(2, "A", &[])];
        let edge = |id, s, t| GraphEdge {
            id,
            relationship_type: "R".into(),
            source_id: s,
            target_id: t,
            properties: HashMap::new(),
        };
        let kept = links_among(vec![edge(1, 1, 2), edge(2, 1, 3), edge(3, 4, 2)], &nodes);
        assert_eq!(kept.iter().map(|e| e.id).collect::<Vec<_>>(), vec![1]);
    }
}
