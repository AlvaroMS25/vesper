use std::{future::Future, task::{Context, Poll}};
use std::pin::Pin;
use tokio::sync::oneshot::{Sender, Receiver, channel};
use crate::{framework::Framework, twilight_exports::Interaction};

type CheckFn<T> = for<'a> fn(&'a Framework<T>, &'a Interaction) -> bool;

pub(crate) fn new_pair<T>(fun: CheckFn<T>) -> (WaiterWaker<T>, InteractionWaiter) {
    let (sender, receiver) = channel();

    (
        WaiterWaker {
            predicate: fun,
            sender
        },
        InteractionWaiter {
            receiver
        }
    )
}

pub struct InteractionWaiter {
    receiver: Receiver<Interaction>
}

impl Future for InteractionWaiter {
    type Output = Result<Interaction, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx)
            .map_err(|e| {
                Box::new(e) as Box<_>
            })
    }
}

pub struct WaiterWaker<T> {
    pub predicate: CheckFn<T>,
    pub sender: Sender<Interaction>
}

impl<T> WaiterWaker<T> {
    pub fn check(&self, framework: &Framework<T>, interaction: &Interaction) -> bool {
        (self.predicate)(framework, interaction)
    }

    pub fn wake(self, interaction: Interaction) {
        let _ = self.sender.send(interaction);
    }
}
