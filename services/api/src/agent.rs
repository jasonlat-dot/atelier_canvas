//! Product capabilities are separate from compiled core primitives.
//! Set a feature true only after its authenticated end-to-end path is implemented.
use crate::{actor, ApiResult, AppState};
use axum::{extract::State, http::HeaderMap, Json};
use serde_json::{json, Value};
use std::sync::Arc;

fn capabilities() -> Value {
    json!({
        "contractVersion": 1,
        "mode": "http",
        "core": atelier_agent_core::capabilities(),
        "features": {
            "chat": false,
            "history": false,
            "tools": false,
            "approvals": false,
            "compaction": false,
            "memory": false
        }
    })
}

pub(crate) async fn get_capabilities(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<Value>> {
    actor(&headers, &state).await?;
    Ok(Json(capabilities()))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn compiled_primitives_never_claim_connected_agent_features() {
        let value = capabilities();
        assert_eq!(value["contractVersion"], 1);
        assert_eq!(value["core"]["toolArgumentValidation"], true);
        assert_eq!(value["core"]["contextBudgetPlanning"], true);
        for (_, enabled) in value["features"].as_object().unwrap() {
            assert_eq!(enabled, false);
        }
    }
    #[tokio::test]
    async fn missing_and_forged_sessions_cannot_read_capabilities() {
        use axum::http::{header, StatusCode};
        use std::collections::HashMap;
        use tokio::sync::RwLock;
        let state = Arc::new(AppState {
            db: sqlx::postgres::PgPoolOptions::new()
                .connect_lazy("postgres://unused:unused@127.0.0.1:1/unused")
                .unwrap(),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            dev_auth: false,
        });
        let mut headers = HeaderMap::new();
        assert_eq!(
            get_capabilities(State(state.clone()), headers.clone())
                .await
                .unwrap_err()
                .0,
            StatusCode::UNAUTHORIZED
        );
        headers.insert(header::COOKIE, "atelier_sid=forged".parse().unwrap());
        assert_eq!(
            get_capabilities(State(state), headers).await.unwrap_err().0,
            StatusCode::UNAUTHORIZED
        );
    }
}
