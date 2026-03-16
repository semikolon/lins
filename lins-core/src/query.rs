use crate::schema::GraphSchema;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// RediSearch reserved words that crash fulltext queries when used as entity names.
const REDISEARCH_RESERVED: &[&str] = &[
    "MAP", "REDUCE", "FILTER", "APPLY", "LIMIT",
    "LOAD", "AS", "GROUPBY", "SORTBY",
];

/// An autocomplete suggestion from the vocabulary index.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// The suggestion type.
    #[serde(rename = "type")]
    pub suggestion_type: SuggestionType,

    /// The display value for this suggestion.
    pub value: String,

    /// Optional detail (e.g., node count for labels).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// A pre-built Cypher query for this suggestion.
    pub cypher: String,
}

/// The type of suggestion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    Label,
    RelationshipType,
    PropertyKey,
    PropertyValue,
}

/// An in-memory index of vocabulary from a graph schema for instant autocomplete.
#[derive(Debug, Clone)]
pub struct VocabularyIndex {
    entries: Vec<VocabEntry>,
}

#[derive(Debug, Clone)]
struct VocabEntry {
    /// Lowercase text for matching.
    search_text: String,
    suggestion: Suggestion,
}

impl VocabularyIndex {
    /// Build a vocabulary index from a discovered schema.
    pub fn build(schema: &GraphSchema) -> Self {
        let mut entries = Vec::new();

        // Index labels
        for label in &schema.labels {
            let count_detail = schema
                .label_properties
                .get(label)
                .map(|_| "label".to_string());

            entries.push(VocabEntry {
                search_text: label.to_lowercase(),
                suggestion: Suggestion {
                    suggestion_type: SuggestionType::Label,
                    value: label.clone(),
                    detail: count_detail,
                    cypher: build_cypher_for_label(label),
                },
            });
        }

        // Index relationship types
        for rel_type in &schema.relationship_types {
            entries.push(VocabEntry {
                search_text: rel_type.to_lowercase(),
                suggestion: Suggestion {
                    suggestion_type: SuggestionType::RelationshipType,
                    value: rel_type.clone(),
                    detail: Some("relationship".to_string()),
                    cypher: build_cypher_for_relationship(rel_type),
                },
            });
        }

        // Index property keys
        for key in &schema.property_keys {
            entries.push(VocabEntry {
                search_text: key.to_lowercase(),
                suggestion: Suggestion {
                    suggestion_type: SuggestionType::PropertyKey,
                    value: key.clone(),
                    detail: Some("property".to_string()),
                    cypher: build_cypher_for_property_key(key),
                },
            });
        }

        // Index sample property values (for autocomplete)
        for (label, props) in &schema.label_properties {
            for prop in props {
                if prop.is_vector {
                    continue; // Skip vector embeddings
                }
                for sample in &prop.sample_values {
                    entries.push(VocabEntry {
                        search_text: sample.to_lowercase(),
                        suggestion: Suggestion {
                            suggestion_type: SuggestionType::PropertyValue,
                            value: sample.clone(),
                            detail: Some(format!("{}.{}", label, prop.key)),
                            cypher: build_cypher_for_property_value(label, &prop.key, sample),
                        },
                    });
                }
            }
        }

        debug!(
            "Built vocabulary index with {} entries",
            entries.len()
        );

        VocabularyIndex { entries }
    }

    /// Return suggestions matching the input prefix.
    pub fn autocomplete(&self, input: &str) -> Vec<Suggestion> {
        if input.is_empty() {
            return Vec::new();
        }

        let query = input.to_lowercase();
        let mut results: Vec<(usize, &Suggestion)> = self
            .entries
            .iter()
            .filter_map(|entry| {
                if entry.search_text.starts_with(&query) {
                    // Exact prefix match scored higher
                    Some((0, &entry.suggestion))
                } else if entry.search_text.contains(&query) {
                    // Substring match
                    Some((1, &entry.suggestion))
                } else {
                    None
                }
            })
            .collect();

        // Sort: prefix matches first, then by type priority (labels > rels > props)
        results.sort_by(|a, b| {
            a.0.cmp(&b.0).then_with(|| {
                type_priority(&a.1.suggestion_type).cmp(&type_priority(&b.1.suggestion_type))
            })
        });

        // Deduplicate by value
        let mut seen = std::collections::HashSet::new();
        results
            .into_iter()
            .filter(|(_, s)| seen.insert(s.value.clone()))
            .map(|(_, s)| s.clone())
            .take(20) // Cap at 20 suggestions
            .collect()
    }
}

fn type_priority(t: &SuggestionType) -> u8 {
    match t {
        SuggestionType::Label => 0,
        SuggestionType::RelationshipType => 1,
        SuggestionType::PropertyValue => 2,
        SuggestionType::PropertyKey => 3,
    }
}

/// Build a Cypher query for selecting all nodes of a label.
fn build_cypher_for_label(label: &str) -> String {
    format!("MATCH (n:`{}`) RETURN n LIMIT 100", label)
}

/// Build a Cypher query for selecting edges of a relationship type.
fn build_cypher_for_relationship(rel_type: &str) -> String {
    format!(
        "MATCH (a)-[r:`{}`]->(b) RETURN a, r, b LIMIT 100",
        rel_type
    )
}

/// Build a Cypher query for nodes with a specific property key.
fn build_cypher_for_property_key(key: &str) -> String {
    format!(
        "MATCH (n) WHERE n.`{}` IS NOT NULL RETURN n LIMIT 100",
        key
    )
}

/// Build a Cypher query for nodes matching a property value.
fn build_cypher_for_property_value(label: &str, key: &str, value: &str) -> String {
    let sanitized = sanitize_fulltext(value);
    format!(
        "MATCH (n:`{}`) WHERE n.`{}` CONTAINS '{}' RETURN n LIMIT 100",
        label,
        key,
        sanitized.replace('\'', "\\'")
    )
}

/// Sanitize input for RediSearch fulltext queries.
/// Escapes reserved words that would crash FalkorDB's RediSearch engine.
pub fn sanitize_fulltext(input: &str) -> String {
    let mut result = input.to_string();
    let upper = input.to_uppercase();
    for reserved in REDISEARCH_RESERVED {
        if upper.contains(reserved) {
            // Case-insensitive replacement: escape first occurrence per reserved word
            if let Some(pos) = upper.find(reserved) {
                let orig = &result[pos..pos + reserved.len()];
                let escaped = format!("\\{}", orig);
                result = format!(
                    "{}{}{}",
                    &result[..pos],
                    escaped,
                    &result[pos + reserved.len()..]
                );
            }
        }
    }
    result
}

/// Validate that a Cypher query is read-only. Delegates to connection module.
pub fn validate_read_only(cypher: &str) -> Result<(), String> {
    crate::connection::validate_read_only(cypher).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_fulltext_reserved_words() {
        assert_eq!(sanitize_fulltext("MAP"), "\\MAP");
        assert_eq!(sanitize_fulltext("hello"), "hello");
        assert_eq!(sanitize_fulltext("FILTER data"), "\\FILTER data");
    }

    #[test]
    fn test_autocomplete_empty_input() {
        let schema = GraphSchema {
            graph_name: "test".to_string(),
            labels: vec!["Entity".to_string()],
            relationship_types: vec![],
            property_keys: vec![],
            label_properties: Default::default(),
            node_count: 0,
            edge_count: 0,
            discovered_at: chrono::Utc::now(),
        };
        let idx = VocabularyIndex::build(&schema);
        assert!(idx.autocomplete("").is_empty());
    }

    #[test]
    fn test_autocomplete_label_match() {
        let schema = GraphSchema {
            graph_name: "test".to_string(),
            labels: vec!["Entity".to_string(), "Episodic".to_string()],
            relationship_types: vec!["RELATES_TO".to_string()],
            property_keys: vec!["name".to_string()],
            label_properties: Default::default(),
            node_count: 0,
            edge_count: 0,
            discovered_at: chrono::Utc::now(),
        };
        let idx = VocabularyIndex::build(&schema);

        let results = idx.autocomplete("ent");
        assert!(!results.is_empty());
        assert_eq!(results[0].value, "Entity");
        assert_eq!(results[0].suggestion_type, SuggestionType::Label);
    }

    #[test]
    fn test_build_cypher_for_label() {
        let cypher = build_cypher_for_label("Entity");
        assert!(cypher.contains("MATCH (n:`Entity`)"));
        assert!(cypher.contains("RETURN n"));
    }
}
