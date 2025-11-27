# iced_headless

[![Crates.io](https://img.shields.io/crates/v/iced_headless.svg)](https://crates.io/crates/iced_headless)
[![Documentation](https://docs.rs/iced_headless/badge.svg)](https://docs.rs/iced_headless)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A headless application runtime for [iced](https://iced.rs), enabling background services and daemon processes without UI or windowing.

## Features

**Compatible with iced 0.13.1**

- **Headless execution**: Run iced applications without windows or rendering
- **Subscription-based**: Event-driven architecture using iced's subscription system
- **Cross-platform**: Works on native platforms (Linux, macOS, Windows) and WebAssembly
- **Flexible executors**: Support for tokio, async-std, smol, and WASM runtimes
- **Simple API**: Familiar interface similar to `iced::daemon`

## Usage

```rust
use iced_futures::Subscription;
use iced_headless::{application, HeadlessProgram};

#[derive(Debug, Clone)]
enum Message {
    Event,
}

#[derive(Default)]
struct MyProgram;

impl HeadlessProgram for MyProgram {
    type State = ();
    type Message = Message;
    type Executor = iced_futures::backend::default::Executor;

    fn name() -> &'static str {
        "my_program"
    }

    fn boot(&self) -> Self::State {
        ()
    }

    fn update(&self, _state: &mut Self::State, _message: Self::Message) {
        // Handle messages
    }

    fn subscription(&self, _state: &Self::State) -> Subscription<Self::Message> {
        // Return subscriptions or Subscription::none() to exit
        Subscription::none()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    application(MyProgram::default())
        .run(|| ())
        .await?;
    Ok(())
}
```

### Builder Pattern

The API follows iced's builder pattern with decorator methods:

```rust
use iced_futures::{Subscription, Executor};
use iced_headless::{application, HeadlessProgram};

// Custom subscription
application(my_program)
    .subscription(|state| {
        // Custom subscription logic
        Subscription::none()
    })
    .run(|| initial_state)
    .await?;

// Custom executor
application(my_program)
    .executor::<MyExecutor>()
    .run(|| initial_state)
    .await?;

// Combined
application(my_program)
    .subscription(my_subscription)
    .executor::<MyExecutor>()
    .run(|| initial_state)
    .await?;
```

## Features

- `tokio`: Use tokio runtime
- `async-std`: Use async-std runtime
- `smol`: Use smol runtime
- `thread-pool`: Use thread pool executor

## Testing

Run native tests:
```bash
cargo test -p iced_headless --features tokio
```

Build for WASM:
```bash
cargo build -p iced_headless --target wasm32-unknown-unknown
```

**Note**: WASM tests are present but marked as `#[ignore]` due to browser automation limitations in CI environments. The crate compiles successfully for WASM and the API is WASM-compatible.

## How It Works

The `iced_headless` runtime follows iced's architecture:

1. Creates an `Application` builder wrapping your `ViewlessProgram`
2. Calls the boot function to get initial state
3. Spawns subscriptions as async tasks using the executor
4. Processes messages as they arrive through the event loop
5. Updates program state via the `update` method
6. Exits when `subscription()` returns `Subscription::none()`

The API uses the decorator pattern similar to iced:
- `subscription()` wraps the program with custom subscription logic
- `executor()` changes the executor type at compile time

Unlike windowing applications, headless programs are purely event-driven through subscriptions. When your program returns `Subscription::none()`, the runtime detects no active subscriptions and exits cleanly.

## License

MIT
