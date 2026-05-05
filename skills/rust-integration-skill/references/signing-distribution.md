# 签名与分发

## 概述

桌面应用需要在目标平台签名后才能正常分发。macOS 必须签名+公证，Windows 推荐签名。

## macOS 签名和公证

### 1. 准备

```bash
# 需要 Apple Developer 账号
# 需要 Developer ID Application 证书
security find-identity -v -p codesigning
```

### 2. codesign

```bash
# 签名 .app
codesign --force --deep --sign "Developer ID Application: Your Name (TEAM_ID)" \
  --options runtime \
  --entitlements entitlements.plist \
  target/release/bundle/macos/ClipVault.app
```

### 3. notarize

```bash
# 创建 DMG
hdiutil create -volname ClipVault -srcfolder target/release/bundle/macos/ClipVault.app \
  -ov -format UDZO ClipVault.dmg

# 提交公证
xcrun notarytool submit ClipVault.dmg \
  --apple-id "your@email.com" \
  --team-id "TEAM_ID" \
  --password "@keychain:AC_PASSWORD" \
  --wait

# 装订公证票据
xcrun stapler staple ClipVault.dmg
```

### 4. entitlements.plist

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>com.apple.security.cs.allow-unsigned-executable-memory</key>
    <true/>
    <key>com.apple.security.cs.allow-jit</key>
    <true/>
</dict>
</plist>
```

## Windows 签名

### signtool

```powershell
# 需要 EV 代码签名证书或标准证书
signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 ^
  /a target\release\bundle\msi\ClipVault_0.1.0_x64_en-US.msi
```

## Tauri 构建配置

```json
// tauri.conf.json
{
  "bundle": {
    "active": true,
    "targets": "all",
    "icon": [
      "icons/32x32.png",
      "icons/128x128.png",
      "icons/128x128@2x.png",
      "icons/icon.icns",
      "icons/icon.ico"
    ],
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "entitlements": "entitlements.plist"
    },
    "windows": {
      "certificateThumbprint": null,
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

## GitHub Actions CI

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

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
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with: { targets: ${{ matrix.target }} }
      - uses: actions/setup-node@v4
        with: { node-version: 20 }
      - run: npm install
      - run: npm run tauri build -- --target ${{ matrix.target }}
      # macOS signing
      - if: runner.os == 'macOS'
        run: |
          codesign --force --deep --sign "$SIGNING_IDENTITY" ...
        env:
          SIGNING_IDENTITY: ${{ secrets.APPLE_SIGNING_IDENTITY }}
```

## 检测命令

```bash
# macOS 验证签名
codesign --verify --deep --strict ClipVault.app
spctl --assess --type execute ClipVault.app

# Windows 验证签名
Get-AuthenticodeSignature ClipVault.msi | Format-List
```
