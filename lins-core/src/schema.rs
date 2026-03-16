use crate::connection::{ConnectionError, FalkorConnection};
use crate::graph_types::PropertyValue;
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
/// Uses verbose (non-compact) mode to get actual property names.
async fn sample_label_properties(
    conn: &FalkorConnection,
    graph: &str,
    cypher: &str,
) -> Result<Vec<PropertyInfo>, ConnectionError> {
    let raw = conn.query_raw_verbose(graph, cypher).await?;

    // In verbose mode, node results include property names directly.
    // The result format is: [header, data_rows, stats]
    // Each data row for a node query contains a node object.
    //
    // We need to parse the verbose node format to extract properties.
    // Verbose node: a nested structure with property key names (not IDs).

    let mut property_map: HashMap<String, PropertyInfo> = HashMap::new();

    if let Value::Array(top) = &raw {
        if top.len() >= 2 {
            if let Value::Array(rows) = &top[1] {
                for row in rows {
                    if let Value::Array(cols) = row {
                        for col in cols {
                            // In verbose mode, a node is returned as an array of properties
                            // where each property is [key_name, value]
                            extract_verbose_node_properties(col, &mut property_map);
                        }
                    }
                }
            }
        }
    }

    Ok(property_map.into_values().collect())
}

/// Extract properties from a verbose-mode node result.
///
/// Verbose node format can vary by FalkorDB version. Common formats:
/// - Array of [key, type, value] triples
/// - Nested structure with property map
///
/// We try multiple parsing strategies.
fn extract_verbose_node_properties(
    value: &Value,
    props: &mut HashMap<String, PropertyInfo>,
) {
    // Strategy 1: Node as array [id, labels, properties_list]
    if let Value::Array(node_arr) = value {
        // Look for property arrays in the node structure
        for element in node_arr {
            if let Value::Array(prop_or_subarray) = element {
                // Check if this looks like a property triple [key, type, value]
                if prop_or_subarray.len() == 2 || prop_or_subarray.len() == 3 {
                    if let Some(key) = value_to_string(&prop_or_subarray[0]) {
                        // Looks like a property entry
                        let val = if prop_or_subarray.len() == 3 {
                            redis_to_property_value(&prop_or_subarray[2])
                        } else {
                            redis_to_property_value(&prop_or_subarray[1])
                        };

                        let is_vec = val.is_vector();
                        let type_name = val.type_name().to_string();
                        let sample = val.display_short();

                        let entry = props.entry(key.clone()).or_insert_with(|| PropertyInfo {
                            key,
                            value_type: type_name.clone(),
                            sample_values: Vec::new(),
                            is_vector: is_vec.is_some(),
                            vector_dimensions: is_vec,
                        });

                        if entry.sample_values.len() < 5 && !sample.is_empty() && sample != "null" {
                            if !entry.sample_values.contains(&sample) {
                                entry.sample_values.push(sample);
                            }
                        }
                    }
                } else {
                    // Recurse into sub-arrays
                    for sub in prop_or_subarray {
                        extract_verbose_node_properties(sub, props);
                    }
                }
            }
        }
    }
}

/// Convert a Redis value to a PropertyValue for schema sampling.
fn redis_to_property_value(value: &Value) -> PropertyValue {
    match value {
        Value::Nil => PropertyValue::Null,
        Value::Int(i) => PropertyValue::Integer(*i),
        Value::Double(f) => PropertyValue::Float(*f),
        Value::Boolean(b) => PropertyValue::Boolean(*b),
        Value::BulkString(bytes) => {
            match String::from_utf8(bytes.clone()) {
                Ok(s) => PropertyValue::String(s),
                Err(_) => PropertyValue::Null,
            }
        }
        Value::SimpleString(s) => PropertyValue::String(s.clone()),
        Value::Array(items) => {
            let parsed: Vec<PropertyValue> = items.iter().map(redis_to_property_value).collect();
            PropertyValue::Array(parsed)
        }
        _ => PropertyValue::Null,
    }
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
