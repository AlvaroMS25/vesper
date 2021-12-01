use crate::twilight_exports::*;
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::sync::oneshot::{channel, error::RecvError, Receiver, Sender};

/// A receiver waiting to a
/// [component interaction](MessageComponentInteraction).
pub struct WaiterReceiver {
    future: Pin<
        Box<dyn Future<Output = Result<MessageComponentInteraction, RecvError>> + Unpin + Send>,
    >,
}

impl WaiterReceiver {
    /// Creates a new [receiver](self::WaiterReceiver).
    fn new(receiver: Receiver<MessageComponentInteraction>) -> Self {
        Self {
            future: Box::pin(receiver),
        }
    }
}

impl Future for WaiterReceiver {
    type Output = Result<MessageComponentInteraction, RecvError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.get_mut().future.as_mut().poll(cx)
    }
}

/// A sender pointing to a [receiver](self::WaiterReceiver).
pub(crate) struct WaiterSender {
    /// The sender pointing to a [receiver](self::WaiterReceiver).
    sender: Sender<MessageComponentInteraction>,
    /// The predicate which the interaction must to satisfy to wake the receiver
    /// bound with this sender.
    predicate: Box<dyn Fn(&MessageComponentInteraction) -> bool + Send>,
}

impl WaiterSender {
    /// Creates a new [sender](self::WaiterSender).
    fn _new<F>(checker: F) -> (WaiterSender, WaiterReceiver)
    where
        F: Fn(&MessageComponentInteraction) -> bool + Send + 'static,
    {
        let (tx, rx) = channel();
        let receiver = WaiterReceiver::new(rx);
        let sender = Self {
            sender: tx,
            predicate: Box::new(checker),
        };
        (sender, receiver)
    }

    /// Checks if the given [interaction](MessageComponentInteraction)
    /// satisfies the predicate.
    pub fn check(&self, interaction: &MessageComponentInteraction) -> bool {
        (self.predicate)(interaction)
    }

    pub fn new<F>(fun: F) -> (WaiterSender, WaiterReceiver)
    where
        F: Fn(&MessageComponentInteraction) -> bool + Send + 'static,
    {
        Self::_new(fun)
    }

    /// Sends the given
    /// [interaction](MessageComponentInteraction)
    /// through the sender, waking the receiver or doing nothing if it has been dropped.
    pub fn send(self, message: MessageComponentInteraction) {
        let _ = self.sender.send(message);
    }
}
