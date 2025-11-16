//! Runtime for viewless applications.

use crate::program::{Instance, ViewlessProgram};
use crate::Result;
use iced_futures::futures::channel::mpsc;
use iced_futures::futures::stream::StreamExt;
use iced_futures::subscription::Tracker;
use iced_futures::Executor;

/// Runs a viewless program until all subscriptions complete.
///
/// This function:
/// 1. Creates a message channel for subscription events
/// 2. Tracks the program's subscriptions
/// 3. Processes messages as they arrive
/// 4. Updates the program state
/// 5. Exits when all subscriptions complete (channel closes)
///
/// # Arguments
/// * `executor` - The executor to spawn subscription tasks
/// * `instance` - The program instance to run
///
/// # Returns
/// `Ok(())` when the runtime completes normally.
pub async fn run<E, P>(executor: E, mut instance: Instance<P>) -> Result<()>
where
    E: Executor,
    P: ViewlessProgram,
{
    let (sender, mut receiver) = mpsc::unbounded::<P::Message>();
    let mut tracker = Tracker::new();

    loop {
        let subscription = instance.subscription();
        let recipes = iced_futures::subscription::into_recipes(subscription);

        let futures = executor.enter(|| tracker.update(recipes.into_iter(), sender.clone()));

        for future in futures {
            executor.spawn(future);
        }

        match receiver.next().await {
            Some(message) => {
                instance.update(message);
            }
            None => {
                break;
            }
        }
    }

    Ok(())
}
