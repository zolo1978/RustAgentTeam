// Template: JWT Authentication — Claims + AuthUser FromRequestParts
// Copy-paste ready. Adapt claim fields and error handling to your domain.

use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use jsonwebtoken::{decode, DecodingKey, Validation, encode, EncodingKey};
use serde::{Serialize, Deserialize};
use std::sync::Arc;

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,  // user ID
    pub role: String, // user role
    pub exp: usize,   // expiration (Unix timestamp)
}

pub struct AuthUser {
    pub id: String,
    pub role: String,
}

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        let auth = parts
            .headers
            .get("authorization")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.strip_prefix("Bearer "))
            .ok_or(AppError::Auth("missing token".into()))?;

        let tok = decode::<Claims>(
            auth,
            &DecodingKey::from_secret(&state.config.jwt_secret),
            &Validation::default(),
        )
        .map_err(|_| AppError::Auth("invalid token".into()))?;

        Ok(AuthUser {
            id: tok.claims.sub,
            role: tok.claims.role,
        })
    }
}
