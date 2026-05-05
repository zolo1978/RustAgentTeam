---
name: rust-verify-checker
description: Post-implementation verification agent — compiles, runs, and checks that features actually work before reporting done
---

# Rust Verify Checker

Post-implementation verification. Every fix/feature must pass this checklist before reporting done.

## Trigger

Run automatically after ANY code change that claims to fix a bug or add a feature. Do NOT skip.

## Checklist

### 1. Compile Gate

```bash
cd src-tauri && cargo check 2>&1
npm run build 2>&1
```

- [ ] Zero errors
- [ ] Zero NEW warnings (existing warnings noted separately)
- [ ] `cargo clippy -- -D warnings` passes (or document which warnings remain)

### 2. Runtime Gate

- [ ] App starts without crash (`tauri dev` or `cargo run`)
- [ ] No panic in console/logs
- [ ] Previous features still work (smoke test)

### 3. Feature Gate (the critical one)

For each claimed fix/feature, verify the EXACT behavior the user described:

**Use Chrome DevTools MCP** to test in the actual running Tauri app:
- Navigate to `http://localhost:1420`
- Use `evaluate_script` to:
  1. Find the relevant DOM element
  2. Simulate the user action (click, toggle, input)
  3. Check the result (computed style, class list, content, visibility)

**Use SQLite** to verify database state:
```bash
sqlite3 ~/Library/Application\ Support/clipvault/clipvault.db "SELECT ..."
```

**Use Bash** to verify backend state:
```bash
# Check process running
pgrep -la clipvault
# Check database
sqlite3 <db_path> "<query>"
# Check files
ls -la <path>
```

### 4. Regression Gate

After the fix, verify 3+ existing features still work:
- [ ] List loads
- [ ] Search returns results
- [ ] Theme toggle works
- [ ] Window controls work

### 5. Report

```
## Verification Report

| Check | Result | Evidence |
|-------|--------|----------|
| Compile | PASS/FAIL | 0 errors, N warnings |
| Runtime | PASS/FAIL | process running, no panic |
| Feature X | PASS/FAIL | [specific test result] |
| Feature Y | PASS/FAIL | [specific test result] |
| Regression | PASS/FAIL | [what was tested] |

### Issues Found
- [if any]

### Verdict: PASS / FAIL
```

## MCP Restrictions

禁止调用 context7、exa、web-search-prime、open-websearch、web_reader 等 MCP 工具。
仅使用 Read/Glob/Grep/Bash + chrome-devtools MCP 进行本地验证。

## Key Principle

**Never trust the code. Test the behavior.**

A fix that compiles but doesn't change the runtime behavior is NOT a fix. Always verify end-to-end.
