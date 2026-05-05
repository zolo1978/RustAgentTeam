# Error Code Directory Reference

## AppError Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Validation(String),
    #[error("{0}")]
    NotFound(String),
    #[error("{0}")]
    Auth(String),
    #[error("{0}")]
    Internal(String),
}

impl Serialize for AppError {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let code = match &self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Auth(_) => StatusCode::UNAUTHORIZED,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (code, Json(ApiResponse::<()>::error_from(self))).into_response()
    }
}

impl From<AppError> for tauri::ipc::InvokeError {
    fn from(e: AppError) -> Self { Self::from(e.to_string()) }
}
```

## ErrorCode Enum

```rust
#[derive(Debug, thiserror::Error)]
pub enum ErrorCode {
    #[error("validation: field {field} failed rule {rule}")]
    VALIDATION_001 { field: String, rule: String },
    #[error("validation: request body format error")]
    VALIDATION_002,
    #[error("auth: token missing")]
    AUTH_001,
    #[error("auth: token expired")]
    AUTH_002,
    #[error("auth: insufficient permissions")]
    AUTH_003,
    #[error("not found: {resource}")]
    NOT_FOUND_001 { resource: String },
    #[error("conflict: unique key violation on {key}")]
    CONFLICT_001 { key: String },
    #[error("internal: {detail}")]
    INTERNAL_001 { detail: String },
    #[error("rate limit exceeded, retry after {seconds}s")]
    RATE_LIMIT_001 { seconds: u64 },
}
```

## HTTP Status Code Mapping

```rust
use axum::http::StatusCode;

impl ErrorCode {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::VALIDATION_001 { .. } | Self::VALIDATION_002 => StatusCode::BAD_REQUEST,
            Self::AUTH_001 => StatusCode::UNAUTHORIZED,
            Self::AUTH_002 => StatusCode::UNAUTHORIZED,
            Self::AUTH_003 => StatusCode::FORBIDDEN,
            Self::NOT_FOUND_001 { .. } => StatusCode::NOT_FOUND,
            Self::CONFLICT_001 { .. } => StatusCode::CONFLICT,
            Self::INTERNAL_001 { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RATE_LIMIT_001 { .. } => StatusCode::TOO_MANY_REQUESTS,
        }
    }
}
```

## ApiResponse Struct

```rust
#[derive(Debug, Serialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct ApiMeta {
    pub request_id: String,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<ApiError>,
    pub meta: Option<ApiMeta>,
}

impl<T: Serialize> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            meta: Some(ApiMeta {
                request_id: uuid::Uuid::now_v7().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }),
        }
    }

    pub fn error(code: ErrorCode, msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: format!("{:?}", code).split_whitespace().next().unwrap_or("UNKNOWN").to_string(),
                message: msg.into(),
            }),
            meta: Some(ApiMeta {
                request_id: uuid::Uuid::now_v7().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }),
        }
    }

    pub fn error_from(err: AppError) -> Self {
        let code_str = match &err {
            AppError::Validation(_) => "VALIDATION",
            AppError::NotFound(_) => "NOT_FOUND",
            AppError::Auth(_) => "AUTH",
            AppError::Internal(_) => "INTERNAL",
        };
        Self {
            success: false,
            data: None,
            error: Some(ApiError {
                code: code_str.to_string(),
                message: err.to_string(),
            }),
            meta: Some(ApiMeta {
                request_id: uuid::Uuid::now_v7().to_string(),
                timestamp: chrono::Utc::now().timestamp_millis(),
            }),
        }
    }
}
```

## Handler Usage

```rust
async fn get_order(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<Json<ApiResponse<OrderDto>>, (StatusCode, Json<ApiResponse<()>>)> {
    let order = state.order_svc
        .find_by_id(id)
        .await
        .map_err(|e| {
            let code = ErrorCode::NOT_FOUND_001 { resource: "order".into() };
            (code.status_code(), Json(ApiResponse::error(code, e.to_string())))
        })?;
    Ok(Json(ApiResponse::success(order)))
}
```
