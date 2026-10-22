//! S1 host-owned provider configuration. Text models are independent of mock generation models.
use crate::{actor, bad, forbidden, not_found, ApiError, ApiResult, AppState};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    routing::{get, post},
    Json, Router,
};
use chrono::{DateTime, Utc};
use reqwest::{redirect::Policy, Url};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::{postgres::PgRow, Row};
use std::{
    sync::Arc,
    time::{Duration, Instant},
};
use uuid::Uuid;

pub(crate) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/v1/admin/providers",
            get(list_providers).post(create_provider),
        )
        .route(
            "/api/v1/admin/providers/{id}",
            axum::routing::put(update_provider),
        )
        .route("/api/v1/admin/providers/{id}/test", post(test_provider))
        .route(
            "/api/v1/admin/text-models",
            get(list_models).post(create_model),
        )
        .route(
            "/api/v1/admin/text-models/{id}",
            axum::routing::put(update_model),
        )
        .route("/api/v1/text-model-catalog", get(public_models))
}
async fn admin(headers: &HeaderMap, state: &AppState, write: bool) -> ApiResult<Uuid> {
    let id = actor(headers, state).await?;
    let role: String = sqlx::query_scalar("SELECT role FROM users WHERE id=$1")
        .bind(id)
        .fetch_one(&state.db)
        .await?;
    if !["ADMIN", "SUPER_ADMIN"].contains(&role.as_str()) {
        return Err(forbidden());
    }
    if write {
        if let Some(origin) = headers.get("origin") {
            let origin = origin.to_str().map_err(|_| forbidden())?;
            let configured =
                std::env::var("WEB_ORIGIN").unwrap_or_else(|_| "http://127.0.0.1:5173".into());
            if origin != configured
                && !(state.dev_auth
                    && [
                        "http://localhost:5173",
                        "http://127.0.0.1:8080",
                        "http://localhost:8080",
                    ]
                    .contains(&origin))
            {
                return Err(forbidden());
            }
        }
    }
    Ok(id)
}
fn conflict() -> ApiError {
    ApiError(
        StatusCode::CONFLICT,
        "configuration changed; reload before saving".into(),
    )
}
fn db_error(error: sqlx::Error) -> ApiError {
    if error.as_database_error().and_then(|e| e.code()).as_deref() == Some("23505") {
        ApiError(
            StatusCode::CONFLICT,
            "model already exists for this provider".into(),
        )
    } else {
        error.into()
    }
}
fn printable(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.len() <= max && !value.chars().any(char::is_control)
}
fn normalize_url(value: &str, dev: bool) -> ApiResult<String> {
    let mut url = Url::parse(value.trim()).map_err(|_| bad("invalid base URL"))?;
    let loopback = url.host_str().is_some_and(|h| {
        h == "localhost"
            || h.parse::<std::net::IpAddr>()
                .is_ok_and(|ip| ip.is_loopback())
            || h == "[::1]"
    });
    if url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
        || url.as_str().len() > 2048
        || !(url.scheme() == "https" || (dev && url.scheme() == "http" && loopback))
    {
        return Err(bad("use HTTPS base URL; HTTP loopback is allowed only in development; credentials/query/fragment are forbidden"));
    }
    let path = format!("{}/", url.path().trim_end_matches('/'));
    url.set_path(&path);
    Ok(url.to_string())
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ProviderWrite {
    name: String,
    protocol: String,
    base_url: String,
    enabled: bool,
    require_api_key: bool,
    #[serde(default)]
    api_key: Option<String>,
    #[serde(default)]
    clear_api_key: bool,
    revision: i64,
}
impl ProviderWrite {
    fn validate(&self, dev: bool) -> ApiResult<String> {
        if !printable(&self.name, 120) || self.protocol != "chat_completions" || self.revision < 0 {
            return Err(bad("invalid provider name, protocol or revision"));
        }
        if self.clear_api_key && self.api_key.is_some() {
            return Err(bad("cannot replace and remove key together"));
        }
        if let Some(key) = &self.api_key {
            if !printable(key, 8192) || key.trim() != key {
                return Err(bad("invalid API key"));
            }
        }
        normalize_url(&self.base_url, dev)
    }
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProviderView {
    id: Uuid,
    name: String,
    protocol: String,
    base_url: String,
    enabled: bool,
    require_api_key: bool,
    has_api_key: bool,
    revision: i64,
    last_test_status: Option<String>,
    last_test_code: Option<String>,
    last_test_at: Option<DateTime<Utc>>,
}
fn provider_view(row: &PgRow) -> ProviderView {
    ProviderView {
        id: row.get("id"),
        name: row.get("name"),
        protocol: row.get("protocol"),
        base_url: row.get("base_url"),
        enabled: row.get("enabled"),
        require_api_key: row.get("require_api_key"),
        has_api_key: row.get::<Option<Vec<u8>>, _>("encrypted_api_key").is_some(),
        revision: row.get("revision"),
        last_test_status: row.get("last_test_status"),
        last_test_code: row.get("last_test_code"),
        last_test_at: row.get("last_test_at"),
    }
}
async fn list_providers(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<ProviderView>>> {
    admin(&headers, &state, false).await?;
    let rows = sqlx::query("SELECT * FROM text_providers ORDER BY created_at,id LIMIT 500")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(rows.iter().map(provider_view).collect()))
}
async fn create_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<ProviderWrite>,
) -> ApiResult<Json<ProviderView>> {
    save_provider(state, headers, None, input).await
}
async fn update_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<ProviderWrite>,
) -> ApiResult<Json<ProviderView>> {
    save_provider(state, headers, Some(id), input).await
}
async fn save_provider(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Option<Uuid>,
    input: ProviderWrite,
) -> ApiResult<Json<ProviderView>> {
    let actor = admin(&headers, &state, true).await?;
    let base = input.validate(state.dev_auth)?;
    if id.is_none() && input.revision != 0 {
        return Err(bad("new provider revision must be zero"));
    }
    let resource = id.unwrap_or_else(Uuid::new_v4);
    let mut tx = state.db.begin().await?;
    let existing = if id.is_some() {
        Some(
            sqlx::query("SELECT * FROM text_providers WHERE id=$1 FOR UPDATE")
                .bind(resource)
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(not_found)?,
        )
    } else {
        None
    };
    if existing
        .as_ref()
        .is_some_and(|r| r.get::<i64, _>("revision") != input.revision)
    {
        return Err(conflict());
    }
    let mut encrypted = existing
        .as_ref()
        .and_then(|r| r.get::<Option<Vec<u8>>, _>("encrypted_api_key"));
    // A stored credential may only be kept while its destination is unchanged.
    if existing
        .as_ref()
        .is_some_and(|r| r.get::<String, _>("base_url") != base)
        && input.api_key.is_none()
        && encrypted.is_some()
        && !input.clear_api_key
    {
        return Err(bad(
            "replace or remove the credential when changing its destination",
        ));
    }
    if input.clear_api_key {
        encrypted = None;
    }
    if let Some(key) = input.api_key {
        let cipher = state.cipher.as_ref().ok_or_else(|| {
            ApiError(
                StatusCode::SERVICE_UNAVAILABLE,
                "configure ATELIER_CREDENTIAL_KEY before saving credentials".into(),
            )
        })?;
        encrypted = Some(cipher.encrypt(resource, &key)?);
    }
    if input.require_api_key && encrypted.is_none() {
        return Err(bad("this provider requires an API key"));
    }
    let row = if id.is_none() {
        sqlx::query("INSERT INTO text_providers(id,name,protocol,base_url,enabled,require_api_key,encrypted_api_key) VALUES($1,$2,$3,$4,$5,$6,$7) RETURNING *")
   .bind(resource).bind(input.name.trim()).bind(input.protocol).bind(base).bind(input.enabled).bind(input.require_api_key).bind(encrypted).fetch_one(&mut *tx).await?
    } else {
        sqlx::query("UPDATE text_providers SET name=$1,base_url=$2,enabled=$3,require_api_key=$4,encrypted_api_key=$5,revision=revision+1,last_test_status=NULL,last_test_code=NULL,last_test_at=NULL,updated_at=now() WHERE id=$6 RETURNING *")
   .bind(input.name.trim()).bind(base).bind(input.enabled).bind(input.require_api_key).bind(encrypted).bind(resource).fetch_one(&mut *tx).await?
    };
    let result = provider_view(&row);
    sqlx::query("INSERT INTO provider_admin_events(actor_id,action,resource_id,details) VALUES($1,$2,$3,$4)")
  .bind(actor).bind(if id.is_some(){"provider.update"}else{"provider.create"}).bind(resource).bind(json!({"revision":result.revision,"enabled":result.enabled})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(result))
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TestInput {
    revision: i64,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TestResult {
    ok: bool,
    code: &'static str,
    models: Vec<String>,
    latency_ms: u64,
}
async fn probe(base: &str, key: Option<&str>) -> Result<Vec<String>, &'static str> {
    let url = Url::parse(base)
        .and_then(|base| base.join("models"))
        .map_err(|_| "INVALID_ENDPOINT")?;
    let client = reqwest::Client::builder()
        .redirect(Policy::none())
        .no_proxy()
        .connect_timeout(Duration::from_secs(4))
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|_| "CLIENT_ERROR")?;
    let mut request = client.get(url).header("Accept", "application/json");
    if let Some(key) = key {
        request = request.bearer_auth(key);
    }
    let mut response = request.send().await.map_err(|e| {
        if e.is_timeout() {
            "UPSTREAM_TIMEOUT"
        } else {
            "UPSTREAM_CONNECT"
        }
    })?;
    if !response.status().is_success() {
        return Err(match response.status().as_u16() {
            401 | 403 => "UPSTREAM_AUTH",
            429 => "UPSTREAM_RATE_LIMIT",
            300..=399 => "UPSTREAM_REDIRECT",
            _ => "UPSTREAM_HTTP",
        });
    }
    let mut bytes = vec![];
    while let Some(chunk) = response.chunk().await.map_err(|e| {
        if e.is_timeout() {
            "UPSTREAM_TIMEOUT"
        } else {
            "UPSTREAM_READ"
        }
    })? {
        if bytes.len() + chunk.len() > 1_048_576 {
            return Err("UPSTREAM_TOO_LARGE");
        }
        bytes.extend_from_slice(&chunk);
    }
    let value: Value = serde_json::from_slice(&bytes).map_err(|_| "UPSTREAM_FORMAT")?;
    let data = value
        .get("data")
        .and_then(Value::as_array)
        .ok_or("UPSTREAM_FORMAT")?;
    if data.len() > 4096 {
        return Err("UPSTREAM_TOO_LARGE");
    }
    let mut models = std::collections::BTreeSet::new();
    for item in data {
        let id = item
            .get("id")
            .and_then(Value::as_str)
            .filter(|id| printable(id, 256))
            .ok_or("UPSTREAM_FORMAT")?;
        models.insert(id.to_owned());
    }
    Ok(models.into_iter().collect())
}
async fn test_provider(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<TestInput>,
) -> ApiResult<Json<TestResult>> {
    admin(&headers, &state, true).await?;
    let row = sqlx::query("SELECT * FROM text_providers WHERE id=$1")
        .bind(id)
        .fetch_optional(&state.db)
        .await?
        .ok_or_else(not_found)?;
    if row.get::<i64, _>("revision") != input.revision {
        return Err(conflict());
    }
    if !row.get::<bool, _>("enabled") {
        return Err(bad("enable the provider before testing"));
    }
    let encrypted: Option<Vec<u8>> = row.get("encrypted_api_key");
    let key = match encrypted {
        Some(bytes) => Some(
            state
                .cipher
                .as_ref()
                .ok_or_else(|| {
                    ApiError(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "credential encryption unavailable".into(),
                    )
                })?
                .decrypt(id, &bytes)?,
        ),
        None => None,
    };
    let start = Instant::now();
    let result = probe(&row.get::<String, _>("base_url"), key.as_deref()).await;
    let code = result.as_ref().err().copied().unwrap_or("OK");
    let ok = result.is_ok();
    let updated=sqlx::query("UPDATE text_providers SET last_test_status=$1,last_test_code=$2,last_test_at=now() WHERE id=$3 AND revision=$4")
  .bind(if ok{"ok"}else{"error"}).bind(code).bind(id).bind(input.revision).execute(&state.db).await?;
    if updated.rows_affected() != 1 {
        return Err(conflict());
    }
    Ok(Json(TestResult {
        ok,
        code,
        models: result.unwrap_or_default(),
        latency_ms: start.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
    }))
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ModelWrite {
    provider_id: Uuid,
    model_id: String,
    label: String,
    enabled: bool,
    context_window: i32,
    max_output_tokens: i32,
    revision: i64,
}
impl ModelWrite {
    fn validate(&self) -> ApiResult<()> {
        if !printable(&self.model_id, 256)
            || !printable(&self.label, 120)
            || !(1024..=4_000_000).contains(&self.context_window)
            || self.max_output_tokens <= 0
            || self.max_output_tokens >= self.context_window
            || self.revision < 0
        {
            return Err(bad("invalid text model or token limits"));
        }
        Ok(())
    }
}
fn model_view(row: &PgRow) -> Value {
    json!({"id":row.get::<Uuid,_>("id"),"providerId":row.get::<Uuid,_>("provider_id"),"modelId":row.get::<String,_>("model_id"),"label":row.get::<String,_>("label"),"enabled":row.get::<bool,_>("enabled"),"contextWindow":row.get::<i32,_>("context_window"),"maxOutputTokens":row.get::<i32,_>("max_output_tokens"),"revision":row.get::<i64,_>("revision")})
}
async fn list_models(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Value>>> {
    admin(&headers, &state, false).await?;
    let rows = sqlx::query("SELECT * FROM text_models ORDER BY updated_at DESC,id LIMIT 500")
        .fetch_all(&state.db)
        .await?;
    Ok(Json(rows.iter().map(model_view).collect()))
}
async fn create_model(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(input): Json<ModelWrite>,
) -> ApiResult<Json<Value>> {
    save_model(state, headers, None, input).await
}
async fn update_model(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
    Path(id): Path<Uuid>,
    Json(input): Json<ModelWrite>,
) -> ApiResult<Json<Value>> {
    save_model(state, headers, Some(id), input).await
}
async fn save_model(
    state: Arc<AppState>,
    headers: HeaderMap,
    id: Option<Uuid>,
    input: ModelWrite,
) -> ApiResult<Json<Value>> {
    let actor = admin(&headers, &state, true).await?;
    input.validate()?;
    if id.is_none() && input.revision != 0 {
        return Err(bad("new model revision must be zero"));
    }
    let mut tx = state.db.begin().await?;
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM text_providers WHERE id=$1)")
            .bind(input.provider_id)
            .fetch_one(&mut *tx)
            .await?;
    if !exists {
        return Err(bad("provider does not exist"));
    }
    let row = if let Some(id) = id {
        sqlx::query("UPDATE text_models SET provider_id=$1,model_id=$2,label=$3,enabled=$4,context_window=$5,max_output_tokens=$6,revision=revision+1,updated_at=now() WHERE id=$7 AND revision=$8 RETURNING *")
   .bind(input.provider_id).bind(input.model_id.trim()).bind(input.label.trim()).bind(input.enabled).bind(input.context_window).bind(input.max_output_tokens).bind(id).bind(input.revision)
   .fetch_optional(&mut *tx).await.map_err(db_error)?.ok_or_else(conflict)?
    } else {
        sqlx::query("INSERT INTO text_models(provider_id,model_id,label,enabled,context_window,max_output_tokens) VALUES($1,$2,$3,$4,$5,$6) RETURNING *")
   .bind(input.provider_id).bind(input.model_id.trim()).bind(input.label.trim()).bind(input.enabled).bind(input.context_window).bind(input.max_output_tokens)
   .fetch_one(&mut *tx).await.map_err(db_error)?
    };
    let result = model_view(&row);
    sqlx::query("INSERT INTO provider_admin_events(actor_id,action,resource_id,details) VALUES($1,$2,$3,$4)")
 .bind(actor).bind(if id.is_some(){"model.update"}else{"model.create"}).bind(row.get::<Uuid,_>("id")).bind(json!({"revision":row.get::<i64,_>("revision"),"enabled":input.enabled})).execute(&mut *tx).await?;
    tx.commit().await?;
    Ok(Json(result))
}
async fn public_models(
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> ApiResult<Json<Vec<Value>>> {
    actor(&headers, &state).await?;
    let rows=sqlx::query("SELECT m.id,m.label,m.context_window,m.max_output_tokens FROM text_models m JOIN text_providers p ON p.id=m.provider_id WHERE m.enabled AND p.enabled AND (NOT p.require_api_key OR p.encrypted_api_key IS NOT NULL) ORDER BY m.label,m.id LIMIT 500").fetch_all(&state.db).await?;
    Ok(Json(rows.iter().map(|r|json!({"id":r.get::<Uuid,_>("id"),"label":r.get::<String,_>("label"),"contextWindow":r.get::<i32,_>("context_window"),"maxOutputTokens":r.get::<i32,_>("max_output_tokens")})).collect()))
}
#[cfg(test)]
mod tests;
