# Tauri v2 Build Optimizations

## Release Profile (Cargo.toml)

```toml
[profile.release]
opt-level = "z"        # Optimize for size
lto = true             # Link-Time Optimization
codegen-units = 1      # Better optimization, slower build
strip = true           # Strip debug symbols
panic = "abort"        # Smaller binary, no unwind tables
trim-paths = ["diagnostics"]  # Tauri v2: strip source paths from diagnostics
```

## mobile_entry_point Pattern

```rust
// src/lib.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Binary Size Audit

```bash
cargo install cargo-bloat
cargo bloat --release --target aarch64-apple-darwin --crates
# Identify large crates; consider alternatives for top offenders
```

## Artifact Management & Provenance

```yaml
# CI: tag commit SHA to every artifact
- name: Archive Artifacts
  run: |
    sha256sum target/release/bundle/*/* > checksums.txt
    echo "Build: ${GITHUB_SHA}" >> metadata.txt
    echo "Time: $(date -u +%Y-%m-%dT%H:%M:%SZ)" >> metadata.txt
- uses: actions/upload-artifact@v4
  with:
    name: release-${{ github.ref_name }}
    path: |
      target/release/bundle/
      checksums.txt
      metadata.txt
```

Sign every artifact. Store signatures alongside binaries. Verify before deployment.
