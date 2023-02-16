use std::{future::Future, task::{Context, Poll}};
use std::pin::Pin;
use tokio::sync::oneshot::{Sender, Receiver, channel, error::RecvError};
use crate::{twilight_exports::Interaction};

pub(crate) fn new_pair<F>(fun: F) -> (WaiterWaker, InteractionWaiter)
where
    F: Fn(&Interaction) -> bool + Send + 'static
{
    let (sender, receiver) = channel();

    (
        WaiterWaker {
            predicate: Box::new(fun),
            sender
        },
        InteractionWaiter {
            receiver
        }
    )
}

/// A waiter used to wait for an interaction.
///
/// The waiter implements [`Future`], so in order to retrieve the interaction, just await the waiter.
///
/// # Examples:
///
/// ```rust
/// use zephyrus::prelude::{command, SlashContext, DefaultCommandResult};
///
/// #[command]
/// #[description = "My Command"]
/// async fn my_command(ctx: &SlashContext<()>) -> DefaultCommandResult {
///     ctx.acknowledge().await?;
///     let interaction = ctx.wait_interaction(|interaction| {
///         // predicate here
///     }).await?;
///
///     Ok(())
/// }
/// ```
///
/// [`Future`]: Future
pub struct InteractionWaiter {
    receiver: Receiver<Interaction>
}

impl Future for InteractionWaiter {
    type Output = Result<Interaction, RecvError>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx)
    }
}


/// A waker used to notify its associate [`waiter`] when the predicate has been satisfied and
/// deliver the interaction.
///
/// [`waiter`]: InteractionWaiter
pub struct WaiterWaker {
    pub predicate: Box<dyn Fn(&Interaction) -> bool + Send + 'static>,
    pub sender: Sender<Interaction>
}

impl WaiterWaker {
    pub fn check(&self, interaction: &Interaction) -> bool {
        (self.predicate)(interaction)
    }

    pub fn wake(self, interaction: Interaction) {
        let _ = self.sender.send(interaction);
    }
}
