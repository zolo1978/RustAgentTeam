// Template: Tauri Command — CreateOrderReq + create_order
// Copy-paste ready. Adapt struct fields and service call to your domain.

use serde::Deserialize;
use validator::Validate;
use tauri::State;

#[derive(Deserialize, Validate)]
pub struct CreateOrderReq {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub items: Vec<OrderItem>,
}

#[tauri::command]
pub async fn create_order(
    req: CreateOrderReq,
    state: State<'_, AppState>,
) -> Result<OrderDto, AppError> {
    req.validate()
        .map_err(|e| AppError::Validation(e.to_string()))?;
    Ok(state.order_svc.create(req.into()).await?.into())
}
