use crate::twilight_exports::Message as TwilightMessage;
use crate::{context::SlashContext, waiter::WaiterReceiver};

/// A wrapper around twilight's [message](TwilightMessage)
/// adding a few convenience methods.
pub struct Message<'a, T> {
    inner: TwilightMessage,
    context: &'a SlashContext<'a, T>,
}

impl<T> std::ops::Deref for Message<'_, T> {
    type Target = TwilightMessage;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> std::ops::DerefMut for Message<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<'a, T> Message<'a, T> {
    /// Creates a new [message](self::Message).
    pub(crate) fn new(context: &'a SlashContext<'a, T>, msg: TwilightMessage) -> Self {
        Self {
            inner: msg,
            context,
        }
    }
    /// Waits for a component interaction, this method is a shortcut to
    /// [wait_component](SlashContext::wait_component).
    pub fn wait_component(&self) -> WaiterReceiver {
        let id = self.inner.id;
        self.context.wait_component(move |f| f.message.id == id)
    }
}
