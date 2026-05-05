// Template: Axum REST Endpoint — router + create_order handler
// Copy-paste ready. Adapt struct, service call, and status code to your domain.

use axum::{
    extract::State,
    routing::post,
    Router,
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;
use validator::Validate;

pub fn router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/orders", post(create_order))
        .with_state(state)
}

async fn create_order(
    State(s): State<Arc<AppState>>,
    Json(req): Json<CreateOrderReq>,
) -> Result<(StatusCode, Json<ApiResponse<OrderDto>>), AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success(
            s.order_svc.create(req.into()).await?.into(),
        )),
    ))
}
