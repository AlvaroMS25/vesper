use std::future::{Future, IntoFuture};
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll, ready};
use tokio::sync::oneshot::error::RecvError;
use crate::context::SlashContext;
use crate::waiter::InteractionWaiter;
use crate::twilight_exports::{Interaction, InteractionResponse, InteractionResponseType};
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

pub struct Modal<'ctx, D, S> {
    pub(crate) waiter: Option<InteractionWaiter>,
    pub(crate) interaction: Option<Interaction>,
    pub(crate) context: &'ctx SlashContext<'ctx, D>,
    pub(crate) acknowledge: Option<ResponseFuture<EmptyBody>>,
    pub(crate) _marker: PhantomData<(D, S)>,
}

impl<'ctx, D, S> Modal<'ctx, D, S> {
    pub(crate) fn new(waiter: InteractionWaiter, ctx: &'ctx SlashContext<'ctx, D>) -> Modal<'ctx, D, S>
    {
        Self {
            waiter: Some(waiter),
            interaction: None,
            context: ctx,
            acknowledge: None,
            _marker: PhantomData,
        }
    }
}

impl<'ctx, D, S> Future for Modal<'ctx, D, S>
where
    S: ParseModal,
{
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
            let response = this.context.interaction_client.create_response(
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

        Poll::Ready(Ok(S::parse(this.interaction.take().unwrap())))
    }
}

pub trait ParseModal: Sized {
    fn parse(interaction: Interaction) -> Self;
}

pub trait CreateModal<D>: ParseModal {
    fn create(ctx: &SlashContext<'_, D>, custom_id: String) -> InteractionResponse;
}
