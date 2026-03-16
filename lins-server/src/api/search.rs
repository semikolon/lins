use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::Deserialize;
use tracing::debug;

use crate::AppState;

/// Build the search API router.
pub fn router() -> Router<AppState> {
    Router::new().route("/search/vocabulary", post(vocabulary_search))
}

/// Request body for vocabulary autocomplete.
#[derive(Deserialize)]
struct VocabularyRequest {
    query: String,
    graph: String,
}

/// POST /api/search/vocabulary — vocabulary autocomplete.
/// Returns matching suggestions from the in-memory vocabulary index.
async fn vocabulary_search(
    State(state): State<AppState>,
    Json(body): Json<VocabularyRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let indexes = state.vocab_indexes.read().await;

    let index = indexes.get(&body.graph).ok_or((
        StatusCode::NOT_FOUND,
        format!(
            "No vocabulary index for graph '{}'. Fetch the schema first via GET /api/graphs/{}/schema",
            body.graph, body.graph
        ),
    ))?;

    debug!(
        "Vocabulary search: query='{}', graph='{}'",
        body.query, body.graph
    );

    let suggestions = index.autocomplete(&body.query);

    Ok(Json(suggestions))
}
