use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tokio::sync::oneshot::error::RecvError;
use crate::context::SlashContext;
use crate::waiter::InteractionWaiter;
use crate::twilight_exports::{Interaction, InteractionClient, InteractionResponse, InteractionResponseType};
use std::{error::Error as StdError, fmt::{Display, Formatter, Result as FmtResult}};
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

pub struct WaitModal<'ctx, S> {
    pub(crate) waiter: Option<InteractionWaiter>,
    pub(crate) interaction: Option<Interaction>,
    pub(crate) http_client: &'ctx InteractionClient<'ctx>,
    pub(crate) acknowledge: Option<ResponseFuture<EmptyBody>>,
    pub(crate) parse_fn: fn(Interaction) -> S,
    pub(crate) _marker: PhantomData<S>,
}

impl<'ctx, S> WaitModal<'ctx, S> {
    pub(crate) fn new(
        waiter: InteractionWaiter,
        http_client: &'ctx InteractionClient<'ctx>,
        parse_fn: fn(Interaction) -> S,
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
    type Output = Result<S, ModalError>;

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

        Poll::Ready(Ok((this.parse_fn)(this.interaction.take().unwrap())))
    }
}

/// Trait used to define modals that can be sent to discord and parsed by the framework.
///
/// This trait is normally implemented using the derive macro, refer to it to see full
/// documentation about its usage and attributes.
pub trait Modal<D> {
    /// Creates the modal, returning the response needed to send it to discord.
    fn create(ctx: &SlashContext<'_, D>, custom_id: String) -> InteractionResponse;
    /// Parses the provided interaction into the modal;
    fn parse(interaction: Interaction) -> Self;
}
