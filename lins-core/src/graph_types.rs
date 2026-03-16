use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A property value from a FalkorDB node or edge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum PropertyValue {
    Null,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Array(Vec<PropertyValue>),
    Map(HashMap<String, PropertyValue>),
}

impl PropertyValue {
    /// Returns a human-readable type name for schema introspection.
    pub fn type_name(&self) -> &'static str {
        match self {
            PropertyValue::Null => "Null",
            PropertyValue::Boolean(_) => "Boolean",
            PropertyValue::Integer(_) => "Integer",
            PropertyValue::Float(_) => "Float",
            PropertyValue::String(_) => "String",
            PropertyValue::Array(_) => "Array",
            PropertyValue::Map(_) => "Map",
        }
    }

    /// Try to display this value as a short string (for autocomplete samples).
    pub fn display_short(&self) -> String {
        match self {
            PropertyValue::Null => "null".to_string(),
            PropertyValue::Boolean(b) => b.to_string(),
            PropertyValue::Integer(i) => i.to_string(),
            PropertyValue::Float(f) => format!("{f:.4}"),
            PropertyValue::String(s) => {
                if s.len() > 80 {
                    format!("{}...", &s[..77])
                } else {
                    s.clone()
                }
            }
            PropertyValue::Array(a) => format!("[{} items]", a.len()),
            PropertyValue::Map(m) => format!("{{{} keys}}", m.len()),
        }
    }

    /// Check if this value looks like a vector embedding (array of floats).
    pub fn is_vector(&self) -> Option<usize> {
        if let PropertyValue::Array(items) = self {
            if items.len() >= 8 && items.iter().all(|v| matches!(v, PropertyValue::Float(_))) {
                return Some(items.len());
            }
        }
        None
    }
}

/// A node from a FalkorDB graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: i64,
    pub labels: Vec<String>,
    pub properties: HashMap<String, PropertyValue>,
}

/// An edge from a FalkorDB graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: i64,
    pub relationship_type: String,
    pub source_id: i64,
    pub target_id: i64,
    pub properties: HashMap<String, PropertyValue>,
}

/// The result of executing a Cypher query against FalkorDB.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
    pub query_time_ms: f64,
    pub source_graph: String,
}
