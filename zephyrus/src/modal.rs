use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tokio::sync::oneshot::error::RecvError;
use crate::context::SlashContext;
use crate::wait::InteractionWaiter;
use crate::twilight_exports::{Interaction, InteractionClient, InteractionResponse, InteractionResponseType};
use std::{error::Error as StdError, fmt::{Debug, Display, Formatter, Result as FmtResult}};
use twilight_http::response::marker::EmptyBody;
use twilight_http::response::ResponseFuture;


/// Errors that can be returned when awaiting modals.
#[derive(Debug)]
pub enum ModalError {
    /// An http error occurred.
    Http(twilight_http::Error),
    /// Something failed when using a [waiter](InteractionWaiter)
    Waiter(RecvError)
}

impl Display for ModalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Self::Http(error) => write!(f, "Http error: {}", error),
            Self::Waiter(error) => write!(f, "Waiter error {}", error)
        }
    }
}

impl StdError for ModalError {}

/// The outcome of `.await`ing a [WaitModal](WaitModal).
///
/// This structure provides both the parsed modal and the interaction used to retrieve it.
pub struct ModalOutcome<S> {
    /// The inner parsed modal.
    pub inner: S,
    /// The interaction used to retrieve the modal.
    pub interaction: Interaction
}

impl<S: Debug> Debug for ModalOutcome<S> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        <S as Debug>::fmt(&self.inner, f)
    }
}

impl<S> std::ops::Deref for ModalOutcome<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<S> std::ops::DerefMut for ModalOutcome<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// A waiter used to retrieve the input of a command. This can be obtained by using
/// [SlashContext::create_modal](SlashContext::create_modal).
///
/// To retrieve the input of the modal, `.await` the waiter.
///
/// If the waiter is never awaited, the user won't be able to submit the modal, and will have to
/// close it without submitting.
#[must_use = "Modals cannot be submitted if the waiter is not awaited"]
pub struct WaitModal<'ctx, S> {
    pub(crate) waiter: Option<InteractionWaiter>,
    pub(crate) interaction: Option<Interaction>,
    pub(crate) http_client: &'ctx InteractionClient<'ctx>,
    pub(crate) acknowledge: Option<ResponseFuture<EmptyBody>>,
    pub(crate) parse_fn: fn(&mut Interaction) -> S,
    pub(crate) _marker: PhantomData<S>,
}

impl<'ctx, S> WaitModal<'ctx, S> {
    pub(crate) fn new(
        waiter: InteractionWaiter,
        http_client: &'ctx InteractionClient<'ctx>,
        parse_fn: fn(&mut Interaction) -> S,
    ) -> WaitModal<'ctx, S>
    {
        Self {
            waiter: Some(waiter),
            interaction: None,
            http_client,
            acknowledge: None,
            parse_fn,
            _marker: PhantomData,
        }
    }
}

impl<'ctx, S> Future for WaitModal<'ctx, S> {
    type Output = Result<ModalOutcome<S>, ModalError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };

        if let Some(waiter) = this.waiter.as_mut() {
            let interaction = ready!(Pin::new(waiter).poll(cx))
                .map_err(ModalError::Waiter)?;

            this.waiter = None;
            this.interaction = Some(interaction);
        }

        if this.acknowledge.is_none() {
            let interaction = this.interaction.as_ref().unwrap();
            let response = this.http_client.create_response(
                interaction.id,
                &interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredUpdateMessage,
                    data: None
                }
            );

            this.acknowledge = Some(response.into_future());
        }

        ready!(Pin::new(this.acknowledge.as_mut().unwrap()).poll(cx))
            .map_err(ModalError::Http)?;

        let mut interaction = this.interaction.take().unwrap();

        Poll::Ready(Ok(ModalOutcome {
            inner: (this.parse_fn)(&mut interaction),
            interaction
        }))
    }
}

/// Trait used to define modals that can be sent to discord and parsed by the framework.
///
/// This trait is normally implemented using the derive macro, refer to it to see full
/// documentation about its usage and attributes.
pub trait Modal<D> {
    /// Creates the modal, returning the response needed to send it to discord.
    ///
    /// The framework provides as a custom id the interaction id converted to a string, this custom
    /// id must be used as the response custom id in order for the framework to retrieve the modal
    /// data.
    fn create(ctx: &SlashContext<'_, D>, custom_id: String) -> InteractionResponse;
    /// Parses the provided interaction into the modal;
    fn parse(interaction: &mut Interaction) -> Self;
}
