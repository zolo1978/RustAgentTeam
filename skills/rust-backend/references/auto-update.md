# Tauri Updater — Complete Implementation

## tauri.conf.json

```json
{
  "bundle": {
    "createUpdaterArtifacts": "v2Compatible"
  },
  "plugins": {
    "updater": {
      "pubkey": "PUBLIC_KEY_FROM_GENERATION",
      "endpoints": [
        "https://releases.example.com/updates/{{target}}/{{arch}}/{{current_version}}"
      ]
    }
  }
}
```

## Generate Signing Key

```bash
cargo tauri signer generate -w ~/.tauri/myapp.key
# Sets TAURI_SIGNING_PRIVATE_KEY env var in CI
# Public key goes into tauri.conf.json pubkey field
```

## Frontend Update Check (TypeScript)

→ 前端更新检查实现见 `rust-frontend` Skill 的 `templates/check-for-update.ts`。

## Server-Side Update JSON Format

```json
{
  "version": "2.1.0",
  "notes": "Bug fixes and performance improvements",
  "pub_date": "2026-04-26T12:00:00Z",
  "platforms": {
    "darwin-aarch64": {
      "signature": "SIGNATURE_FROM_BUILD",
      "url": "https://releases.example.com/myapp_2.1.0_aarch64.app.tar.gz"
    },
    "windows-x86_64": {
      "signature": "SIGNATURE_FROM_BUILD",
      "url": "https://releases.example.com/myapp_2.1.0_x64-setup.nsis.zip"
    },
    "linux-x86_64": {
      "signature": "SIGNATURE_FROM_BUILD",
      "url": "https://releases.example.com/myapp_2.1.0_amd64.AppImage.tar.gz"
    }
  }
}
```

## Staged Canary Rollout (Server-Side Percentage Gate)

```javascript
// Server endpoint returns update only for canary % of clients
// tracked by installation_id hash
const update = await check();
if (!update?.available) return;
if (update.metadata?.rollout_pct) {
  const hash = hashCode(installationId) % 100;
  if (hash > update.metadata.rollout_pct) {
    console.log("Not in canary cohort, skipping");
    return;
  }
}
await update.downloadAndInstall();
```

## Signing Verification Troubleshooting

1. Check `TAURI_SIGNING_PRIVATE_KEY` matches `tauri.conf.json` `pubkey`
2. Confirm `createUpdaterArtifacts: "v2Compatible"` is set
3. Regenerate key pair: `cargo tauri signer generate -w ~/.tauri/myapp.key`
