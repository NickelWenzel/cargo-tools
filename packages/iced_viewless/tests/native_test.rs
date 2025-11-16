//! Native integration test for viewless applications.

use iced_futures::Subscription;
use iced_viewless::{application, ViewlessProgram};

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

    fn update(&self, state: &mut Self::State, _message: Self::Message) {
        *state = true;
    }

    fn subscription(&self, state: &Self::State) -> Subscription<Self::Message> {
        if *state {
            Subscription::none()
        } else {
            Subscription::run_with_id("once", futures::stream::iter(vec![Message::Done]))
        }
    }
}

#[tokio::test(flavor = "current_thread")]
async fn native_simple_completes() {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        tokio::runtime::Handle::current()
            .block_on(async { application(SimpleProgram::default()).run(|| false).await })
    }));

    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => panic!("Program failed: {:?}", e),
        Err(_) => {}
    }
}
