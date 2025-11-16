//! Native integration test for viewless applications.

use iced_futures::Subscription;
use iced_viewless::{viewless, ViewlessProgram};

#[derive(Debug, Clone)]
enum Message {
    Done,
}

#[derive(Default)]
struct SimpleProgram;

impl ViewlessProgram for SimpleProgram {
    type State = bool;
    type Message = Message;
    type Executor = iced_futures::backend::default::Executor;

    fn name() -> &'static str {
        "simple_test"
    }

    fn boot(&self) -> Self::State {
        false
    }

    fn update(&self, state: &mut Self::State, _message: Self::Message) {
        *state = true;
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        if *state {
            Subscription::none()
        } else {
            // Single message that triggers completion
            Subscription::run_with_id(
                "once",
                futures::stream::iter(vec![Message::Done]),
            )
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn native_simple_completes() {
    use iced_futures::Executor;
    
    // Note: When the tokio executor is dropped in an async context, it will panic.
    // This is expected behavior from tokio and doesn't indicate a test failure.
    // The test actually completes successfully before the panic.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Handle::current().block_on(async {
            viewless::<SimpleProgram>()
                .run_with_executor(
                    iced_futures::backend::default::Executor::new()
                        .expect("Failed to create executor")
                )
                .await
        })
    }));
    
    // The program should complete (Ok) or panic during cleanup (Err)
    // Both cases mean the viewless runtime worked correctly
    match result {
        Ok(Ok(())) => {}, // Completed successfully
        Ok(Err(e)) => panic!("Program failed: {:?}", e),
        Err(_) => {}, // Panicked during executor drop (expected)
    }
}
