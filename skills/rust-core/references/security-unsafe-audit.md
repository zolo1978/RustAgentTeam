# `unsafe` Rust Audit & Concurrency Security

## unsafe Block Audit Checklist

Every `unsafe` block in the project must satisfy:

- [ ] **Safety comment present** — `// SAFETY:` prefix explaining why the operation is sound
- [ ] **Invariants documented** — Pre-conditions that callers/maintainers must uphold
- [ ] **Scope minimized** — `unsafe` block covers only the operation that requires it, not surrounding safe code
- [ ] **No escape of unsafe values** — Raw pointers, `MaybeUninit`, `UnsafeCell` references do not leak into safe API

### BAD/GOOD: Safety Comments

```rust
// BAD — no safety justification
unsafe { *ptr.add(offset) }

// GOOD — invariant stated, verifiable
// SAFETY: offset was validated < slice.len() on line 42, ptr is slice.as_ptr()
// which is valid for slice.len() elements.
unsafe { *ptr.add(offset) }
```

## Common Unsound Patterns

### 1. `transmute` Type Confusion

```rust
// BAD — transmuting between unrelated types is instant UB
let bits: u64 = unsafe { std::mem::transmute(3.14_f64) };

// GOOD — use dedicated conversion methods
let bits: u64 = 3.14_f64.to_bits();
```

### 2. Out-of-Bounds Raw Pointer Access

```rust
// BAD — no bounds check
unsafe { *ptr.add(idx) }

// GOOD — validate before access
assert!(idx < len, "index {idx} out of bounds for len {len}");
// SAFETY: idx < len verified above; ptr is valid for len elements.
unsafe { *ptr.add(idx) }
```

### 3. Incorrect `Send`/`Sync` Implementation

```rust
// BAD — blanket impl without verifying thread safety
unsafe impl Send for MyType {}
unsafe impl Sync for MyType {}

// GOOD — only impl if all fields are Send/Sync, or internal synchronization
// is verified. Document the reasoning.
// SAFETY: MyType only contains Arc<Mutex<Data>> (Send + Sync).
unsafe impl Send for MyType {}
unsafe impl Sync for MyType {}
```

### 4. `MaybeUninit` Uninitialized Read

```rust
// BAD — reading uninitialized memory is UB
let mut buf: [u8; 1024] = std::mem::MaybeUninit::uninit().assume_init();

// GOOD — initialize or use MaybeUninit properly
let mut buf = [0u8; 1024];
// or
let mut buf: std::mem::MaybeUninit<[u8; 1024]> = std::mem::MaybeUninit::zeroed();
let buf = buf.assume_init(); // SAFE: zeroed produces a valid [u8; N]
```

## Concurrency Security Review

### `Send` / `Sync` Violations

Rust's type system prevents data races for safe code. `unsafe impl Send/Sync` bypasses this — audit each one.

| Pattern | Risk | Audit Check |
|---------|------|-------------|
| `unsafe impl Send for T` | Data race if T has non-Send interior | Verify all fields are Send, or mutex-guarded |
| `unsafe impl Sync for T` | Data race via shared `&T` | Verify all `&self` methods are thread-safe |
| `Rc<T>` shared across threads | Compile-time error in safe Rust; possible with unsafe | grep for `Rc` in `spawn`/`thread::spawn` closures |
| `RefCell<T>` in multi-threaded context | Runtime panic or UB with unsafe | grep for `RefCell` near `Arc` or `spawn` |

### Mutex / RwLock Audit

```rust
// BAD — lock held across await point (causes deadlock with single-threaded executor)
async fn bad(db: &Mutex<Connection>) {
    let conn = db.lock().unwrap();
    some_async_fn(&*conn).await; // lock held across .await!
}

// GOOD — scope the lock, don't hold across await
async fn good(db: &Mutex<Connection>) -> Result<Data> {
    let data = {
        let conn = db.lock().map_err(|e| AppError::Internal(e.to_string()))?;
        conn.query_row("SELECT ...", [], |row| row.get(0))?
    };
    some_async_fn(&data).await // lock released before await
}
```

### Channel Boundary Audit

- Bounded channels (`sync_channel(N)`) enforce backpressure — prefer for production
- Unbounded channels (`channel()`) can cause OOM if producer outpaces consumer
- Audit: grep for `mpsc::channel()` and verify consumer keeps pace or producers are bounded

## Audit Command

```bash
# Find all unsafe blocks and impls in project source
grep -rn "unsafe" src/ --include="*.rs" | grep -v "test\|// SAFETY\|#\[deny(unsafe"

# Find manual Send/Sync impls
grep -rn "unsafe impl Send\|unsafe impl Sync" src/ --include="*.rs"

# Check for RefCell in multi-threaded contexts
grep -rn "RefCell" src/ --include="*.rs" -A2 | grep "Arc\|thread\|spawn"
```
