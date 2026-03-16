use crate::config::ConnectionConfig;
use crate::graph_types::{GraphEdge, GraphNode, PropertyValue, QueryResult};
use redis::aio::ConnectionManager;
use redis::Value;
use std::collections::HashMap;
use std::time::Instant;
use thiserror::Error;
use tracing::{debug, instrument};

#[derive(Error, Debug)]
pub enum ConnectionError {
    #[error("Redis connection error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("Write operation blocked: query contains '{keyword}'. Lins is read-only.")]
    WriteBlocked { keyword: String },

    #[error("Failed to parse FalkorDB result: {0}")]
    ParseError(String),

    #[error("Graph not found: {0}")]
    GraphNotFound(String),
}

/// Wraps a redis ConnectionManager for FalkorDB communication.
#[derive(Clone)]
pub struct FalkorConnection {
    manager: ConnectionManager,
}

impl FalkorConnection {
    /// Connect to FalkorDB using the provided configuration.
    #[instrument(skip(config), fields(host = %config.host, port = config.port))]
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, ConnectionError> {
        let url = if let Some(ref pw) = config.password {
            format!("redis://:{}@{}:{}", pw, config.host, config.port)
        } else {
            format!("redis://{}:{}", config.host, config.port)
        };

        let client = redis::Client::open(url)?;
        let manager = ConnectionManager::new(client).await?;
        debug!("Connected to FalkorDB");
        Ok(Self { manager })
    }

    /// List all available graphs via GRAPH.LIST.
    #[instrument(skip(self))]
    pub async fn list_graphs(&self) -> Result<Vec<String>, ConnectionError> {
        let mut conn = self.manager.clone();
        let result: Value = redis::cmd("GRAPH.LIST")
            .query_async(&mut conn)
            .await?;

        match result {
            Value::Array(items) => {
                let mut graphs = Vec::new();
                for item in items {
                    if let Some(name) = value_to_string(&item) {
                        graphs.push(name);
                    }
                }
                Ok(graphs)
            }
            _ => Ok(Vec::new()),
        }
    }

    /// Execute a read-only Cypher query against a named graph.
    #[instrument(skip(self), fields(graph = %graph))]
    pub async fn query(
        &self,
        graph: &str,
        cypher: &str,
    ) -> Result<QueryResult, ConnectionError> {
        validate_read_only(cypher)?;

        let start = Instant::now();
        let raw = self.query_raw(graph, cypher).await?;
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        let (nodes, edges) = parse_graph_result(&raw)?;

        Ok(QueryResult {
            nodes,
            edges,
            query_time_ms: elapsed,
            source_graph: graph.to_string(),
        })
    }

    /// Execute a raw Cypher query and return the unparsed Redis Value.
    /// Uses verbose mode so labels and property keys are returned as strings.
    #[instrument(skip(self), fields(graph = %graph))]
    pub async fn query_raw(
        &self,
        graph: &str,
        cypher: &str,
    ) -> Result<Value, ConnectionError> {
        let mut conn = self.manager.clone();
        let result: Value = redis::cmd("GRAPH.QUERY")
            .arg(graph)
            .arg(cypher)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }

    /// Execute a count query and return the numeric result.
    /// Useful for queries like `MATCH (n) RETURN count(n)`.
    pub async fn query_count(
        &self,
        graph: &str,
        cypher: &str,
    ) -> Result<usize, ConnectionError> {
        let raw = self.query_raw_verbose(graph, cypher).await?;
        Ok(extract_count_from_result(&raw))
    }

    /// Execute a raw query without the --compact flag (for introspection calls).
    pub async fn query_raw_verbose(
        &self,
        graph: &str,
        cypher: &str,
    ) -> Result<Value, ConnectionError> {
        let mut conn = self.manager.clone();
        let result: Value = redis::cmd("GRAPH.QUERY")
            .arg(graph)
            .arg(cypher)
            .query_async(&mut conn)
            .await?;
        Ok(result)
    }
}

/// Keywords that indicate a write operation.
const WRITE_KEYWORDS: &[&str] = &[
    "CREATE", "MERGE", "DELETE", "DETACH", "SET ", "REMOVE", "DROP",
    "CALL DB.IDX", "GRAPH.DELETE",
];

/// Validate that a Cypher query does not contain write operations.
pub fn validate_read_only(cypher: &str) -> Result<(), ConnectionError> {
    let upper = cypher.to_uppercase();
    for keyword in WRITE_KEYWORDS {
        if upper.contains(keyword) {
            return Err(ConnectionError::WriteBlocked {
                keyword: keyword.trim().to_string(),
            });
        }
    }
    Ok(())
}

/// Parse a FalkorDB GRAPH.QUERY verbose result into nodes and edges.
///
/// Verbose result format: [headers, data_rows, stats]
///
/// Headers: [["col_name"], ...] — just column names, no type IDs.
///
/// Each data row column value for a node is:
///   [["id", node_id], ["labels", ["Label1", ...]], ["properties", [[key, value], ...]]]
///
/// Each data row column value for an edge is:
///   [["id", edge_id], ["type", "REL_TYPE"], ["src_node", src_id], ["dest_node", dst_id],
///    ["properties", [[key, value], ...]]]
///
/// Null values (e.g., OPTIONAL MATCH with no match) appear as Value::Nil.
fn parse_graph_result(
    value: &Value,
) -> Result<(Vec<GraphNode>, Vec<GraphEdge>), ConnectionError> {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    let top_array = match value {
        Value::Array(arr) => arr,
        _ => return Ok((nodes, edges)),
    };

    if top_array.len() < 2 {
        return Ok((nodes, edges));
    }

    let data_rows = match &top_array[1] {
        Value::Array(rows) => rows,
        _ => return Ok((nodes, edges)),
    };

    for row in data_rows {
        if let Value::Array(columns) = row {
            for col in columns {
                // Skip nulls from OPTIONAL MATCH
                if matches!(col, Value::Nil) {
                    continue;
                }

                // Try to identify what this column value is by looking for
                // marker fields: "labels" → node, "type" + "src_node" → edge
                if let Some(node) = parse_verbose_node(col) {
                    if !nodes.iter().any(|n: &GraphNode| n.id == node.id) {
                        nodes.push(node);
                    }
                } else if let Some(edge) = parse_verbose_edge(col) {
                    if !edges.iter().any(|e: &GraphEdge| e.id == edge.id) {
                        edges.push(edge);
                    }
                }
            }
        }
    }

    Ok((nodes, edges))
}

/// Parse a verbose-mode node.
///
/// Format: [["id", N], ["labels", ["L1", ...]], ["properties", [[k, v], ...]]]
fn parse_verbose_node(value: &Value) -> Option<GraphNode> {
    let arr = match value {
        Value::Array(a) => a,
        _ => return None,
    };

    let fields = parse_verbose_fields(arr);

    // Must have "labels" to be a node (edges have "type" instead)
    if !fields.contains_key("labels") {
        return None;
    }

    let id = fields
        .get("id")
        .and_then(|v| value_to_i64(v))?;

    let labels = match fields.get("labels") {
        Some(Value::Array(label_arr)) => label_arr
            .iter()
            .filter_map(|v| value_to_string(v))
            .collect(),
        _ => Vec::new(),
    };

    let properties = match fields.get("properties") {
        Some(props_val) => parse_verbose_properties(props_val),
        None => HashMap::new(),
    };

    Some(GraphNode {
        id,
        labels,
        properties,
    })
}

/// Parse a verbose-mode edge.
///
/// Format: [["id", N], ["type", "REL"], ["src_node", S], ["dest_node", D],
///          ["properties", [[k, v], ...]]]
fn parse_verbose_edge(value: &Value) -> Option<GraphEdge> {
    let arr = match value {
        Value::Array(a) => a,
        _ => return None,
    };

    let fields = parse_verbose_fields(arr);

    // Must have "type" and "src_node" to be an edge
    if !fields.contains_key("type") || !fields.contains_key("src_node") {
        return None;
    }

    let id = fields
        .get("id")
        .and_then(|v| value_to_i64(v))?;

    let relationship_type = fields
        .get("type")
        .and_then(|v| value_to_string(v))
        .unwrap_or_default();

    let source_id = fields
        .get("src_node")
        .and_then(|v| value_to_i64(v))?;

    let target_id = fields
        .get("dest_node")
        .and_then(|v| value_to_i64(v))?;

    let properties = match fields.get("properties") {
        Some(props_val) => parse_verbose_properties(props_val),
        None => HashMap::new(),
    };

    Some(GraphEdge {
        id,
        relationship_type,
        source_id,
        target_id,
        properties,
    })
}

/// Parse verbose-mode field pairs: [["key", value], ...] into a lookup map.
///
/// Each element is a 2-element array: [BulkString(key), value].
fn parse_verbose_fields(arr: &[Value]) -> HashMap<String, Value> {
    let mut map = HashMap::new();
    for item in arr {
        if let Value::Array(pair) = item {
            if pair.len() == 2 {
                if let Some(key) = value_to_string(&pair[0]) {
                    map.insert(key, pair[1].clone());
                }
            }
        }
    }
    map
}

/// Parse verbose-mode properties: [[key_name, value], ...]
fn parse_verbose_properties(value: &Value) -> HashMap<String, PropertyValue> {
    let mut props = HashMap::new();
    if let Value::Array(prop_list) = value {
        for prop in prop_list {
            if let Value::Array(pair) = prop {
                if pair.len() == 2 {
                    if let Some(key) = value_to_string(&pair[0]) {
                        let val = redis_value_to_property(&pair[1]);
                        props.insert(key, val);
                    }
                }
            }
        }
    }
    props
}

/// Convert a raw Redis Value to a PropertyValue using heuristics.
fn redis_value_to_property(value: &Value) -> PropertyValue {
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
            let parsed: Vec<PropertyValue> = items.iter().map(redis_value_to_property).collect();
            PropertyValue::Array(parsed)
        }
        _ => PropertyValue::Null,
    }
}

/// Extract a string from a Redis Value.
fn value_to_string(value: &Value) -> Option<String> {
    match value {
        Value::BulkString(bytes) => String::from_utf8(bytes.clone()).ok(),
        Value::SimpleString(s) => Some(s.clone()),
        Value::Int(i) => Some(i.to_string()),
        _ => None,
    }
}

/// Extract an i64 from a Redis Value.
fn value_to_i64(value: &Value) -> Option<i64> {
    match value {
        Value::Int(i) => Some(*i),
        Value::BulkString(bytes) => String::from_utf8(bytes.clone())
            .ok()
            .and_then(|s| s.parse().ok()),
        _ => None,
    }
}

/// Extract a count value from a verbose FalkorDB query result.
fn extract_count_from_result(value: &Value) -> usize {
    if let Value::Array(top) = value {
        if top.len() >= 2 {
            if let Value::Array(rows) = &top[1] {
                if let Some(Value::Array(cols)) = rows.first() {
                    if let Some(Value::Int(n)) = cols.first() {
                        return *n as usize;
                    }
                }
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_read_only_allows_match() {
        assert!(validate_read_only("MATCH (n) RETURN n").is_ok());
        assert!(validate_read_only("MATCH (n:Entity) WHERE n.name = 'test' RETURN n LIMIT 10").is_ok());
        assert!(validate_read_only("CALL db.labels()").is_ok());
    }

    #[test]
    fn test_validate_read_only_blocks_writes() {
        assert!(validate_read_only("CREATE (n:Test {name: 'x'})").is_err());
        assert!(validate_read_only("MATCH (n) DELETE n").is_err());
        assert!(validate_read_only("MATCH (n) SET n.name = 'y'").is_err());
        assert!(validate_read_only("MERGE (n:Test {name: 'x'})").is_err());
        assert!(validate_read_only("MATCH (n) REMOVE n.name").is_err());
        assert!(validate_read_only("DROP INDEX ON :Test(name)").is_err());
    }
}
