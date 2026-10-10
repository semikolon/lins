use crate::connection::{ConnectionError, FalkorConnection};
use chrono::{DateTime, Utc};
use redis::Value;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, instrument, warn};

/// Discovered schema for a single FalkorDB graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSchema {
    pub graph_name: String,
    pub labels: Vec<String>,
    pub relationship_types: Vec<String>,
    pub property_keys: Vec<String>,
    pub label_properties: HashMap<String, Vec<PropertyInfo>>,
    pub node_count: usize,
    pub edge_count: usize,
    pub discovered_at: DateTime<Utc>,
}

/// Information about a property discovered via sampling.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    pub key: String,
    pub value_type: String,
    pub sample_values: Vec<String>,
    pub is_vector: bool,
    pub vector_dimensions: Option<usize>,
}

/// Discover the full schema of a FalkorDB graph via introspection queries.
#[instrument(skip(conn), fields(graph = %graph))]
pub async fn discover_schema(
    conn: &FalkorConnection,
    graph: &str,
) -> Result<GraphSchema, ConnectionError> {
    debug!("Discovering schema for graph '{}'", graph);

    let labels = query_string_list(conn, graph, "CALL db.labels()").await?;
    debug!("Found {} labels", labels.len());

    let relationship_types =
        query_string_list(conn, graph, "CALL db.relationshipTypes()").await?;
    debug!("Found {} relationship types", relationship_types.len());

    let property_keys =
        query_string_list(conn, graph, "CALL db.propertyKeys()").await?;
    debug!("Found {} property keys", property_keys.len());

    let node_count = query_count(conn, graph, "MATCH (n) RETURN count(n)").await?;
    let edge_count =
        query_count(conn, graph, "MATCH ()-[r]->() RETURN count(r)").await?;
    debug!("Graph has {} nodes and {} edges", node_count, edge_count);

    // Sample properties per label
    let mut label_properties = HashMap::new();
    for label in &labels {
        let cypher = format!("MATCH (n:`{}`) RETURN n LIMIT 5", label);
        match sample_label_properties(conn, graph, &cypher).await {
            Ok(props) => {
                debug!("Sampled {} properties for label '{}'", props.len(), label);
                label_properties.insert(label.clone(), props);
            }
            Err(e) => {
                warn!("Failed to sample properties for label '{}': {}", label, e);
            }
        }
    }

    Ok(GraphSchema {
        graph_name: graph.to_string(),
        labels,
        relationship_types,
        property_keys,
        label_properties,
        node_count,
        edge_count,
        discovered_at: Utc::now(),
    })
}

/// Query FalkorDB for a list of strings (labels, types, keys).
async fn query_string_list(
    conn: &FalkorConnection,
    graph: &str,
    cypher: &str,
) -> Result<Vec<String>, ConnectionError> {
    let raw = conn.query_raw_verbose(graph, cypher).await?;

    let mut results = Vec::new();

    // FalkorDB verbose result: [header, data_rows, stats]
    if let Value::Array(top) = &raw {
        if top.len() >= 2 {
            if let Value::Array(rows) = &top[1] {
                for row in rows {
                    if let Value::Array(cols) = row {
                        for col in cols {
                            if let Some(s) = value_to_string(col) {
                                results.push(s);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(results)
}

/// Query FalkorDB for a single count value.
async fn query_count(
    conn: &FalkorConnection,
    graph: &str,
    cypher: &str,
) -> Result<usize, ConnectionError> {
    let raw = conn.query_raw_verbose(graph, cypher).await?;

    if let Value::Array(top) = &raw {
        if top.len() >= 2 {
            if let Value::Array(rows) = &top[1] {
                if let Some(Value::Array(cols)) = rows.first() {
                    if let Some(val) = cols.first() {
                        if let Some(n) = value_to_i64(val) {
                            return Ok(n as usize);
                        }
                    }
                }
            }
        }
    }

    Ok(0)
}

/// Sample properties from nodes of a given label.
///
/// The nodes are read with the same parser every other query uses, and their
/// properties counted. (An earlier version walked the raw reply by shape and
/// took a node's own three fields, `id`, `labels` and `properties`, for its
/// properties, so every label was recorded as having exactly those three.)
async fn sample_label_properties(
    conn: &FalkorConnection,
    graph: &str,
    cypher: &str,
) -> Result<Vec<PropertyInfo>, ConnectionError> {
    let sampled = conn.query_built(graph, cypher).await?;
    Ok(properties_of(&sampled.nodes))
}

/// The properties a set of nodes carries, with a few sample values of each.
pub(crate) fn properties_of(nodes: &[crate::graph_types::GraphNode]) -> Vec<PropertyInfo> {
    let mut property_map: HashMap<String, PropertyInfo> = HashMap::new();
    for node in nodes {
        for (key, val) in &node.properties {
            let is_vec = val.is_vector();
            let entry = property_map.entry(key.clone()).or_insert_with(|| PropertyInfo {
                key: key.clone(),
                value_type: val.type_name().to_string(),
                sample_values: Vec::new(),
                is_vector: is_vec.is_some(),
                vector_dimensions: is_vec,
            });
            if is_vec.is_some() {
                continue;
            }
            let sample = val.display_short();
            if entry.sample_values.len() < 5
                && !sample.is_empty()
                && sample != "null"
                && !entry.sample_values.contains(&sample)
            {
                entry.sample_values.push(sample);
            }
        }
    }
    let mut properties: Vec<PropertyInfo> = property_map.into_values().collect();
    properties.sort_by(|a, b| a.key.cmp(&b.key));
    properties
}

fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
        Value::SimpleString(s) => Some(s.clone()),
        Value::Int(i) => Some(i.to_string()),
        _ => None,
    }
}

fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Int(i) => Some(*i),
        Value::BulkString(bytes) => String::from_utf8(bytes.clone())
            .ok()
            .and_then(|s| s.parse().ok()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph_types::{GraphNode, PropertyValue};

    #[test]
    fn a_labels_properties_are_the_nodes_own_and_never_the_reply_fields() {
        let node = |id, props: &[(&str, PropertyValue)]| GraphNode {
            id,
            labels: vec!["Claim".into()],
            properties: props.iter().map(|(k, v)| (k.to_string(), v.clone())).collect(),
        };
        let text = |s: &str| PropertyValue::String(s.to_string());
        let found = properties_of(&[
            node(1, &[("claim_id", text("c1")), ("text", text("one")), ("embedding", text("[1, 2, 3, 4, 5, 6, 7, 8, 9]"))]),
            node(2, &[("claim_id", text("c2")), ("confidence", PropertyValue::Float(0.5))]),
        ]);
        let keys: Vec<&str> = found.iter().map(|p| p.key.as_str()).collect();
        assert_eq!(keys, vec!["claim_id", "confidence", "embedding", "text"]);
        assert!(!keys.contains(&"id") && !keys.contains(&"labels") && !keys.contains(&"properties"));
        let embedding = found.iter().find(|p| p.key == "embedding").unwrap();
        assert!(embedding.is_vector);
        assert_eq!(embedding.vector_dimensions, Some(9));
        assert!(embedding.sample_values.is_empty(), "a vector is never a sample value");
        let claim_id = found.iter().find(|p| p.key == "claim_id").unwrap();
        assert_eq!(claim_id.sample_values, vec!["c1", "c2"]);
    }
}
