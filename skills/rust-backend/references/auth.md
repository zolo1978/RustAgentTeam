# Authentication/Authorization Reference

> Copy-paste ready version: [templates/jwt-auth.rs](../templates/jwt-auth.rs)

## Axum JWT (Server-side)

JWT Bearer token extracted via `FromRequestParts`. Key concepts:

1. **Claims** — JWT payload struct with `sub` (user ID), `role`, `exp` (expiration).
2. **AuthUser** — Decoded identity passed into handlers, extracted from `Authorization: Bearer <token>`.
3. **FromRequestParts impl** — Reads header, strips "Bearer ", decodes with `jsonwebtoken::decode`, returns `AuthUser` or `AppError::Auth`.

The `AppState` must expose `config.jwt_secret` for `DecodingKey::from_secret`.

Template: [templates/jwt-auth.rs](../templates/jwt-auth.rs)

### Handler Usage

```rust
async fn get_profile(
    auth: AuthUser,  // extracted automatically
) -> Result<Json<ApiResponse<UserDto>>, AppError> {
    // auth.id and auth.role are available
    let user = state.user_svc.find_by_id(&auth.id).await?;
    Ok(Json(ApiResponse::success(user)))
}
```

## Tauri Desktop (System Keychain)

Tauri desktop does not need JWT. Use `tauri-plugin-authenticator` to leverage the system keychain:

```rust
use tauri_plugin_authenticator::Authenticator;

#[tauri::command]
async fn save_credential(
    key: String,
    value: String,
    auth: State<'_, Authenticator>,
) -> Result<(), AppError> {
    auth.save(&key, &value).map_err(|e| AppError::Auth(e.to_string()))
}

#[tauri::command]
async fn get_credential(
    key: String,
    auth: State<'_, Authenticator>,
) -> Result<Option<String>, AppError> {
    auth.get(&key).map_err(|e| AppError::Auth(e.to_string()))
}
```
