pub mod graph;
pub mod search;

use axum::Router;
use crate::AppState;

/// Build the complete API router.
pub fn router() -> Router<AppState> {
    Router::new()
        .merge(graph::router())
        .merge(search::router())
}
