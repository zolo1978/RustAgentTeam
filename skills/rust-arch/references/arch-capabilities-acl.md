# Capabilities ACL 模型 (v2)

v2 用 Capabilities 替代 v1 allowlist。窗口默认无 IPC 权限，必须显式声明。

## 配置示例

```jsonc
// src-tauri/capabilities/default.json
{
  "identifier": "main-cap",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "dialog:allow-open",
    {
      "identifier": "fs:allow-write-text-file",
      "allow": [{ "path": "$APPDATA/**" }]
    }
  ],
  "platforms": ["macOS", "windows", "linux"]
}
```

## 字段说明

| 字段 | 说明 |
|------|------|
| `identifier` | Capability 唯一标识，用于关联窗口 |
| `windows` | 适用窗口列表，`["main"]` 表示仅主窗口 |
| `permissions` | 权限声明数组，支持字符串或带 scope 的对象 |
| `platforms` | 目标平台过滤：`macOS`、`windows`、`linux`、`iOS`、`android` |

## 权限最小化原则

1. **默认拒绝**：窗口创建后无任何 IPC 权限，必须通过 Capabilities 显式授予。
2. **最小 scope**：文件系统、网络等权限用 `allow`/`deny` 路径模式限制范围（如 `$APPDATA/**`）。
3. **平台隔离**：通过 `platforms` 字段为不同平台配置不同权限集，移动端与桌面端权限分离。
4. **多文件组织**：按功能域拆分多个 JSON 文件（`default.json`、`admin.json`），每个文件对应一组权限。

## Isolation Pattern

安全敏感应用启用 AES-GCM 加密 IPC：

```json
{
  "app": {
    "security": {
      "pattern": {
        "use": "isolation",
        "options": {
          "dir": "../dist-isolation"
        }
      }
    }
  }
}
```

启用后 IPC 消息经 AES-GCM 加密传输，防止 WebView 注入攻击。

## Mobile Notes

- **iOS:** Capabilities must be declared in `src-tauri/capabilities/default.json` — no separate Info.plist needed for IPC permissions.
- **Android:** Permissions map to AndroidManifest.xml entries via Tauri's plugin system.
- **Both:** `#[cfg_attr(mobile, tauri::mobile_entry_point)]` on `run()` is required for mobile entry point registration.
