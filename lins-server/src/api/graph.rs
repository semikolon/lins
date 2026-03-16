use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use lins_core::connection::validate_read_only;
use lins_core::schema::discover_schema;
use lins_core::VocabularyIndex;
use serde::{Deserialize, Serialize};
use tracing::{info, warn};

use crate::AppState;

/// Build the graph API router.
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/graphs", get(list_graphs))
        .route("/graphs/{name}/schema", get(get_schema))
        .route("/graphs/{name}/data", get(get_graph_data))
        .route("/graphs/{name}/query", post(execute_query))
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
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Check cache first
    {
        let schemas = state.schemas.read().await;
        if let Some(schema) = schemas.get(&name) {
            return Ok(Json(schema.clone()));
        }
    }

    // Discover schema on demand
    info!("Discovering schema for graph '{}'", name);
    let schema = discover_schema(&state.conn, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Cache the schema and build vocabulary index
    let vocab = VocabularyIndex::build(&schema);
    {
        let mut schemas = state.schemas.write().await;
        schemas.insert(name.clone(), schema.clone());
    }
    {
        let mut indexes = state.vocab_indexes.write().await;
        indexes.insert(name.clone(), vocab);
    }

    Ok(Json(schema))
}

/// GET /api/graphs/:name/data — return all nodes and edges.
/// Counts first and warns if the graph is large.
async fn get_graph_data(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    // Count nodes first to decide loading strategy
    let node_count = state
        .conn
        .query_count(&name, "MATCH (n) RETURN count(n)")
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if node_count > 5000 {
        warn!(
            "Graph '{}' has {} nodes - loading may be slow",
            name, node_count
        );
    }

    // Load nodes and edges with separate queries for reliable parsing.
    let node_limit = if node_count > 1000 { 1000 } else { node_count.max(1000) };

    let node_cypher = format!("MATCH (n) RETURN n LIMIT {}", node_limit);
    let edge_cypher = "MATCH ()-[r]->() RETURN r LIMIT 5000";

    let node_result = state
        .conn
        .query(&name, &node_cypher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let edge_result = state
        .conn
        .query(&name, edge_cypher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let result = lins_core::QueryResult {
        nodes: node_result.nodes,
        edges: edge_result.edges,
        query_time_ms: node_result.query_time_ms + edge_result.query_time_ms,
        source_graph: name.clone(),
    };

    #[derive(Serialize)]
    struct DataResponse {
        #[serde(flatten)]
        result: lins_core::QueryResult,
        total_node_count: usize,
        truncated: bool,
    }

    Ok(Json(DataResponse {
        truncated: node_count > 1000,
        total_node_count: node_count,
        result,
    }))
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

    let result = state
        .conn
        .query(&name, &body.cypher)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(result))
}
