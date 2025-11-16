# iced_viewless

A headless application runtime for [iced](https://iced.rs), enabling background services and daemon processes without UI or windowing.

## Features

- **Headless execution**: Run iced applications without windows or rendering
- **Subscription-based**: Event-driven architecture using iced's subscription system
- **Cross-platform**: Works on native platforms (Linux, macOS, Windows) and WebAssembly
- **Flexible executors**: Support for tokio, async-std, smol, and WASM runtimes
- **Simple API**: Familiar interface similar to `iced::daemon`

## Usage

```rust
use iced_futures::Subscription;
use iced_viewless::{viewless, ViewlessProgram};

#[derive(Debug, Clone)]
enum Message {
    Event,
}

#[derive(Default)]
struct MyProgram;

impl ViewlessProgram for MyProgram {
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
    viewless::<MyProgram>().run().await?;
    Ok(())
}
```

## Features

- `tokio`: Use tokio runtime (enables timeout support)
- `async-std`: Use async-std runtime
- `smol`: Use smol runtime
- `thread-pool`: Use thread pool executor

## Testing

Run native tests:
```bash
cargo test -p iced_viewless --features tokio
```

Build for WASM:
```bash
cargo build -p iced_viewless --target wasm32-unknown-unknown
```

## How It Works

The `iced_viewless` runtime:

1. Boots the program to get initial state
2. Spawns subscriptions as async tasks
3. Processes messages as they arrive
4. Updates program state
5. Exits when `subscription()` returns `Subscription::none()`

Unlike windowing applications, viewless programs are purely event-driven through subscriptions. When your program returns `Subscription::none()`, the runtime detects no active subscriptions and exits cleanly.

## License

MIT
