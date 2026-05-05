# Mobile CI/CD Reference

## iOS — GitHub Actions Provisioning

```yaml
ios-build:
  runs-on: macos-latest
  steps:
    - uses: actions/checkout@v4
    - name: Install Apple Certificate
      env:
        P12_BASE64: ${{ secrets.IOS_P12_BASE64 }}
        P12_PASSWORD: ${{ secrets.IOS_P12_PASSWORD }}
      run: |
        echo "$P12_BASE64" | base64 -d > cert.p12
        security create-keychain -p actions build.keychain
        security import cert.p12 -P "$P12_PASSWORD" -k build.keychain -T /usr/bin/codesign
        security list-keychain -s build.keychain
    - name: Build iOS
      env:
        APPLE_ID: ${{ secrets.APPLE_ID }}
        APPLE_PASSWORD: ${{ secrets.APPLE_APP_PASSWORD }}
        APPLE_TEAM_ID: ${{ secrets.APPLE_TEAM_ID }}
      run: |
        cargo tauri ios build --release
        xcrun notarytool submit path/to/app.ipa --apple-id "$APPLE_ID" --password "$APPLE_PASSWORD" --team-id "$APPLE_TEAM_ID" --wait
        # 注意: altool 已在 Xcode 15+ 弃用，推荐 notarytool 或 Transporter
```

## Android — Signing Keystore

```yaml
android-build:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Decode Keystore
      run: echo "${{ secrets.ANDROID_KEYSTORE_BASE64 }}" | base64 -d > android/app/release.keystore
    - name: Build Android
      env:
        KEYSTORE_PATH: release.keystore
        KEYSTORE_PASSWORD: ${{ secrets.ANDROID_KEYSTORE_PASSWORD }}
        KEY_ALIAS: ${{ secrets.ANDROID_KEY_ALIAS }}
        KEY_PASSWORD: ${{ secrets.ANDROID_KEY_PASSWORD }}
      run: cargo tauri android build --release
```

## CI Matrix — Multi-Platform Release

```yaml
jobs:
  release:
    strategy:
      matrix:
        include:
          - platform: macos-latest
            target: aarch64-apple-darwin
          - platform: macos-latest
            target: x86_64-apple-darwin
          - platform: windows-latest
            target: x86_64-pc-windows-msvc
          - platform: ubuntu-22.04
            target: x86_64-unknown-linux-gnu
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - run: cargo tauri build --target ${{ matrix.target }}
```

## Code Signing (All Platforms)

```bash
# macOS: codesign + notarize
codesign --deep --force --sign "Developer ID Application: ..." target/release/bundle/macos/*.app
xcrun notarytool submit target/release/bundle/macos/*.dmg --apple-id "$APPLE_ID" --password "$APPLE_PASS" --team-id "$TEAM_ID" --wait

# Windows: signtool
signtool sign /fd SHA256 /a /tr http://timestamp.digicert.com /td SHA256 target/release/bundle/msi/*.msi

# Linux: GPG (optional, no system-level signing required)
gpg --default-key YOUR_KEY_ID -detach-sign target/release/bundle/appimage/*.AppImage
```
