# Tauri v2 Security Specifics

## Capabilities ACL (src-tauri/capabilities/default.json)

```json
{
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    "fs:allow-read-text-file",
    "shell:allow-execute"
  ]
}
```
Rule: Only list permissions your app actually needs. Never use `core:all`.

> **High-risk permission**: `shell:allow-execute` allows executing arbitrary system commands. Use only when calling specific system tools, must be paired with a path whitelist, and must never be combined with global `fs:allow-*` write permissions.

## Isolation Pattern (tauri.conf.json)

```json
{
  "app": {
    "security": {
      "csp": "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'",
      "freezePrototype": true,
      "dangerousDisableAssetCspModification": false,
      "capabilities": ["default"]
    }
  }
}
```
- `freezePrototype`: prevents prototype pollution on WebView globals.
- CSP must restrict to `'self'`; never use `'unsafe-eval'`.

## Command Input Validation

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Deserialize, Validate)]
struct CreateFileReq {
    #[validate(length(min = 1, max = 255))]
    name: String,
    #[validate(length(max = 10_000_000))]
    content: String,
}

#[tauri::command]
fn create_file(req: CreateFileReq) -> Result<(), AppError> {
    req.validate()?;
    // ... safe logic ...
    Ok(())
}
```

## Mobile Security (iOS/Android)

### Keychain / Keystore

Never store sensitive data (tokens, keys) in SharedPreferences (Android) or UserDefaults (iOS). Use platform keystore:

```rust
// Use tauri-plugin-authenticator for secure credential storage
// iOS: Keychain Services | Android: Android Keystore + EncryptedSharedPreferences
use tauri_plugin_authenticator::AuthenticatorExt;

#[tauri::command]
async fn store_token(token: String, app: AppHandle) -> Result<(), AppError> {
    let auth = app.authenticator();
    auth.store("api_token", token.as_bytes()).await?;
    Ok(())
}
```

### Biometric Authentication

Require biometric before sensitive operations:

```rust
// tauri-plugin-biometric
use tauri_plugin_biometric::BiometricExt;

#[tauri::command]
async fn delete_account(app: AppHandle) -> Result<(), AppError> {
    let bio = app.biometric();
    bio.authenticate("Confirm account deletion".into(), None).await?;
    // proceed with deletion
    Ok(())
}
```

### Jailbreak / Root Detection

```rust
// tauri-plugin-root-detection (or manual check)
fn is_device_compromised() -> bool {
    #[cfg(target_os = "ios")]
    { std::path::Path::new("/Applications/Cydia.app").exists() }
    #[cfg(target_os = "android")]
    { std::path::Path::new("/system/app/Superuser.apk").exists() ||
      std::path::Path::new("/sbin/su").exists() }
    #[cfg(not(any(target_os = "ios", target_os = "android")))]
    { false }
}
```

On compromised devices: refuse sensitive operations or warn the user.

### Certificate Pinning

Mobile apps are vulnerable to MITM on public WiFi. Pin API server certificate:

```rust
// reqwest certificate pinning
let cert = reqwest::Certificate::from_pem(include_bytes!("../../certs/api-server.pem"))?;
let client = reqwest::ClientBuilder::new()
    .add_root_certificate(cert)
    .build()?;
```

### App Store Security Requirements

| Platform | Requirement |
|----------|-------------|
| iOS App Store | ATS (App Transport Security) enforces TLS 1.2+, no HTTP exceptions |
| Google Play | SafetyNet/Play Integrity API for device attestation (required for sensitive apps) |
| Both | Encrypted backup exclusion for sensitive data (`NSURLIsExcludedFromBackupKey` on iOS) |
```
