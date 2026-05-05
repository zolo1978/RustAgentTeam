// Template: Cursor-based Paged List — PageReq + PageResp + list handler
// Copy-paste ready. Adapt entity type, sort fields, and cursor encoding to your domain.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde::{Deserialize, Serialize};
use tauri::State;

/// Request: optional cursor + optional limit
#[derive(Deserialize)]
pub struct PageReq {
    pub cursor: Option<String>,
    pub limit: Option<usize>,
}

/// Response: items + next_cursor + has_more flag
#[derive(Serialize)]
pub struct PageResp<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

#[tauri::command]
pub async fn list_orders(
    req: PageReq,
    state: State<'_, AppState>,
) -> Result<PageResp<OrderDto>, AppError> {
    let limit = req.limit.unwrap_or(50).min(200);
    // fetch limit+1 to detect has_more without extra query
    let items = state.order_svc.list(req.cursor.as_deref(), limit + 1).await?;
    let has_more = items.len() > limit;
    let items: Vec<OrderDto> = items.into_iter().take(limit).map(Into::into).collect();

    // cursor: composite encoding of sort field + ID
    let next_cursor = if has_more {
        items.last().map(|o| {
            BASE64.encode(format!("{}:{}", o.created_at.timestamp_millis(), o.id))
        })
    } else {
        None
    };

    Ok(PageResp {
        items,
        next_cursor,
        has_more,
    })
}
