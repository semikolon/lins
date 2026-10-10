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

    /// Plugin config overlay file paths, loaded in order. A relative path is
    /// resolved against the folder the config file is in.
    #[serde(default)]
    pub plugins: Vec<String>,

    /// The overlays read from `plugins`. Filled by `load_config`, never written.
    #[serde(skip)]
    pub overlays: Vec<Overlay>,
}

/// A plugin overlay: styling and display rules laid over the base config,
/// for every graph or for the graphs it names.
///
/// `graphs` entries match a graph name exactly, or by prefix when they end
/// in `*` (`"notes_blob_*"`). An overlay with no `graphs` applies to all.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Overlay {
    #[serde(default)]
    pub graphs: Vec<String>,

    #[serde(default)]
    pub styling: StylingConfig,

    #[serde(default)]
    pub display: DisplayConfig,
}

impl Overlay {
    /// Does this overlay apply to the named graph?
    pub fn applies_to(&self, graph: &str) -> bool {
        self.graphs.is_empty()
            || self.graphs.iter().any(|pattern| match pattern.strip_suffix('*') {
                Some(prefix) => graph.starts_with(prefix),
                None => pattern == graph,
            })
    }
}

/// What one graph looks like: the base config with every overlay that
/// applies to it laid on top. This is what the frontend is given.
#[derive(Debug, Clone, Serialize)]
pub struct GraphView {
    pub graph: String,
    pub styling: StylingConfig,
    pub display: DisplayConfig,
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

    /// The property that names a node: one key, or several tried in order
    /// (`caption = ["short_title", "text"]`).
    #[serde(default)]
    pub caption: Option<Caption>,

    #[serde(default)]
    pub shape: Option<String>,

    #[serde(default)]
    pub tooltip: Option<Vec<String>>,
}

/// The property, or properties tried in order, that give a node its name.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Caption {
    One(String),
    Any(Vec<String>),
}

impl Caption {
    /// The keys to try, in order.
    pub fn keys(&self) -> Vec<&str> {
        match self {
            Caption::One(key) => vec![key.as_str()],
            Caption::Any(keys) => keys.iter().map(String::as_str).collect(),
        }
    }
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

    #[serde(default)]
    pub nodes: NodeDisplayConfig,

    #[serde(default)]
    pub relationships: RelationshipDisplayConfig,
}

/// Which nodes a view leaves out, and how many it loads at once.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeDisplayConfig {
    /// Nodes left out of every view unless the caller asks for everything.
    #[serde(default)]
    pub hide: Vec<NodeHideRule>,

    /// A graph with at most this many visible nodes is loaded whole (default 1500).
    #[serde(default)]
    pub max_nodes: Option<usize>,

    /// For a larger graph, how many of its best-connected nodes open the view (default 150).
    #[serde(default)]
    pub overview: Option<usize>,

    /// How many neighbours one widening brings in (default 60).
    #[serde(default)]
    pub neighbours: Option<usize>,
}

/// Hide the nodes with this label, or only those whose `key` holds one of `values`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NodeHideRule {
    pub label: String,

    #[serde(default)]
    pub key: Option<String>,

    #[serde(default)]
    pub values: Vec<String>,
}

/// Which relationship types a view leaves out.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RelationshipDisplayConfig {
    #[serde(default)]
    pub hide: Vec<String>,
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
            overlays: Vec::new(),
        }
    }
}

/// Load configuration from a TOML file, with the overlays its `plugins` name
/// and with the connection settings the environment overrides.
pub fn load_config(path: &Path) -> Result<LinsConfig, ConfigError> {
    let content = std::fs::read_to_string(path)?;
    let mut config: LinsConfig = toml::from_str(&content)?;
    let base = path.parent().unwrap_or_else(|| Path::new("."));
    for plugin in &config.plugins {
        let plugin_path = if Path::new(plugin).is_absolute() {
            Path::new(plugin).to_path_buf()
        } else {
            base.join(plugin)
        };
        let overlay: Overlay = toml::from_str(&std::fs::read_to_string(&plugin_path)?)?;
        debug!("Loaded overlay from {}", plugin_path.display());
        config.overlays.push(overlay);
    }
    config.connection.apply_env(|name| std::env::var(name).ok());
    debug!("Loaded config from {}", path.display());
    Ok(config)
}

impl ConnectionConfig {
    /// Let the environment override where FalkorDB is, so that a host name or
    /// a password need not be written into a config file that is shared.
    /// `LINS_FALKORDB_HOST`, `_PORT` and `_PASSWORD` win over the conventional
    /// `FALKORDB_HOST`, `_PORT` and `_PASSWORD`, which win over the file.
    pub fn apply_env(&mut self, get: impl Fn(&str) -> Option<String>) {
        let pick = |suffix: &str| {
            get(&format!("LINS_FALKORDB_{suffix}"))
                .or_else(|| get(&format!("FALKORDB_{suffix}")))
                .filter(|value| !value.is_empty())
        };
        if let Some(host) = pick("HOST") {
            self.host = host;
        }
        if let Some(port) = pick("PORT").and_then(|value| value.parse().ok()) {
            self.port = port;
        }
        if let Some(password) = pick("PASSWORD") {
            self.password = Some(password);
        }
    }
}

impl LinsConfig {
    /// The view of one graph, in the three layers the config is designed in:
    /// the plugin overlays that apply to it, in the order they were listed,
    /// and then this config's own styling and display rules on top, since
    /// what the person running Lins wrote always wins.
    ///
    /// A later layer's label or relationship style wins field by field;
    /// the lists of hidden things are added together.
    pub fn view_for(&self, graph: &str) -> GraphView {
        let mut styling = StylingConfig { defaults: self.styling.defaults.clone(), ..Default::default() };
        let mut display = DisplayConfig::default();
        for overlay in self.overlays.iter().filter(|o| o.applies_to(graph)) {
            lay_over(&mut styling, &mut display, &overlay.styling, &overlay.display);
        }
        lay_over(&mut styling, &mut display, &self.styling, &self.display);
        GraphView { graph: graph.to_string(), styling, display }
    }
}

/// Lay one layer's styling and display rules over what is there.
fn lay_over(
    styling: &mut StylingConfig,
    display: &mut DisplayConfig,
    over_styling: &StylingConfig,
    over: &DisplayConfig,
) {
    for (label, over) in &over_styling.labels {
        let style = styling.labels.entry(label.clone()).or_insert_with(|| LabelStyle {
            color: None,
            size: None,
            caption: None,
            shape: None,
            tooltip: None,
        });
        if over.color.is_some() {
            style.color = over.color.clone();
        }
        if over.size.is_some() {
            style.size = over.size.clone();
        }
        if over.caption.is_some() {
            style.caption = over.caption.clone();
        }
        if over.shape.is_some() {
            style.shape = over.shape.clone();
        }
        if over.tooltip.is_some() {
            style.tooltip = over.tooltip.clone();
        }
    }
    for (rel, over) in &over_styling.relationships {
        let style = styling
            .relationships
            .entry(rel.clone())
            .or_insert_with(|| RelationshipStyle { color: None, label: None, style: None });
        if over.color.is_some() {
            style.color = over.color.clone();
        }
        if over.label.is_some() {
            style.label = over.label.clone();
        }
        if over.style.is_some() {
            style.style = over.style.clone();
        }
    }
    add_missing(&mut display.properties.show, &over.properties.show);
    add_missing(&mut display.properties.hide, &over.properties.hide);
    if over.properties.date_format.is_some() {
        display.properties.date_format = over.properties.date_format.clone();
    }
    for rule in &over.nodes.hide {
        if !display.nodes.hide.contains(rule) {
            display.nodes.hide.push(rule.clone());
        }
    }
    display.nodes.max_nodes = over.nodes.max_nodes.or(display.nodes.max_nodes);
    display.nodes.overview = over.nodes.overview.or(display.nodes.overview);
    display.nodes.neighbours = over.nodes.neighbours.or(display.nodes.neighbours);
    add_missing(&mut display.relationships.hide, &over.relationships.hide);
}

fn add_missing(into: &mut Vec<String>, from: &[String]) {
    for item in from {
        if !into.contains(item) {
            into.push(item.clone());
        }
    }
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
                caption: caption.map(Caption::One),
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
            nodes: NodeDisplayConfig::default(),
            relationships: RelationshipDisplayConfig::default(),
        },
        search: SearchConfig::default(),
        plugins: Vec::new(),
        overlays: Vec::new(),
    }
}

/// Serialize a LinsConfig to a TOML string suitable for writing to a file.
pub fn serialize_config(config: &LinsConfig) -> Result<String, ConfigError> {
    Ok(toml::to_string_pretty(config)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn overlay(toml_text: &str) -> Overlay {
        toml::from_str(toml_text).expect("the overlay parses")
    }

    #[test]
    fn an_overlay_applies_by_exact_name_by_prefix_or_to_all() {
        let named = overlay("graphs = [\"notes\", \"notes_blob_*\"]");
        assert!(named.applies_to("notes"));
        assert!(named.applies_to("notes_blob_alpha"));
        assert!(!named.applies_to("notes_test"));
        assert!(!named.applies_to("blob_alpha"));
        assert!(overlay("").applies_to("anything"));
    }

    #[test]
    fn a_view_is_the_overlay_for_its_graphs_with_the_users_own_config_on_top() {
        let mut config: LinsConfig = toml::from_str(
            r##"
            [styling.labels.Claim]
            color = "#111111"
            [styling.relationships.ATTACKS]
            color = "#bb0000"
            [display.properties]
            hide = ["uuid"]
            "##,
        )
        .unwrap();
        config.overlays.push(overlay(
            r##"
            graphs = ["g1"]
            [styling.labels.Claim]
            color = "#999999"
            caption = ["short_title", "text"]
            [styling.relationships.ATTACKS]
            color = "#aa0000"
            [styling.relationships.SUPPORTS]
            color = "#00aa00"
            [display.properties]
            hide = ["embedding", "uuid"]
            [[display.nodes.hide]]
            label = "Claim"
            key = "claim_kind"
            values = ["cq_positive", "cq_negative"]
            [display.relationships]
            hide = ["SIMILAR_TO"]
            [display.nodes]
            overview = 80
            "##,
        ));

        let view = config.view_for("g1");
        let claim = &view.styling.labels["Claim"];
        assert_eq!(claim.color.as_deref(), Some("#111111"), "what the user wrote wins over a plugin");
        assert_eq!(
            claim.caption.as_ref().unwrap().keys(),
            vec!["short_title", "text"],
            "a field the user left out comes from the plugin"
        );
        assert_eq!(view.styling.relationships["ATTACKS"].color.as_deref(), Some("#bb0000"));
        assert_eq!(view.styling.relationships["SUPPORTS"].color.as_deref(), Some("#00aa00"));
        assert_eq!(view.display.properties.hide, vec!["embedding", "uuid"], "added once");
        assert_eq!(view.display.nodes.hide.len(), 1);
        assert_eq!(view.display.nodes.overview, Some(80));
        assert_eq!(view.display.relationships.hide, vec!["SIMILAR_TO"]);

        let other = config.view_for("g2");
        assert_eq!(other.styling.labels["Claim"].color.as_deref(), Some("#111111"));
        assert!(other.styling.labels["Claim"].caption.is_none(), "the plugin is not for this graph");
        assert!(other.display.nodes.hide.is_empty());
        assert!(!other.styling.relationships.contains_key("SUPPORTS"));
        assert_eq!(other.display.properties.hide, vec!["uuid"]);
    }

    #[test]
    fn the_environment_overrides_the_connection_and_the_lins_names_win() {
        let mut conn = ConnectionConfig::default();
        conn.apply_env(|name| match name {
            "FALKORDB_HOST" => Some("general.example".into()),
            "LINS_FALKORDB_HOST" => Some("lins.example".into()),
            "FALKORDB_PORT" => Some("7000".into()),
            "FALKORDB_PASSWORD" => Some("".into()),
            _ => None,
        });
        assert_eq!(conn.host, "lins.example");
        assert_eq!(conn.port, 7000);
        assert_eq!(conn.password, None, "an empty value overrides nothing");

        let mut untouched = ConnectionConfig::default();
        untouched.apply_env(|_| None);
        assert_eq!(untouched.host, "127.0.0.1");
        assert_eq!(untouched.port, 6380);
    }

    #[test]
    fn a_caption_is_one_key_or_several() {
        let one: LabelStyle = toml::from_str("caption = \"name\"").unwrap();
        assert_eq!(one.caption.unwrap().keys(), vec!["name"]);
        let several: LabelStyle = toml::from_str("caption = [\"short_title\", \"text\"]").unwrap();
        assert_eq!(several.caption.unwrap().keys(), vec!["short_title", "text"]);
    }

    #[test]
    fn plugins_are_read_from_beside_the_config_file() {
        let dir = std::env::temp_dir().join(format!("lins-config-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("lins.toml"), "plugins = [\"view.toml\"]\n").unwrap();
        std::fs::write(
            dir.join("view.toml"),
            "graphs = [\"g\"]\n[display.relationships]\nhide = [\"SIMILAR_TO\"]\n",
        )
        .unwrap();
        let config = load_config(&dir.join("lins.toml")).unwrap();
        assert_eq!(config.overlays.len(), 1);
        assert_eq!(config.view_for("g").display.relationships.hide, vec!["SIMILAR_TO"]);
        assert!(config.view_for("other").display.relationships.hide.is_empty());
        let _ = std::fs::remove_file(dir.join("lins.toml"));
        let _ = std::fs::remove_file(dir.join("view.toml"));
    }
}
