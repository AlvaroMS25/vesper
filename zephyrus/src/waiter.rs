use std::{future::Future, task::{Context, Poll}};
use std::pin::Pin;
use tokio::sync::oneshot::{Sender, Receiver, channel};
use twilight_model::application::interaction::message_component::MessageComponentInteractionData;
use crate::framework::Framework;

type CheckFn<T> = for<'a> fn(&'a Framework<T>, &'a MessageComponentInteractionData) -> bool;

pub(crate) fn new_pair<T>(fun: CheckFn<T>) -> (ComponentWaiterWaker<T>, ComponentWaiter) {
    let (sender, receiver) = channel();

    (
        ComponentWaiterWaker {
            predicate: fun,
            sender
        },
        ComponentWaiter {
            receiver
        }
    )
}

pub struct ComponentWaiter {
    receiver: Receiver<MessageComponentInteractionData>
}

impl Future for ComponentWaiter {
    type Output = Result<MessageComponentInteractionData, Box<dyn std::error::Error + Send + Sync>>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        Pin::new(&mut self.receiver).poll(cx)
            .map_err(|e| {
                Box::new(e) as Box<_>
            })
    }
}

pub struct ComponentWaiterWaker<T> {
    pub predicate: CheckFn<T>,
    pub sender: Sender<MessageComponentInteractionData>
}

impl<T> ComponentWaiterWaker<T> {
    pub fn check(&self, framework: &Framework<T>, msg: &MessageComponentInteractionData) -> bool {
        (self.predicate)(framework, msg)
    }

    pub fn wake(self, interaction: MessageComponentInteractionData) {
        let _ = self.sender.send(interaction);
    }
}
