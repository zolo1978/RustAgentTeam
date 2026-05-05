# BAD/GOOD API Design Comparison

## 1: God Command vs Single Responsibility

```rust
// BAD - one command does everything
async fn manage_user(action: String, data: Value, ...) -> Result<Value, String> { match action.as_str() { "create" => ... } }

// GOOD - one command, one job
#[tauri::command] async fn create_user(req: CreateUserReq, ...) -> Result<UserDto, AppError> { ... }
#[tauri::command] async fn list_users(query: ListQuery, ...) -> Result<Vec<UserDto>, AppError> { ... }
```

## 2: Internal Type vs DTO

```rust
// BAD - database model returned directly (includes password_hash)
async fn get_user(id: Uuid, pool: &PgPool) -> Result<User, AppError> { ... }

// GOOD - internal model -> DTO conversion
#[derive(Serialize)] pub struct UserDto { pub id: Uuid, pub name: String, pub email: String }
impl From<User> for UserDto { fn from(u: User) -> Self { Self { id: u.id, name: u.name, email: u.email } } }
```

## 3: No Pagination vs Cursor

```rust
// BAD - return all at once
async fn list_orders(...) -> Result<Vec<Order>, AppError> { ... }

// GOOD - cursor pagination
#[derive(Serialize)] pub struct PageResp<T> { pub items: Vec<T>, pub next_cursor: Option<String>, pub has_more: bool }
#[tauri::command]
async fn list_orders(req: PageReq, state: State<'_, AppState>) -> Result<PageResp<OrderDto>, AppError> {
    let limit = req.limit.unwrap_or(50).min(200);
    let items = state.order_svc.list(req.cursor.as_deref(), limit + 1).await?;
    let has_more = items.len() > limit;
    let items: Vec<OrderDto> = items.into_iter().take(limit).map(Into::into).collect();
        let next_cursor = if has_more {
        items.last().map(|o| BASE64.encode(format!("{}:{}", o.created_at.timestamp_millis(), o.id)))
    } else {
        None
    };
    Ok(PageResp { items, next_cursor, has_more })
}
```
