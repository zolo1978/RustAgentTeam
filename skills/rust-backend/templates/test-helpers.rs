// Template: Test Helpers — Tauri Commands + Axum Handlers
// Copy-paste ready. Adapt AppState and test fixtures to your types.

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use http::{Request, StatusCode};
    use tower::ServiceExt; // enables oneshot()

    // -- Shared test fixtures --

    fn test_state() -> Arc<AppState> {
        Arc::new(AppState {
            order_svc: OrderService::new(/* test db / mock */),
        })
    }

    // -- Pattern 1: Tauri Command (call function directly) --

    #[tokio::test]
    async fn test_tauri_create_order() {
        let state = test_state();
        let req = CreateOrderReq {
            title: "test order".into(),
            items: vec![],
        };
        // Tauri command is a plain async fn — call it directly
        let result = create_order(req, tauri::State::from(&*state)).await;
        assert!(result.is_ok());
        let dto = result.unwrap();
        assert_eq!(dto.title, "test order");
    }

    // -- Pattern 2: Axum Handler (oneshot integration test) --

    #[tokio::test]
    async fn test_axum_create_order() {
        let state = test_state();
        let app = router(state);

        let body = serde_json::json!({
            "title": "test order",
            "items": []
        });
        let req = Request::builder()
            .method("POST")
            .uri("/orders")
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let resp = app.oneshot(req).await.unwrap();
        assert_eq!(resp.status(), StatusCode::CREATED);
    }
}
