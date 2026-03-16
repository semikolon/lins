use crate::schema::GraphSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;
use tracing::debug;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Failed to read config file: {0}")]
    Io(#[from] std::io::Error),

    #[error("Failed to parse config: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Failed to serialize config: {0}")]
    Serialize(#[from] toml::ser::Error),
}

/// Top-level Lins configuration, parsed from lins.toml.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinsConfig {
    #[serde(default = "default_connection")]
    pub connection: ConnectionConfig,

    #[serde(default = "default_server")]
    pub server: ServerConfig,

    #[serde(default)]
    pub styling: StylingConfig,

    #[serde(default)]
    pub display: DisplayConfig,

    #[serde(default)]
    pub search: SearchConfig,

    /// Plugin config overlay file paths, loaded in order.
    #[serde(default)]
    pub plugins: Vec<String>,
}

/// FalkorDB connection settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    #[serde(default = "default_host")]
    pub host: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default)]
    pub password: Option<String>,

    #[serde(default)]
    pub default_graph: Option<String>,
}

/// Web server settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_server_port")]
    pub port: u16,

    #[serde(default = "default_server_host")]
    pub host: String,
}

/// Styling configuration with per-label rules.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StylingConfig {
    #[serde(default)]
    pub defaults: StylingDefaults,

    #[serde(default)]
    pub labels: HashMap<String, LabelStyle>,

    #[serde(default)]
    pub relationships: HashMap<String, RelationshipStyle>,
}

/// Default styling values applied to all nodes/edges.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylingDefaults {
    #[serde(default = "default_font")]
    pub font_family: String,

    #[serde(default = "default_background")]
    pub background: String,

    #[serde(default = "default_edge_color")]
    pub edge_color: String,
}

/// Styling for a specific node label.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabelStyle {
    #[serde(default)]
    pub color: Option<String>,

    #[serde(default)]
    pub size: Option<SizeConfig>,

    #[serde(default)]
    pub caption: Option<String>,

    #[serde(default)]
    pub shape: Option<String>,

    #[serde(default)]
    pub tooltip: Option<Vec<String>>,
}

/// Node size can be a fixed number or derived from a property.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SizeConfig {
    Fixed(f64),
    Property(String),
}

/// Styling for a specific relationship type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipStyle {
    #[serde(default)]
    pub color: Option<String>,

    #[serde(default)]
    pub label: Option<String>,

    #[serde(default)]
    pub style: Option<String>,
}

/// Display preferences for property panels.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DisplayConfig {
    #[serde(default)]
    pub properties: PropertyDisplayConfig,
}

/// Which properties to show/hide.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PropertyDisplayConfig {
    #[serde(default)]
    pub show: Vec<String>,

    #[serde(default)]
    pub hide: Vec<String>,

    #[serde(default)]
    pub date_format: Option<String>,
}

/// Search configuration (LLM and embedding endpoints).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchConfig {
    #[serde(default)]
    pub llm_endpoint: Option<String>,

    #[serde(default)]
    pub llm_model: Option<String>,

    #[serde(default)]
    pub llm_api_key_env: Option<String>,

    #[serde(default)]
    pub embedding_endpoint: Option<String>,

    #[serde(default)]
    pub embedding_dimensions: Option<usize>,
}

// Default value functions

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    6380
}

fn default_server_port() -> u16 {
    3000
}

fn default_server_host() -> String {
    "0.0.0.0".to_string()
}

fn default_font() -> String {
    "Manrope".to_string()
}

fn default_background() -> String {
    "#1a1a2e".to_string()
}

fn default_edge_color() -> String {
    "#4a5568".to_string()
}

fn default_connection() -> ConnectionConfig {
    ConnectionConfig {
        host: default_host(),
        port: default_port(),
        password: None,
        default_graph: None,
    }
}

fn default_server() -> ServerConfig {
    ServerConfig {
        port: default_server_port(),
        host: default_server_host(),
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        default_connection()
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        default_server()
    }
}

impl Default for StylingDefaults {
    fn default() -> Self {
        StylingDefaults {
            font_family: default_font(),
            background: default_background(),
            edge_color: default_edge_color(),
        }
    }
}

impl Default for LinsConfig {
    fn default() -> Self {
        LinsConfig {
            connection: ConnectionConfig::default(),
            server: ServerConfig::default(),
            styling: StylingConfig::default(),
            display: DisplayConfig::default(),
            search: SearchConfig::default(),
            plugins: Vec::new(),
        }
    }
}

/// Load configuration from a TOML file.
pub fn load_config(path: &Path) -> Result<LinsConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let config: LinsConfig = toml::from_str(&content)?;
    debug!("Loaded config from {}", path.display());
    Ok(config)
}

/// A set of distinct colors for auto-assigning to labels.
const LABEL_PALETTE: &[&str] = &[
    "#2d6a4f", // forest green
    "#e07a5f", // terra cotta
    "#7c3aed", // purple
    "#0891b2", // cyan
    "#d97706", // amber
    "#dc2626", // red
    "#059669", // emerald
    "#7c3aed", // violet
    "#db2777", // pink
    "#4f46e5", // indigo
    "#65a30d", // lime
    "#0d9488", // teal
];

/// Generate a default config from a discovered schema.
/// Assigns distinct colors to each label and hides known vector/internal properties.
pub fn generate_default_config(schema: &GraphSchema) -> LinsConfig {
    let mut labels = HashMap::new();

    for (i, label) in schema.labels.iter().enumerate() {
        let color = LABEL_PALETTE[i % LABEL_PALETTE.len()].to_string();

        // Try to pick a good caption property
        let caption = schema
            .label_properties
            .get(label)
            .and_then(|props| {
                // Prefer "name", then "title", then "summary", then first string property
                for preferred in &["name", "title", "summary", "label"] {
                    if props.iter().any(|p| p.key == *preferred) {
                        return Some(preferred.to_string());
                    }
                }
                props
                    .iter()
                    .find(|p| p.value_type == "String" && !p.is_vector)
                    .map(|p| p.key.clone())
            });

        labels.insert(
            label.clone(),
            LabelStyle {
                color: Some(color),
                size: None,
                caption,
                shape: None,
                tooltip: None,
            },
        );
    }

    // Detect properties to hide (vectors, embeddings, internal IDs)
    let mut hide_props = Vec::new();
    for props_list in schema.label_properties.values() {
        for prop in props_list {
            if prop.is_vector
                || prop.key.ends_with("_embedding")
                || prop.key == "group_id"
                || prop.key == "uuid"
            {
                if !hide_props.contains(&prop.key) {
                    hide_props.push(prop.key.clone());
                }
            }
        }
    }

    LinsConfig {
        connection: ConnectionConfig {
            host: default_host(),
            port: default_port(),
            password: Some("falkordb".to_string()),
            default_graph: Some(schema.graph_name.clone()),
        },
        server: ServerConfig::default(),
        styling: StylingConfig {
            defaults: StylingDefaults::default(),
            labels,
            relationships: HashMap::new(),
        },
        display: DisplayConfig {
            properties: PropertyDisplayConfig {
                show: Vec::new(),
                hide: hide_props,
                date_format: Some("%Y-%m-%d %H:%M".to_string()),
            },
        },
        search: SearchConfig::default(),
        plugins: Vec::new(),
    }
}

/// Serialize a LinsConfig to a TOML string suitable for writing to a file.
pub fn serialize_config(config: &LinsConfig) -> Result<String, ConfigError> {
    Ok(toml::to_string_pretty(config)?)
}
