# Automated Scanning Integration

## Tool Overview

| Tool | Purpose | Install |
|------|---------|---------|
| `cargo audit` | Known vulnerability database (RustSec Advisory DB) | `cargo install cargo-audit` |
| `cargo deny` | License compliance, banned crates, duplicate deps | `cargo install cargo-deny` |
| `cargo geiger` | Count unsafe usage across dependency tree | `cargo install cargo-geiger` |
| `cargo fuzz` | Coverage-guided fuzzing (libFuzzer) | `cargo install cargo-fuzz` |

## Commands

```bash
# Dependency vulnerability audit
cargo audit

# Unsafe usage audit — review every unsafe block in deps
cargo geiger --all-features | grep "Unsafe"

# License policy check (requires deny.toml in project root)
cargo deny check licenses bans sources

# Fuzzing target (requires nightly)
cargo fuzz run parse_input -- -max_total_time=300
```

## CI Integration (GitHub Actions)

```yaml
# .github/workflows/security.yml
name: Security Audit
on: [push, pull_request]
jobs:
  security:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - name: Install tools
        run: cargo install cargo-audit cargo-deny
      - name: Audit dependencies
        run: cargo audit
      - name: License & ban check
        run: cargo deny check
```

## Handling Findings

### cargo audit finds vulnerabilities

1. Check if the advisory affects your code path (not all vulns are exploitable).
2. If exploitable: `cargo update -p <crate>` to patch, or pin a fixed version in `Cargo.toml`.
3. If no fix available yet: add to `[advisories]` in `deny.toml` with `ignore` and a TODO comment with the advisory URL and expiry date. Re-evaluate weekly.

### cargo deny fails

1. **License failure**: Add the license to `allow` in `deny.toml` only if it is compatible with your project license. Do not blindly allow.
2. **Banned crate**: If a banned crate appears, replace it with an approved alternative. If unavoidable, document the exception in `deny.toml`.
3. **Duplicate dependency**: `cargo tree -d <crate>` to identify which parent pulls the duplicate. File an upstream issue or pin to a single version.

### cargo geiger shows unexpected unsafe

1. `cargo geiger --all-features` to see the full table.
2. For each unsafe line in a dependency, check the crate's issue tracker for known soundness bugs.
3. If a core dependency has excessive or unjustified unsafe usage, consider alternatives and document the risk.

### cargo fuzz crashes

1. The fuzzer writes a test case to `fuzz/artifacts/parse_input/crash-<hash>`.
2. Reproduce locally: `cargo fuzz run parse_input fuzz/artifacts/parse_input/crash-<hash>`.
3. Fix the underlying bug, add a regression test with the crash input, then re-run the fuzzer to confirm the fix.
