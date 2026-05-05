# WebSocket Real-time API Reference

## Axum WebSocket Handler

```rust
pub async fn ws_handler(
    ws: WebSocketUpgrade,
    State(s): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, s))
}

async fn handle_socket(mut socket: WebSocket, state: Arc<AppState>) {
    let mut rx = state.event_bus.subscribe();
    loop {
        tokio::select! {
            msg = socket.recv() => match msg {
                Some(Ok(Message::Text(t))) => { /* handle */ }
                _ => break,
            },
            event = rx.recv() => {
                if let Ok(json) = serde_json::to_string(&event) {
                    let _ = socket.send(Message::Text(json)).await;
                }
            }
        }
    }
}
```

## Tauri Alternative

Tauri desktop does not need WebSocket. Use `app.emit("event", payload)` on the backend + frontend `listen()` instead:

```rust
// Backend: emit event
app.emit("order-created", &order_dto)?;

// Frontend (JavaScript): listen
import { listen } from '@tauri-apps/api/event';
const unlisten = await listen('order-created', (event) => {
    console.log(event.payload);
});
```

## Heartbeat (Connection Health)

Server sends periodic pings; client must respond with pong. Drop connection on timeout. Add to the `tokio::select!` loop in `handle_socket`:

```rust
_ = interval.tick() => {
    if socket.send(Message::Ping(vec![])).await.is_err() { break; }
}
```

Frontend: use browser WebSocket `onclose` to detect drops. Reconnect with exponential backoff (1s, 2s, 4s, max 30s). Reset on successful connect.

## Authentication on Connect

Do NOT send JWT in URL query params (logged by proxies). Send token as the first message after upgrade:

```rust
// Server: first message must be auth
let Some(Ok(Message::Text(raw))) = socket.recv().await else { return; };
let AuthMsg { token } = serde_json::from_str(&raw)?;
let claims = verify_jwt(&token)?;
// proceed with authenticated socket
```

Frontend sends `JSON.stringify({ token })` immediately after `ws.onopen`.
