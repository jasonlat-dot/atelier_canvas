use super::*;
#[test]
fn destination_validation_is_explicit_and_development_scoped() {
    assert_eq!(
        normalize_url("https://example.test/v1", false).unwrap(),
        "https://example.test/v1/"
    );
    assert!(normalize_url("http://127.0.0.1:1234/v1", true).is_ok());
    for url in [
        "http://example.test/v1",
        "https://user:secret@example.test",
        "https://example.test?key=value",
        "https://example.test/#fragment",
        "file:///tmp/key",
    ] {
        assert!(normalize_url(url, true).is_err());
    }
    assert!(normalize_url("http://127.0.0.1:1234/v1", false).is_err());
}
#[test]
fn token_limits_and_unknown_fields_are_rejected() {
    let provider = Uuid::new_v4();
    let mut value = json!({"providerId":provider,"modelId":"sample","label":"Sample","enabled":true,"contextWindow":1024,"maxOutputTokens":1024,"revision":0});
    assert!(serde_json::from_value::<ModelWrite>(value.clone())
        .unwrap()
        .validate()
        .is_err());
    value["maxOutputTokens"] = json!(64);
    assert!(serde_json::from_value::<ModelWrite>(value.clone())
        .unwrap()
        .validate()
        .is_ok());
    value["role"] = json!("ADMIN");
    assert!(serde_json::from_value::<ModelWrite>(value).is_err());
}
#[tokio::test]
async fn upstream_error_bodies_and_redirects_never_escape_or_forward_secrets() {
    use axum::response::IntoResponse;
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new()
        .route(
            "/auth/models",
            get(|| async {
                (
                    StatusCode::UNAUTHORIZED,
                    "reflected secret should never be returned",
                )
            }),
        )
        .route(
            "/redirect/models",
            get(|| async {
                (
                    StatusCode::FOUND,
                    [("location", "http://127.0.0.1:1/stolen")],
                )
                    .into_response()
            }),
        )
        .route(
            "/bad/models",
            get(|| async { Json(json!({"notModels":[]})) }),
        );
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    assert_eq!(
        probe(&format!("http://{addr}/auth/"), Some("synthetic-key"))
            .await
            .unwrap_err(),
        "UPSTREAM_AUTH"
    );
    assert_eq!(
        probe(&format!("http://{addr}/redirect/"), Some("synthetic-key"))
            .await
            .unwrap_err(),
        "UPSTREAM_REDIRECT"
    );
    assert_eq!(
        probe(&format!("http://{addr}/bad/"), None)
            .await
            .unwrap_err(),
        "UPSTREAM_FORMAT"
    );
    task.abort();
}
#[tokio::test]
#[ignore = "requires an isolated atelier_test_* PostgreSQL database; run scripts/test-providers.ps1"]
async fn postgres_provider_lifecycle() {
    use axum::{
        body::{to_bytes, Body},
        http::Request,
    };
    use base64::{engine::general_purpose::STANDARD, Engine};
    use std::collections::HashMap;
    use tokio::sync::RwLock;
    use tower::ServiceExt;
    async fn call(
        app: &Router,
        method: &str,
        path: &str,
        cookie: Option<&str>,
        body: Value,
    ) -> (StatusCode, Value, Option<String>) {
        let mut req = Request::builder()
            .method(method)
            .uri(path)
            .header("content-type", "application/json");
        if let Some(cookie) = cookie {
            req = req.header("cookie", cookie);
        }
        let response = app
            .clone()
            .oneshot(req.body(Body::from(body.to_string())).unwrap())
            .await
            .unwrap();
        let status = response.status();
        let cookie = response
            .headers()
            .get("set-cookie")
            .map(|h| h.to_str().unwrap().split(';').next().unwrap().to_owned());
        let bytes = to_bytes(response.into_body(), 2_000_000).await.unwrap();
        let data = serde_json::from_slice(&bytes).unwrap();
        (status, data, cookie)
    }
    let url = std::env::var("ATELIER_TEST_DATABASE_URL").expect("Use scripts/test-providers.ps1");
    let parsed = Url::parse(&url).unwrap();
    assert!(
        parsed.path().starts_with("/atelier_test_"),
        "Refusing non-test database"
    );
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    crate::seed_development_users(&pool).await.unwrap();
    let cipher = Arc::new(
        crate::credentials::CredentialCipher::from_base64(&STANDARD.encode([9u8; 32])).unwrap(),
    );
    let state = Arc::new(AppState {
        db: pool.clone(),
        sessions: Arc::new(RwLock::new(HashMap::new())),
        dev_auth: true,
        cipher: Some(cipher.clone()),
    });
    let app = crate::app_router(state.clone());
    assert_eq!(
        call(&app, "GET", "/api/v1/admin/providers", None, json!({}))
            .await
            .0,
        StatusCode::UNAUTHORIZED
    );
    let user = call(
        &app,
        "POST",
        "/api/v1/dev/login",
        None,
        json!({"userId":"user-demo"}),
    )
    .await
    .2
    .unwrap();
    let admin = call(
        &app,
        "POST",
        "/api/v1/dev/login",
        None,
        json!({"userId":"admin-demo"}),
    )
    .await
    .2
    .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/admin/providers",
            Some(&user),
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let upstream = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let address = upstream.local_addr().unwrap();
    let upstream_task = tokio::spawn(async move {
        axum::serve(
            upstream,
            Router::new().route(
                "/v1/models",
                get(|headers: HeaderMap| async move {
                    assert_eq!(
                        headers.get("authorization").unwrap(),
                        "Bearer synthetic-credential"
                    );
                    Json(json!({"data":[{"id":"test-text-model"}]}))
                }),
            ),
        )
        .await
        .unwrap()
    });
    let mut input = json!({"name":"Test provider","protocol":"chat_completions","baseUrl":format!("http://{address}/v1"),"enabled":true,"requireApiKey":true,"apiKey":"synthetic-credential","revision":0});
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/admin/providers",
            Some(&user),
            input.clone()
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    let forged_origin = app
        .clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/admin/providers")
                .header("content-type", "application/json")
                .header("cookie", &admin)
                .header("origin", "https://untrusted.example")
                .body(Body::from(input.to_string()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(forged_origin.status(), StatusCode::FORBIDDEN);
    let no_cipher = crate::app_router(Arc::new(AppState {
        db: pool.clone(),
        sessions: state.sessions.clone(),
        dev_auth: true,
        cipher: None,
    }));
    assert_eq!(
        call(
            &no_cipher,
            "POST",
            "/api/v1/admin/providers",
            Some(&admin),
            input.clone()
        )
        .await
        .0,
        StatusCode::SERVICE_UNAVAILABLE
    );
    let (status, provider, _) = call(
        &app,
        "POST",
        "/api/v1/admin/providers",
        Some(&admin),
        input.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{provider}");
    assert_eq!(provider["hasApiKey"], true);
    assert!(provider.get("apiKey").is_none());
    assert!(!provider.to_string().contains("synthetic-credential"));
    let id = Uuid::parse_str(provider["id"].as_str().unwrap()).unwrap();
    let path = format!("/api/v1/admin/providers/{id}");
    let stored: Vec<u8> =
        sqlx::query_scalar("SELECT encrypted_api_key FROM text_providers WHERE id=$1")
            .bind(id)
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(!String::from_utf8_lossy(&stored).contains("synthetic-credential"));
    assert_eq!(cipher.decrypt(id, &stored).unwrap(), "synthetic-credential");
    let result = call(
        &app,
        "POST",
        &format!("{path}/test"),
        Some(&admin),
        json!({"revision":1}),
    )
    .await;
    assert_eq!(result.0, StatusCode::OK);
    assert_eq!(result.1["ok"], true);
    assert_eq!(result.1["models"][0], "test-text-model");
    let mut model = json!({"providerId":id,"modelId":"test-text-model","label":"Test Text","enabled":true,"contextWindow":32768,"maxOutputTokens":1024,"revision":0});
    let (status, text_model, _) = call(
        &app,
        "POST",
        "/api/v1/admin/text-models",
        Some(&admin),
        model.clone(),
    )
    .await;
    assert_eq!(status, StatusCode::OK, "{text_model}");
    assert_eq!(
        call(
            &app,
            "POST",
            "/api/v1/admin/text-models",
            Some(&admin),
            model.clone()
        )
        .await
        .0,
        StatusCode::CONFLICT
    );
    let catalog = call(
        &app,
        "GET",
        "/api/v1/text-model-catalog",
        Some(&user),
        json!({}),
    )
    .await
    .1;
    assert_eq!(catalog.as_array().unwrap().len(), 1);
    assert!(catalog[0].get("modelId").is_none());
    assert!(catalog[0].get("baseUrl").is_none());
    input.as_object_mut().unwrap().remove("apiKey");
    input["revision"] = json!(1);
    input["enabled"] = json!(false);
    let updated = call(&app, "PUT", &path, Some(&admin), input.clone()).await;
    assert_eq!(updated.0, StatusCode::OK);
    assert_eq!(updated.1["revision"], 2);
    assert_eq!(updated.1["hasApiKey"], true);
    assert_eq!(updated.1["lastTestStatus"], Value::Null);
    assert_eq!(
        call(&app, "PUT", &path, Some(&admin), input.clone())
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert!(call(
        &app,
        "GET",
        "/api/v1/text-model-catalog",
        Some(&user),
        json!({})
    )
    .await
    .1
    .as_array()
    .unwrap()
    .is_empty());
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("{path}/test"),
            Some(&admin),
            json!({"revision":2})
        )
        .await
        .0,
        StatusCode::BAD_REQUEST
    );
    input["revision"] = json!(2);
    input["baseUrl"] = json!("http://127.0.0.1:1/v1");
    assert_eq!(
        call(&app, "PUT", &path, Some(&admin), input.clone())
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    input["baseUrl"] = provider["baseUrl"].clone();
    input["enabled"] = json!(true);
    assert_eq!(
        call(&app, "PUT", &path, Some(&admin), input.clone())
            .await
            .0,
        StatusCode::OK
    );
    model["revision"] = json!(1);
    model["enabled"] = json!(false);
    let model_path = format!(
        "/api/v1/admin/text-models/{}",
        text_model["id"].as_str().unwrap()
    );
    assert_eq!(
        call(&app, "PUT", &model_path, Some(&admin), model.clone())
            .await
            .0,
        StatusCode::OK
    );
    assert_eq!(
        call(&app, "PUT", &model_path, Some(&admin), model.clone())
            .await
            .0,
        StatusCode::CONFLICT
    );
    assert!(call(
        &app,
        "GET",
        "/api/v1/text-model-catalog",
        Some(&user),
        json!({})
    )
    .await
    .1
    .as_array()
    .unwrap()
    .is_empty());
    drop(app);
    pool.close().await;
    let reopened = sqlx::postgres::PgPoolOptions::new()
        .max_connections(3)
        .connect(&url)
        .await
        .unwrap();
    let new_state = Arc::new(AppState {
        db: reopened.clone(),
        sessions: Arc::new(RwLock::new(HashMap::new())),
        dev_auth: true,
        cipher: Some(cipher),
    });
    let app = crate::app_router(new_state);
    let admin = call(
        &app,
        "POST",
        "/api/v1/dev/login",
        None,
        json!({"userId":"admin-demo"}),
    )
    .await
    .2
    .unwrap();
    let list = call(
        &app,
        "GET",
        "/api/v1/admin/providers",
        Some(&admin),
        json!({}),
    )
    .await
    .1;
    assert_eq!(list[0]["revision"], 3);
    assert_eq!(list[0]["hasApiKey"], true);
    assert_eq!(
        call(
            &app,
            "POST",
            &format!("{path}/test"),
            Some(&admin),
            json!({"revision":3})
        )
        .await
        .1["ok"],
        true
    );
    let audit: Vec<Value> = sqlx::query_scalar("SELECT details FROM provider_admin_events")
        .fetch_all(&reopened)
        .await
        .unwrap();
    assert!(audit.len() >= 5);
    assert!(audit
        .iter()
        .all(|v| !v.to_string().contains("synthetic-credential")));
    input["revision"] = json!(3);
    input["clearApiKey"] = json!(true);
    assert_eq!(
        call(&app, "PUT", &path, Some(&admin), input.clone())
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
    input["requireApiKey"] = json!(false);
    let cleared = call(&app, "PUT", &path, Some(&admin), input).await;
    assert_eq!(cleared.0, StatusCode::OK);
    assert_eq!(cleared.1["hasApiKey"], false);
    sqlx::query("UPDATE users SET status='DISABLED' WHERE email='jordan@gmail.com'")
        .execute(&reopened)
        .await
        .unwrap();
    assert_eq!(
        call(
            &app,
            "GET",
            "/api/v1/admin/providers",
            Some(&admin),
            json!({})
        )
        .await
        .0,
        StatusCode::FORBIDDEN
    );
    upstream_task.abort();
    reopened.close().await;
}

#[tokio::test]
async fn upstream_timeout_and_connection_failure_are_classified() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let app = Router::new().route(
        "/models",
        get(|| async {
            tokio::time::sleep(Duration::from_secs(30)).await;
            Json(json!({"data":[]}))
        }),
    );
    let task = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    assert_eq!(
        probe(&format!("http://{addr}/"), None).await.unwrap_err(),
        "UPSTREAM_TIMEOUT"
    );
    task.abort();
    let unused = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let closed = unused.local_addr().unwrap();
    drop(unused);
    assert_eq!(
        probe(&format!("http://{closed}/"), None).await.unwrap_err(),
        "UPSTREAM_CONNECT"
    );
}
