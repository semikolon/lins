use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use lins_core::connection::validate_read_only;
use lins_core::schema::{discover_schema, GraphSchema};
use lins_core::view;
use lins_core::{GraphView, VocabularyIndex};
use serde::{Deserialize, Serialize};
use tracing::info;

use crate::AppState;

type ApiError = (StatusCode, String);

fn internal(e: impl std::fmt::Display) -> ApiError {
    (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
}

/// A graph with at most this many visible nodes is loaded whole.
const DEFAULT_MAX_NODES: usize = 1500;
/// A larger graph opens on this many of its best-connected nodes.
const DEFAULT_OVERVIEW: usize = 150;
/// One widening brings in at most this many neighbours.
const DEFAULT_NEIGHBOURS: usize = 60;
/// The most pairs read to rank nodes by how connected they are.
const PAIR_CAP: usize = 400_000;
/// The property names tried as a node's name when its label has no caption.
const NAME_KEYS: &[&str] = &["name", "title", "short_title", "summary", "label", "text"];

/// Build the graph API router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/config", get(get_config))
        .route("/graphs", get(list_graphs))
        .route("/graphs/{name}/schema", get(get_schema))
        .route("/graphs/{name}/view", get(get_view))
        .route("/graphs/{name}/data", get(get_graph_data))
        .route("/graphs/{name}/around/{id}", get(get_around))
        .route("/graphs/{name}/find", get(find_nodes))
        .route("/graphs/{name}/query", post(execute_query))
}

/// What the page needs to know before it has chosen a graph.
#[derive(Serialize)]
struct ServerConfigInfo {
    default_graph: Option<String>,
}

/// GET /api/config — the graph to open first, if the config names one.
async fn get_config(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(ServerConfigInfo { default_graph: config.connection.default_graph.clone() })
}

/// GET /api/graphs/:name/view — how this graph is to be shown: names, colours,
/// and what is left out. The base config with its overlays laid on top.
async fn get_view(State(state): State<AppState>, Path(name): Path<String>) -> impl IntoResponse {
    Json(view_of(&state, &name).await)
}

async fn view_of(state: &AppState, name: &str) -> GraphView {
    state.config.read().await.view_for(name)
}

/// `?all=true` asks for everything, the hidden kinds included.
#[derive(Deserialize, Default)]
struct ViewParams {
    #[serde(default)]
    all: bool,
    #[serde(default)]
    limit: Option<usize>,
}

/// The two conditions of a view, for nodes bound to any alias and for links.
struct Conditions {
    rules: Vec<lins_core::NodeHideRule>,
    hidden_rels: Vec<String>,
}

impl Conditions {
    fn of(view: &GraphView, all: bool) -> Self {
        if all {
            Conditions { rules: Vec::new(), hidden_rels: Vec::new() }
        } else {
            Conditions {
                rules: view.display.nodes.hide.clone(),
                hidden_rels: view.display.relationships.hide.clone(),
            }
        }
    }
    fn node(&self, alias: &str) -> String {
        view::node_visible(alias, &self.rules)
    }
    fn rel(&self, alias: &str) -> String {
        view::rel_visible(alias, &self.hidden_rels)
    }
}

/// Summary info about a graph, returned by the list endpoint.
#[derive(Serialize)]
struct GraphInfo {
    name: String,
    node_count: usize,
    edge_count: usize,
}

/// GET /api/graphs — list available FalkorDB graphs with node/edge counts.
async fn list_graphs(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let graphs = state
        .conn
        .list_graphs()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let mut result = Vec::new();
    for name in graphs {
        let node_count = state
            .conn
            .query_count(&name, "MATCH (n) RETURN count(n)")
            .await
            .unwrap_or(0);
        let edge_count = state
            .conn
            .query_count(&name, "MATCH ()-[r]->() RETURN count(r)")
            .await
            .unwrap_or(0);

        result.push(GraphInfo {
            name,
            node_count,
            edge_count,
        });
    }

    Ok(Json(result))
}

/// GET /api/graphs/:name/schema — return discovered schema as JSON.
/// Discovers on demand if not yet cached.
async fn get_schema(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ApiError> {
    Ok(Json(ensure_schema(&state, &name).await?))
}

/// The schema of a graph, discovered once and kept, with its vocabulary index.
async fn ensure_schema(state: &AppState, name: &str) -> Result<GraphSchema, ApiError> {
    {
        let schemas = state.schemas.read().await;
        if let Some(schema) = schemas.get(name) {
            return Ok(schema.clone());
        }
    }

    info!("Discovering schema for graph '{}'", name);
    let schema = discover_schema(&state.conn, name).await.map_err(internal)?;

    let vocab = VocabularyIndex::build(&schema);
    {
        let mut schemas = state.schemas.write().await;
        schemas.insert(name.to_string(), schema.clone());
    }
    {
        let mut indexes = state.vocab_indexes.write().await;
        indexes.insert(name.to_string(), vocab);
    }
    Ok(schema)
}

/// What a view returns beside its nodes and links.
#[derive(Serialize)]
struct ViewResponse {
    #[serde(flatten)]
    result: lins_core::QueryResult,
    /// Every node in the graph.
    total_node_count: usize,
    /// How many of them the view's own rules leave out.
    hidden_node_count: usize,
    /// "all": every visible node is here. "overview": the best-connected ones.
    /// "first": the nodes the view names. "around": one node and its neighbours.
    mode: &'static str,
    /// True when visible nodes exist that this response does not hold.
    truncated: bool,
    /// For "around": the node at the centre and how many neighbours it has in all.
    #[serde(skip_serializing_if = "Option::is_none")]
    center: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    neighbour_count: Option<usize>,
}

/// GET /api/graphs/:name/data — what a person should see first.
///
/// A view that names the nodes it opens on gets those and the links among them.
/// Otherwise a graph small enough is returned whole, without the kinds its view
/// hides, and a larger one opens on its best-connected visible nodes and the
/// links among them, since the first thousand nodes in storage order are mostly
/// unjoined. A caller that asks for everything is not held to the named nodes.
async fn get_graph_data(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<ViewParams>,
) -> Result<impl IntoResponse, ApiError> {
    let graph_view = view_of(&state, &name).await;
    let cond = Conditions::of(&graph_view, params.all);
    let conn = &state.conn;

    let total = conn.query_count(&name, "MATCH (n) RETURN count(n)").await.map_err(internal)?;
    let visible = conn
        .query_count(&name, &format!("MATCH (n) WHERE {} RETURN count(n)", cond.node("n")))
        .await
        .map_err(internal)?;
    let max_nodes = graph_view.display.nodes.max_nodes.unwrap_or(DEFAULT_MAX_NODES);
    let link_condition = format!("{} AND {} AND {}", cond.node("a"), cond.node("b"), cond.rel("r"));

    let named = if params.all { None } else { view::node_named("n", &graph_view.display.nodes.first) };
    let (mut result, mode, truncated) = if let Some(named) = named {
        let ends = format!(
            "{} AND {}",
            view::node_named("a", &graph_view.display.nodes.first).unwrap_or_default(),
            view::node_named("b", &graph_view.display.nodes.first).unwrap_or_default()
        );
        let nodes = conn
            .query_built(&name, &format!("MATCH (n) WHERE {} AND {named} RETURN n LIMIT {max_nodes}", cond.node("n")))
            .await
            .map_err(internal)?;
        let edges = conn
            .query_built(&name, &format!("MATCH (a)-[r]->(b) WHERE {link_condition} AND {ends} RETURN r"))
            .await
            .map_err(internal)?;
        let truncated = nodes.nodes.len() < visible;
        let result = lins_core::QueryResult {
            edges: view::links_among(edges.edges, &nodes.nodes),
            nodes: nodes.nodes,
            query_time_ms: nodes.query_time_ms + edges.query_time_ms,
            source_graph: name.clone(),
        };
        (result, "first", truncated)
    } else if visible <= max_nodes {
        let nodes = conn
            .query_built(&name, &format!("MATCH (n) WHERE {} RETURN n", cond.node("n")))
            .await
            .map_err(internal)?;
        let edges = conn
            .query_built(&name, &format!("MATCH (a)-[r]->(b) WHERE {link_condition} RETURN r"))
            .await
            .map_err(internal)?;
        let result = lins_core::QueryResult {
            edges: view::links_among(edges.edges, &nodes.nodes),
            nodes: nodes.nodes,
            query_time_ms: nodes.query_time_ms + edges.query_time_ms,
            source_graph: name.clone(),
        };
        (result, "all", false)
    } else {
        let overview = params
            .limit
            .or(graph_view.display.nodes.overview)
            .unwrap_or(DEFAULT_OVERVIEW)
            .min(max_nodes);
        let pairs = conn
            .query_int_rows(
                &name,
                &format!("MATCH (a)-[r]->(b) WHERE {link_condition} RETURN id(a), id(b) LIMIT {PAIR_CAP}"),
            )
            .await
            .map_err(internal)?;
        let ids = view::best_connected(&pairs, overview);
        let mut set = load_set(&state, &name, &ids, &cond).await?;
        view::drop_unjoined(&mut set);
        (set, "overview", true)
    };

    view::strip(&mut result, &graph_view);
    Ok(Json(ViewResponse {
        result,
        total_node_count: total,
        hidden_node_count: total.saturating_sub(visible),
        mode,
        truncated,
        center: None,
        neighbour_count: None,
    }))
}

/// The named nodes and every visible link among them.
async fn load_set(
    state: &AppState,
    name: &str,
    ids: &[i64],
    cond: &Conditions,
) -> Result<lins_core::QueryResult, ApiError> {
    if ids.is_empty() {
        return Ok(lins_core::QueryResult {
            nodes: Vec::new(),
            edges: Vec::new(),
            query_time_ms: 0.0,
            source_graph: name.to_string(),
        });
    }
    let list = view::cypher_ids(ids);
    let nodes = state
        .conn
        .query_built(name, &format!("MATCH (n) WHERE id(n) IN {list} RETURN n"))
        .await
        .map_err(internal)?;
    let edges = state
        .conn
        .query_built(
            name,
            &format!(
                "MATCH (a)-[r]->(b) WHERE id(a) IN {list} AND id(b) IN {list} AND {} RETURN r",
                cond.rel("r")
            ),
        )
        .await
        .map_err(internal)?;
    Ok(lins_core::QueryResult {
        edges: view::links_among(edges.edges, &nodes.nodes),
        nodes: nodes.nodes,
        query_time_ms: nodes.query_time_ms + edges.query_time_ms,
        source_graph: name.to_string(),
    })
}

/// GET /api/graphs/:name/around/:id — one node, its visible neighbours, and
/// the links among them all. This is what a click on a node widens the view by.
async fn get_around(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, i64)>,
    Query(params): Query<ViewParams>,
) -> Result<impl IntoResponse, ApiError> {
    let graph_view = view_of(&state, &name).await;
    let cond = Conditions::of(&graph_view, params.all);
    let limit = params.limit.or(graph_view.display.nodes.neighbours).unwrap_or(DEFAULT_NEIGHBOURS);
    let neighbour_condition = format!("id(n) = {id} AND {} AND {}", cond.node("m"), cond.rel("r"));

    let neighbour_count = state
        .conn
        .query_count(
            &name,
            &format!("MATCH (n)-[r]-(m) WHERE {neighbour_condition} RETURN count(DISTINCT m)"),
        )
        .await
        .map_err(internal)?;
    let rows = state
        .conn
        .query_int_rows(
            &name,
            &format!("MATCH (n)-[r]-(m) WHERE {neighbour_condition} RETURN DISTINCT id(m) LIMIT {limit}"),
        )
        .await
        .map_err(internal)?;
    let mut ids: Vec<i64> = vec![id];
    ids.extend(rows.into_iter().filter_map(|row| row.first().copied()).filter(|m| *m != id));

    let mut result = load_set(&state, &name, &ids, &cond).await?;
    if !result.nodes.iter().any(|n| n.id == id) {
        return Err((StatusCode::NOT_FOUND, format!("No node {id} in graph '{name}'")));
    }
    view::strip(&mut result, &graph_view);

    let total = state.conn.query_count(&name, "MATCH (n) RETURN count(n)").await.map_err(internal)?;
    Ok(Json(ViewResponse {
        truncated: neighbour_count > ids.len() - 1,
        result,
        total_node_count: total,
        hidden_node_count: 0,
        mode: "around",
        center: Some(id),
        neighbour_count: Some(neighbour_count),
    }))
}

#[derive(Deserialize)]
struct FindParams {
    q: String,
    #[serde(default)]
    all: bool,
    #[serde(default)]
    limit: Option<usize>,
}

/// The properties a search looks in for one label: its caption keys, or the
/// usual name keys the label really has.
fn search_keys(graph_view: &GraphView, schema: &GraphSchema, label: &str) -> Vec<String> {
    let has = |key: &str| {
        schema
            .label_properties
            .get(label)
            .map(|props| props.iter().any(|p| p.key == key && !p.is_vector))
            .unwrap_or(false)
    };
    let configured: Vec<String> = graph_view
        .styling
        .labels
        .get(label)
        .and_then(|style| style.caption.as_ref())
        .map(|caption| caption.keys().into_iter().map(str::to_string).collect())
        .unwrap_or_default();
    let keys: Vec<String> = if configured.is_empty() {
        NAME_KEYS.iter().filter(|k| has(k)).map(|k| k.to_string()).collect()
    } else {
        configured
    };
    keys.into_iter().filter(|k| view::is_ident(k)).collect()
}

/// The properties that identify a node of this label, for an exact match:
/// `id`, `uuid`, and anything ending in `_id`.
fn id_keys(schema: &GraphSchema, label: &str) -> Vec<String> {
    schema
        .label_properties
        .get(label)
        .map(|props| {
            props
                .iter()
                .map(|p| p.key.clone())
                .filter(|k| (k == "id" || k == "uuid" || k.ends_with("_id")) && view::is_ident(k))
                .collect()
        })
        .unwrap_or_default()
}

/// GET /api/graphs/:name/find?q=words — nodes whose name holds the words,
/// or whose id is exactly them. The way into a large graph.
async fn find_nodes(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Query(params): Query<FindParams>,
) -> Result<impl IntoResponse, ApiError> {
    let words = params.q.trim();
    if words.is_empty() {
        return Err((StatusCode::BAD_REQUEST, "q is empty".to_string()));
    }
    let graph_view = view_of(&state, &name).await;
    let cond = Conditions::of(&graph_view, params.all);
    let schema = ensure_schema(&state, &name).await?;
    let limit = params.limit.unwrap_or(25).min(200);
    let exact = view::cypher_str(words);
    // Every word has to be in the name somewhere, in any order: people remember
    // the words of a claim, seldom their sequence.
    let needles: Vec<String> = words
        .split_whitespace()
        .take(8)
        .map(|word| view::cypher_str(&word.to_lowercase()))
        .collect();

    let mut found = lins_core::QueryResult {
        nodes: Vec::new(),
        edges: Vec::new(),
        query_time_ms: 0.0,
        source_graph: name.clone(),
    };
    for label in &schema.labels {
        if found.nodes.len() >= limit || !view::is_ident(label) {
            continue;
        }
        let keys = search_keys(&graph_view, &schema, label);
        let mut tests: Vec<String> = Vec::new();
        if !keys.is_empty() {
            let per_word: Vec<String> = needles
                .iter()
                .map(|needle| {
                    let any_key: Vec<String> = keys
                        .iter()
                        .map(|key| format!("toLower(toString(n.{key})) CONTAINS {needle}"))
                        .collect();
                    format!("({})", any_key.join(" OR "))
                })
                .collect();
            tests.push(format!("({})", per_word.join(" AND ")));
        }
        if !words.contains(char::is_whitespace) {
            tests.extend(id_keys(&schema, label).iter().map(|key| format!("toString(n.{key}) = {exact}")));
        }
        if tests.is_empty() {
            continue;
        }
        let cypher = format!(
            "MATCH (n:`{label}`) WHERE {} AND ({}) RETURN n LIMIT {}",
            cond.node("n"),
            tests.join(" OR "),
            limit - found.nodes.len()
        );
        let part = state.conn.query_built(&name, &cypher).await.map_err(internal)?;
        found.query_time_ms += part.query_time_ms;
        for node in part.nodes {
            if !found.nodes.iter().any(|n| n.id == node.id) {
                found.nodes.push(node);
            }
        }
    }
    view::strip(&mut found, &graph_view);
    Ok(Json(found))
}

/// Request body for the query endpoint.
#[derive(Deserialize)]
struct QueryRequest {
    cypher: String,
}

/// POST /api/graphs/:name/query — execute a read-only Cypher query.
async fn execute_query(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(body): Json<QueryRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Validate read-only
    validate_read_only(&body.cypher).map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

    let mut result = state
        .conn
        .query(&name, &body.cypher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    view::strip(&mut result, &view_of(&state, &name).await);

    Ok(Json(result))
}
