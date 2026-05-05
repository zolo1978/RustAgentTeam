# cargo-fuzz Setup and Operations

## Fuzz Target Template

```rust
// fuzz/fuzz_targets/parse_input.rs
#![no_main]
use libfuzzer_sys::fuzz_target;
use my_app::parser;

fuzz_target!(|data: &[u8]| {
    let _ = std::panic::catch_unwind(|| {
        if let Ok(s) = std::str::from_utf8(data) {
            let _ = parser::parse(s);
        }
    });
});
```

## Setup

```bash
# Install (requires nightly for libFuzzer)
rustup install nightly
cargo install cargo-fuzz

# Initialize fuzz directory
cargo fuzz init

# Add a new fuzz target
cargo fuzz add parse_input
```

## Running

```bash
# Basic run — 5 minutes
cargo +nightly fuzz run parse_input -- -max_total_time=300

# Run with memory sanitizer (detect use-after-free, buffer overflow)
cargo +nightly fuzz run parse_input --sanitizer=address -- -max_total_time=300

# Reproduce a crash
cargo +nightly fuzz run parse_input fuzz/artifacts/parse_input/crash-<hash>

# Minimize a crash input (reduce to smallest reproducer)
cargo +nightly fuzz cmin parse_input
```

## Corpus Management

```bash
# Seed corpus — place representative inputs in fuzz/corpus/parse_input/
# Good seeds: valid inputs, edge cases, empty, max-length

# Minimize corpus (remove redundant inputs that exercise same code paths)
cargo +nightly fuzz cmin parse_input
```

Coverage target: fuzzing should exercise >=80% of the target function's branches. Check with:

```bash
cargo +nightly fuzz coverage parse_input
# Then inspect with llvm-cov or similar
```

## Crash Triage

When a crash is found:

1. **Reproduce**: `cargo fuzz run parse_input fuzz/artifacts/parse_input/crash-<hash>`
2. **Minimize**: `cargo fuzz tmin parse_input fuzz/artifacts/parse_input/crash-<hash>`
3. **Classify**:
   - **Panic** (unwrap, assertion) — usually LOW/MEDIUM, input validation gap
   - **Memory error** (ASAN: heap-buffer-overflow, use-after-free) — HIGH, potential security exploit
   - **Timeout** — LOW, DoS vector, add input size/time limits
4. **Fix**: Write regression test with the crash input, fix the underlying bug
5. **Verify**: Re-run fuzzer to confirm no regression

## CI Integration (Long-Running Fuzz)

```yaml
# Fuzz for 10 minutes on CI (scheduled or on main branch)
- name: Fuzz
  run: |
    cargo +nightly fuzz run parse_input -- -max_total_time=600 -jobs=2 -workers=2
```

For continuous fuzzing, consider [oss-fuzz](https://google.github.io/oss-fuzz/) or [cargo-fuzz-Continuous](https://github.com/nickwilliams-cpi/cargo-fuzz-action).

## Handling Findings

See [templates/finding.md](../templates/finding.md) for tracking and [templates/incident-response.md](../templates/incident-response.md) for response SLA.
