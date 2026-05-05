// src/config.rs — AppConfig with .env + Build Profiles
// Usage: Copy into your Tauri v2 project's src/ directory
//
// Environment variables:
//   Required in production: API_BASE_URL, SENTRY_DSN
//   Optional: LOG_LEVEL (defaults: "warn" in production, "debug" otherwise)
//             TAURI_ENV  (default: "development")

use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_base: String,
    pub sentry_dsn: Option<String>,
    pub log_level: String,
}

impl AppConfig {
    /// Load config from environment, falling back to safe defaults per profile.
    /// Returns an error if production-required vars are missing.
    pub fn load() -> Result<Self, String> {
        let profile = env::var("TAURI_ENV").unwrap_or_else(|_| "development".into());
        let config = Self {
            api_base: env::var("API_BASE_URL")
                .unwrap_or_else(|_| match profile.as_str() {
                    "production" => "https://api.example.com".into(),
                    "staging"    => "https://staging-api.example.com".into(),
                    _           => "http://localhost:8080".into(),
                }),
            sentry_dsn: env::var("SENTRY_DSN").ok(),
            log_level: env::var("LOG_LEVEL")
                .unwrap_or_else(|_| match profile.as_str() {
                    "production" => "warn".into(),
                    _           => "debug".into(),
                }),
        };
        config.validate(&profile)?;
        Ok(config)
    }

    /// Validate production-required environment variables.
    /// Returns an error if production-critical vars are missing.
    fn validate(&self, profile: &str) -> Result<(), String> {
        if profile == "production" {
            if env::var("API_BASE_URL").is_err() {
                return Err("API_BASE_URL must be set in production. Refusing to start with fallback URL.".into());
            }
            if self.sentry_dsn.is_none() {
                eprintln!("[config] WARNING: SENTRY_DSN not set — crash reporting disabled");
            }
        }
        Ok(())
    }
}
