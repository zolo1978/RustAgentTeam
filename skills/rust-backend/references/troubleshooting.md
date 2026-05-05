# Troubleshooting Guide

## Mobile Build Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `xcodebuild` linker error | Missing iOS target | `rustup target add aarch64-apple-ios` |
| Android NDK version mismatch | NDK r25+ required | Set `NDK_HOME` env var; verify AGP version in `android/gradle/wrapper/gradle-wrapper.properties` |
| `cargo tauri ios build` timeout | Xcode signing config | Check Signing & Capabilities -> Team selected correctly |

## CI Cache Strategy

```yaml
- name: Cache Rust
  uses: Swatinem/rust-cache@v2
  with:
    workspaces: "src-tauri -> target"
    cache-on-failure: true
```

## Update Signing Verification Failed

1. Check `TAURI_SIGNING_PRIVATE_KEY` matches `pubkey` in `tauri.conf.json`
2. Confirm `createUpdaterArtifacts: "v2Compatible"` is set correctly
3. Regenerate signing key pair: `cargo tauri signer generate -w ~/.tauri/myapp.key`

## Linux Signing Notes

Linux desktop apps have no system-level code signing requirement (unlike macOS/Windows). AppImage and .deb packages do not require notarization. For optional GPG signature verification:

```bash
gpg --default-key YOUR_KEY_ID -detach-sign target/release/bundle/appimage/*.AppImage
```
