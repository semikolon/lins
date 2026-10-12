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
                // Counted and cut in letters. A cut at a byte position can land inside a letter
                // (ö is two bytes), and cutting a text there is a panic.
                if s.chars().count() > 80 {
                    let head: String = s.chars().take(77).collect();
                    format!("{head}...")
                } else {
                    s.clone()
                }
            }
            PropertyValue::Array(a) => format!("[{} items]", a.len()),
            PropertyValue::Map(m) => format!("{{{} keys}}", m.len()),
        }
    }

    /// Check if this value looks like a vector embedding: an array of floats,
    /// or the text FalkorDB prints one as (`[-0.0002, 0.0201, ...]`), which is
    /// how a vector arrives when a whole node is returned.
    pub fn is_vector(&self) -> Option<usize> {
        match self {
            PropertyValue::Array(items)
                if items.len() >= 8 && items.iter().all(|v| matches!(v, PropertyValue::Float(_))) =>
            {
                Some(items.len())
            }
            PropertyValue::String(text) => vector_text_len(text),
            _ => None,
        }
    }
}

/// How many numbers a printed vector holds, or None when the text is not one.
/// Eight or more numbers between brackets, of which the first eight are read.
fn vector_text_len(text: &str) -> Option<usize> {
    let text = text.trim();
    let inner = text
        .strip_prefix('[')
        .and_then(|t| t.strip_suffix(']'))
        .or_else(|| text.strip_prefix('<').and_then(|t| t.strip_suffix('>')))?;
    let mut parts = inner.split(',');
    for _ in 0..8 {
        parts.next()?.trim().parse::<f64>().ok()?;
    }
    Some(8 + parts.count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_vector_is_known_as_an_array_and_as_the_text_it_is_printed_as() {
        let array = PropertyValue::Array((0..16).map(|i| PropertyValue::Float(i as f64)).collect());
        assert_eq!(array.is_vector(), Some(16));
        let printed = PropertyValue::String("[-0.000203, -0.000150, -0.022070, 0.020126, -0.000804, 0.075813, 0.043536, -0.001, 0.5]".into());
        assert_eq!(printed.is_vector(), Some(9));
        let angle = PropertyValue::String("<1, 2, 3, 4, 5, 6, 7, 8>".into());
        assert_eq!(angle.is_vector(), Some(8));
    }

    #[test]
    fn a_long_text_is_cut_between_letters_and_never_inside_one() {
        // Byte 77 of this text is the second byte of an ö. A cut by bytes panicked there, and a
        // graph that held such a text could not be opened at all.
        let text = format!("{}ö and the rest of a sentence that runs well past eighty letters", "a".repeat(76));
        assert!(!text.is_char_boundary(77));
        let short = PropertyValue::String(text).display_short();
        assert_eq!(short, format!("{}ö...", "a".repeat(76)));
        // Eighty letters are shown whole, however many bytes they take.
        let eighty = "ö".repeat(80);
        assert_eq!(PropertyValue::String(eighty.clone()).display_short(), eighty);
        assert_eq!(PropertyValue::String("ö".repeat(81)).display_short(), format!("{}...", "ö".repeat(77)));
    }

    #[test]
    fn ordinary_text_and_short_lists_are_not_vectors() {
        for text in ["[a claim in brackets]", "[1, 2, 3]", "1, 2, 3, 4, 5, 6, 7, 8", "", "[1, 2, three, 4, 5, 6, 7, 8]"] {
            assert_eq!(PropertyValue::String(text.into()).is_vector(), None, "{text}");
        }
        let short = PropertyValue::Array(vec![PropertyValue::Float(1.0), PropertyValue::Float(2.0)]);
        assert_eq!(short.is_vector(), None);
        let mixed = PropertyValue::Array((0..9).map(PropertyValue::Integer).collect());
        assert_eq!(mixed.is_vector(), None);
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
