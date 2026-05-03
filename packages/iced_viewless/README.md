# iced_viewless

[![Crates.io](https://img.shields.io/crates/v/iced_viewless.svg)](https://crates.io/crates/iced_viewless)
[![Documentation](https://docs.rs/iced_viewless/badge.svg)](https://docs.rs/iced_viewless)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A viewless application runtime for [iced](https://iced.rs) that supports native platforms as well as WebAssembly, enabling integration into other UI frameworks, terminal applications, background services and daemon processes.

## Usage

```rust
use iced_viewless::{application, event_loop::Exit, Subscription, Task};

#[derive(Default)]
struct MyState {
    done: bool,
}

#[derive(Debug, Clone)]
enum Message {
    Tick,
}

fn update(state: &mut MyState, message: Message) -> Task<Message> {
    match message {
        Message::Tick => state.done = true,
    }
    Task::none()
}

fn exit_on(state: &MyState) -> Subscription<Exit> {
    if state.done {
        Subscription::run(|| iced_futures::stream::channel(1, |mut tx| async move {
            let _ = tx.send(Exit).await;
        }))
    } else {
        Subscription::none()
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    application(update)
        .exit_on(exit_on)
        .run()?;
    Ok(())
}
```

### Providing Initial State

Use `run_with` when the initial state doesn't implement `Default` or requires a startup task:

```rust
application(update)
    .exit_on(exit_on)
    .run_with(|| (MyState { done: false }, Task::done(Message::Tick)))?;
```

### Custom Subscription

Override subscription logic with the `.subscription()` builder method:

```rust
application(update)
    .subscription(|state| my_subscription(state))
    .exit_on(exit_on)
    .run()?;
```

### Custom Executor

```rust
use iced_futures::backend::tokio::Executor as TokioExecutor;

application(update)
    .executor::<TokioExecutor>()
    .exit_on(exit_on)
    .run()?;
```

### Within an Async Context

Use `async_application` when running inside an existing async runtime (e.g. as part of a larger tokio application):

```rust
use iced_viewless::viewless::async_application;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    async_application(update)
        .exit_on(exit_on)
        .run()
        .await?;
    Ok(())
}
```

## Feature Flags

| Feature | Description |
|---------|-------------|
| `thread-pool` (default) | Thread pool executor |
| `tokio` | Tokio runtime executor |
| `smol` | smol runtime executor |

## License

MIT
