//! Cards in lanes: the queries behind an arrangement, and the order of its lanes.
//!
//! An arrangement places cards by a rule, so that where a card sits says something
//! (`config::Arrangement`). A lane is a node, with the cards that hold a link to it,
//! or a value of a property, with the cards that carry it. Everything here writes
//! or orders; nothing here talks to the database. Names that are written into a
//! query are checked by the caller's `Arrangement::lane_rule` and again here.

use crate::config::{FillRule, NodeHideRule};
use crate::graph_types::{GraphNode, PropertyValue};
use crate::view::{cypher_str, is_ident, node_named};

/// How many cards a lane shows before the rest wait behind a counted stub.
pub const DEFAULT_PER_LANE: usize = 8;
/// The most lanes one arrangement draws. A rule that names more is cut here, and the page says so.
pub const MAX_LANES: usize = 40;
/// The most cards one lane hands over at a time, whatever the page asks for.
pub const MAX_PER_LANE: usize = 400;

/// A number, wherever it is one: FalkorDB hands a sum back as text.
pub fn as_number(value: &PropertyValue) -> Option<f64> {
    match value {
        PropertyValue::Integer(i) => Some(*i as f64),
        PropertyValue::Float(f) if f.is_finite() => Some(*f),
        PropertyValue::String(s) => s.trim().parse::<f64>().ok().filter(|f| f.is_finite()),
        _ => None,
    }
}

/// How much of something is done, out of how much.
#[derive(Debug, Clone, Copy, PartialEq, serde::Serialize)]
pub struct Fill {
    pub done: f64,
    pub of: f64,
}

impl Fill {
    /// From the two numbers a rule names. Nothing when `of` is missing or not above zero.
    pub fn from_numbers(rule: &FillRule, of: Option<f64>, left: Option<f64>, done: Option<f64>) -> Option<Fill> {
        let of = of.filter(|n| *n > 0.0)?;
        let done = match (done, left) {
            (Some(done), _) if rule.done.is_some() => done,
            (_, Some(left)) if rule.left.is_some() => of - left,
            _ => return None,
        };
        Some(Fill { done: done.clamp(0.0, of), of })
    }

    /// A node's own fill, read from its properties.
    pub fn of_node(node: &GraphNode, rule: &FillRule) -> Option<Fill> {
        let read = |key: &Option<String>| key.as_ref().and_then(|k| node.properties.get(k)).and_then(as_number);
        Fill::from_numbers(rule, node.properties.get(&rule.of).and_then(as_number), read(&rule.left), read(&rule.done))
    }
}

/// The three sums a lane's fill is made from, as query text: `of`, `left`, `done`.
/// A property the rule does not name, or one that is not a plain name, sums to nothing.
fn fill_sums(alias: &str, rule: Option<&FillRule>) -> String {
    let sum = |key: Option<&String>| match key.filter(|k| is_ident(k)) {
        Some(key) => format!("sum(coalesce(toFloat({alias}.{key}), 0))"),
        None => "0".to_string(),
    };
    // FalkorDB requires distinct result-column names, including repeated zero placeholders
    // when no fill is configured. Aliases also cover two roles using the same property.
    let (of, left, done) = match rule {
        Some(rule) => (sum(Some(&rule.of)), sum(rule.left.as_ref()), sum(rule.done.as_ref())),
        None => ("0".to_string(), "0".to_string(), "0".to_string()),
    };
    format!("{of} AS fill_of, {left} AS fill_left, {done} AS fill_done")
}

/// How a lane's cards are ordered: by the property, the cards that lack it last, and by id
/// where that leaves a tie, so that the same lane always reads the same.
pub fn card_order(alias: &str, sort: Option<&str>, ascending: bool) -> String {
    match sort.filter(|key| is_ident(key)) {
        Some(key) => format!(
            "({alias}.{key} IS NULL), {alias}.{key} {}, id({alias})",
            if ascending { "ASC" } else { "DESC" }
        ),
        None => format!("id({alias})"),
    }
}

/// The nodes that are lanes. Nothing when no rule can be used.
pub fn lane_nodes_query(rules: &[NodeHideRule], visible: &str) -> Option<String> {
    let named = node_named("l", rules)?;
    Some(format!("MATCH (l) WHERE {visible} AND {named} RETURN l LIMIT {}", MAX_LANES + 1))
}

/// How many cards a lane that is a node holds, and the sums for its fill.
pub fn node_lane_totals_query(lane_id: i64, member: &str, visible: &str, fill: Option<&FillRule>) -> Option<String> {
    if !is_ident(member) {
        return None;
    }
    Some(format!(
        "MATCH (c)-[:`{member}`]->(l) WHERE id(l) = {lane_id} AND {visible} WITH DISTINCT c RETURN count(c), {}",
        fill_sums("c", fill)
    ))
}

/// Some of the cards of a lane that is a node, in order.
pub fn node_lane_cards_query(
    lane_id: i64,
    member: &str,
    visible: &str,
    order: &str,
    skip: usize,
    limit: usize,
) -> Option<String> {
    if !is_ident(member) {
        return None;
    }
    Some(format!(
        "MATCH (c)-[:`{member}`]->(l) WHERE id(l) = {lane_id} AND {visible} WITH DISTINCT c RETURN c ORDER BY {order} SKIP {skip} LIMIT {limit}"
    ))
}

/// The values of a property that are lanes: each with how many cards carry it and the sums for
/// its fill. The cards that lack the property come back under no value.
pub fn value_lanes_query(
    cards: &[NodeHideRule],
    property: &str,
    visible: &str,
    fill: Option<&FillRule>,
) -> Option<String> {
    if !is_ident(property) {
        return None;
    }
    let named = node_named("c", cards)?;
    Some(format!(
        "MATCH (c) WHERE {visible} AND {named} RETURN toString(c.{property}), count(c), {}",
        fill_sums("c", fill)
    ))
}

/// Some of the cards that carry one value of a property, or that lack it (`value` absent), in order.
pub fn value_lane_cards_query(
    cards: &[NodeHideRule],
    property: &str,
    value: Option<&str>,
    visible: &str,
    order: &str,
    skip: usize,
    limit: usize,
) -> Option<String> {
    if !is_ident(property) {
        return None;
    }
    let named = node_named("c", cards)?;
    let which = match value {
        Some(value) => format!("toString(c.{property}) = {}", cypher_str(value)),
        None => format!("c.{property} IS NULL"),
    };
    Some(format!(
        "MATCH (c) WHERE {visible} AND {named} AND {which} RETURN c ORDER BY {order} SKIP {skip} LIMIT {limit}"
    ))
}

/// Put the lanes that are nodes in order: by the property, the largest first unless `ascending`,
/// the lanes that lack it last, and by id where that leaves a tie.
pub fn order_lane_nodes(lanes: &mut [GraphNode], sort: Option<&str>, ascending: bool) {
    let key = |node: &GraphNode| sort.and_then(|key| node.properties.get(key)).and_then(as_number);
    lanes.sort_by(|a, b| match (key(a), key(b)) {
        (Some(x), Some(y)) => {
            let by_number = x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal);
            (if ascending { by_number } else { by_number.reverse() }).then(a.id.cmp(&b.id))
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => a.id.cmp(&b.id),
    });
}

/// Put the lanes that are values in order: by the value itself, the largest first unless
/// `ascending`, the lane of cards with no value last. Two values that are both numbers are
/// compared as numbers, so that 9 comes before 10.
pub fn order_values<T>(lanes: &mut [(Option<String>, T)], ascending: bool) {
    lanes.sort_by(|(a, _), (b, _)| match (a, b) {
        (Some(x), Some(y)) => {
            let by_value = match (x.trim().parse::<f64>(), y.trim().parse::<f64>()) {
                (Ok(m), Ok(n)) => m.partial_cmp(&n).unwrap_or(std::cmp::Ordering::Equal),
                _ => x.cmp(y),
            };
            if ascending {
                by_value
            } else {
                by_value.reverse()
            }
        }
        (Some(_), None) => std::cmp::Ordering::Less,
        (None, Some(_)) => std::cmp::Ordering::Greater,
        (None, None) => std::cmp::Ordering::Equal,
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(label: &str) -> NodeHideRule {
        NodeHideRule { label: label.to_string(), key: None, values: Vec::new() }
    }

    fn node(id: i64, props: &[(&str, PropertyValue)]) -> GraphNode {
        GraphNode {
            id,
            labels: vec!["Topic".to_string()],
            properties: props.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        }
    }

    fn fill(left: Option<&str>, done: Option<&str>) -> FillRule {
        FillRule { of: "total".to_string(), left: left.map(str::to_string), done: done.map(str::to_string) }
    }

    #[test]
    fn a_number_is_read_whether_it_came_as_a_number_or_as_text() {
        assert_eq!(as_number(&PropertyValue::Integer(3)), Some(3.0));
        assert_eq!(as_number(&PropertyValue::Float(2.5)), Some(2.5));
        assert_eq!(as_number(&PropertyValue::String("12.000000".into())), Some(12.0));
        assert_eq!(as_number(&PropertyValue::String("yes".into())), None);
        assert_eq!(as_number(&PropertyValue::Null), None);
    }

    #[test]
    fn a_fill_is_what_is_done_or_what_is_not_left() {
        let s = PropertyValue::Integer;
        let topic = node(1, &[("total", s(14)), ("open", s(11)), ("settled", s(3))]);
        assert_eq!(Fill::of_node(&topic, &fill(Some("open"), None)), Some(Fill { done: 3.0, of: 14.0 }));
        assert_eq!(Fill::of_node(&topic, &fill(None, Some("settled"))), Some(Fill { done: 3.0, of: 14.0 }));
        // More left than there is in all: drawn empty, never below it.
        let odd = node(2, &[("total", s(2)), ("open", s(5))]);
        assert_eq!(Fill::of_node(&odd, &fill(Some("open"), None)), Some(Fill { done: 0.0, of: 2.0 }));
    }

    #[test]
    fn a_card_without_the_numbers_is_drawn_plain() {
        let s = PropertyValue::Integer;
        assert_eq!(Fill::of_node(&node(1, &[("open", s(1))]), &fill(Some("open"), None)), None, "no total");
        assert_eq!(Fill::of_node(&node(1, &[("total", s(0)), ("open", s(0))]), &fill(Some("open"), None)), None, "nothing in all");
        assert_eq!(Fill::of_node(&node(1, &[("total", s(4))]), &fill(Some("open"), None)), None, "no count of what is left");
        assert_eq!(Fill::of_node(&node(1, &[("total", s(4)), ("open", s(1))]), &fill(None, None)), None, "the rule names neither");
    }

    #[test]
    fn cards_are_ordered_by_the_property_with_those_that_lack_it_last() {
        assert_eq!(card_order("c", Some("open"), false), "(c.open IS NULL), c.open DESC, id(c)");
        assert_eq!(card_order("c", Some("open"), true), "(c.open IS NULL), c.open ASC, id(c)");
        assert_eq!(card_order("c", None, false), "id(c)");
        assert_eq!(card_order("c", Some("open) DETACH DELETE c //"), false), "id(c)", "never written into a query");
    }

    #[test]
    fn the_queries_for_lanes_that_are_nodes() {
        assert_eq!(
            lane_nodes_query(&[rule("Review")], "true").as_deref(),
            Some("MATCH (l) WHERE true AND ('Review' IN labels(l)) RETURN l LIMIT 41")
        );
        assert_eq!(lane_nodes_query(&[rule("Review) DELETE")], "true"), None);
        assert_eq!(
            node_lane_totals_query(7, "BELONGS_TO", "true", Some(&fill(Some("open"), None))).as_deref(),
            Some(
                "MATCH (c)-[:`BELONGS_TO`]->(l) WHERE id(l) = 7 AND true WITH DISTINCT c RETURN count(c), \
                 sum(coalesce(toFloat(c.total), 0)) AS fill_of, sum(coalesce(toFloat(c.open), 0)) AS fill_left, 0 AS fill_done"
            )
        );
        assert_eq!(
            node_lane_cards_query(7, "BELONGS_TO", "true", "id(c)", 8, 8).as_deref(),
            Some("MATCH (c)-[:`BELONGS_TO`]->(l) WHERE id(l) = 7 AND true WITH DISTINCT c RETURN c ORDER BY id(c) SKIP 8 LIMIT 8")
        );
        assert_eq!(node_lane_cards_query(7, "BELONGS`TO", "true", "id(c)", 0, 8), None);
        assert_eq!(node_lane_totals_query(7, "a b", "true", None), None);
    }

    #[test]
    fn the_queries_for_lanes_that_are_values() {
        assert_eq!(
            value_lanes_query(&[rule("Topic")], "week", "true", None).as_deref(),
            Some("MATCH (c) WHERE true AND ('Topic' IN labels(c)) RETURN toString(c.week), count(c), 0 AS fill_of, 0 AS fill_left, 0 AS fill_done")
        );
        assert_eq!(
            value_lane_cards_query(&[rule("Topic")], "week", Some("it's"), "true", "id(c)", 0, 8).as_deref(),
            Some("MATCH (c) WHERE true AND ('Topic' IN labels(c)) AND toString(c.week) = 'it\\'s' RETURN c ORDER BY id(c) SKIP 0 LIMIT 8")
        );
        assert_eq!(
            value_lane_cards_query(&[rule("Topic")], "week", None, "true", "id(c)", 0, 8).as_deref(),
            Some("MATCH (c) WHERE true AND ('Topic' IN labels(c)) AND c.week IS NULL RETURN c ORDER BY id(c) SKIP 0 LIMIT 8")
        );
        assert_eq!(value_lanes_query(&[rule("Topic")], "week) //", "true", None), None);
        assert_eq!(value_lanes_query(&[], "week", "true", None), None, "no rule names the cards");
    }

    #[test]
    fn lanes_that_are_nodes_stand_largest_first_and_those_without_the_number_last() {
        let s = PropertyValue::Integer;
        let mut lanes = vec![node(1, &[("open", s(3))]), node(2, &[]), node(3, &[("open", s(40))]), node(4, &[("open", s(3))])];
        order_lane_nodes(&mut lanes, Some("open"), false);
        assert_eq!(lanes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![3, 1, 4, 2]);
        order_lane_nodes(&mut lanes, Some("open"), true);
        assert_eq!(lanes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1, 4, 3, 2]);
        order_lane_nodes(&mut lanes, None, false);
        assert_eq!(lanes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1, 2, 3, 4]);
    }

    #[test]
    fn lanes_that_are_values_stand_in_order_of_the_value_with_no_value_last() {
        let mut weeks = vec![(Some("2026-10-05".to_string()), 'c'), (None, 'x'), (Some("2026-09-14".to_string()), 'a')];
        order_values(&mut weeks, true);
        assert_eq!(weeks.iter().map(|(_, c)| *c).collect::<String>(), "acx");
        order_values(&mut weeks, false);
        assert_eq!(weeks.iter().map(|(_, c)| *c).collect::<String>(), "cax");
        let mut numbers = vec![(Some("10".to_string()), 'b'), (Some("9".to_string()), 'a')];
        order_values(&mut numbers, true);
        assert_eq!(numbers.iter().map(|(_, c)| *c).collect::<String>(), "ab", "9 before 10, as numbers");
    }
}
