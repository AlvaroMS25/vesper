use parking_lot::Mutex;
use twilight_model::channel::message::MessageFlags;
use crate::{
    builder::WrappedClient,
    twilight_exports::*,
    wait::{InteractionWaiter, WaiterWaker}
};

use crate::iter::DataIterator;
use crate::modal::{Modal, WaitModal};
use crate::parse::{Parse, ParseError};
use crate::wait::new_pair;

/// The value the user is providing to the argument.
#[derive(Debug, Clone)]
pub struct Focused {
    pub input: String,
    pub kind: CommandOptionType,
}

/// Context given to all functions used to autocomplete arguments.
pub struct AutocompleteContext<'a, D> {
    /// The http client used by the framework.
    pub http_client: &'a WrappedClient,
    /// The data shared across the framework.
    pub data: &'a D,
    /// The user input.
    pub user_input: Focused,
    /// The interaction itself.
    pub interaction: &'a mut Interaction,
}

impl<'a, D> AutocompleteContext<'a, D> {
    pub(crate) fn new(
        http_client: &'a WrappedClient,
        data: &'a D,
        user_input: Focused,
        interaction: &'a mut Interaction,
    ) -> Self {
        Self {
            http_client,
            data,
            user_input,
            interaction,
        }
    }

    /// Gets the http client used by the framework.
    pub fn http_client(&self) -> &Client {
        self.http_client.inner()
    }
}

/// Framework context given to all command functions, this struct contains all the necessary
/// items to respond the interaction and access shared data.
pub struct SlashContext<'a, D> {
    /// The http client used by the framework.
    pub http_client: &'a WrappedClient,
    /// The application id provided to the framework.
    pub application_id: Id<ApplicationMarker>,
    /// An [interaction client](InteractionClient) made out of the framework's [http client](Client)
    pub interaction_client: InteractionClient<'a>,
    /// The data shared across the framework.
    pub data: &'a D,
    /// Components waiting for an interaction.
    pub waiters: &'a Mutex<Vec<WaiterWaker>>,
    /// The interaction itself.
    pub interaction: Interaction,
}

impl<'a, D> Clone for SlashContext<'a, D> {
    fn clone(&self) -> Self {
        SlashContext {
            http_client: self.http_client,
            application_id: self.application_id,
            interaction_client: self.http_client.inner().interaction(self.application_id),
            data: self.data,
            waiters: self.waiters,
            interaction: self.interaction.clone(),
        }
    }
}

impl<'a, D> SlashContext<'a, D> {
    /// Creates a new context.
    pub(crate) fn new(
        http_client: &'a WrappedClient,
        application_id: Id<ApplicationMarker>,
        data: &'a D,
        waiters: &'a Mutex<Vec<WaiterWaker>>,
        interaction: Interaction,
    ) -> Self {
        let interaction_client = http_client.inner().interaction(application_id);
        Self {
            http_client,
            application_id,
            interaction_client,
            data,
            waiters,
            interaction,
        }
    }

    /// Gets the http client used by the framework.
    pub fn http_client(&self) -> &Client {
        self.http_client.inner()
    }

    /// Gets a mutable reference to the [interaction](Interaction) owned by the context.
    #[deprecated(since = "0.12.0", note = "Use the `interaction` field directly with a mutable context")]
    pub fn interaction_mut(&mut self) -> &mut Interaction {
        &mut self.interaction
    }

    /// Acknowledges the interaction, allowing to respond later.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    ///
    /// #[command]
    /// #[description = "My command description"]
    /// async fn my_command(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ///     // Acknowledge the interaction, this way we can respond to it later.
    ///     ctx.acknowledge().await?;
    ///
    ///     // Do something here
    ///
    ///     // Now edit the interaction
    ///     ctx.interaction_client.update_response(&ctx.interaction.token)
    ///         .content(Some("Hello world"))
    ///         .unwrap()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    #[deprecated(since = "0.10.0", note = "Use `.defer` instead")]
    pub async fn acknowledge(&self) -> Result<(), twilight_http::Error> {
        self.defer(false).await
    }

    /// Defers the interaction, allowing to respond later.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    ///
    /// #[command]
    /// #[description = "My command description"]
    /// async fn my_command(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ///     // Defer the interaction, this way we can respond to it later.
    ///     ctx.defer(false).await?;
    ///
    ///     // Do something here
    ///
    ///     // Now edit the interaction
    ///     ctx.interaction_client.update_response(&ctx.interaction.token)
    ///         .content(Some("Hello world"))
    ///         .unwrap()
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub async fn defer(&self, ephemeral: bool) -> Result<(), twilight_http::Error> {
        self.interaction_client
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: if ephemeral {
                        Some(InteractionResponseData {
                            flags: Some(MessageFlags::EPHEMERAL),
                            ..Default::default()
                        })
                    } else {
                        None
                    },
                },
            )
            .await?;

        Ok(())
    }

    /// Creates a modal that will be prompted to the user in discord, returning a [`WaitModal`] that
    /// can be `.await`ed to retrieve the user input. If the returned [`WaitModal`] is not awaited,
    /// the modal will not close when submitted and the user won't be able to submit the modal.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use zephyrus::prelude::*;
    ///
    /// #[derive(Debug, Modal)]
    /// struct MyModal {
    ///     #[modal(paragraph)]
    ///     field: String
    /// }
    ///
    /// #[command]
    /// #[description = "My command description"]
    /// async fn my_command(ctx: &SlashContext<()>) -> DefaultCommandResult {
    ///     let modal = ctx.create_modal::<MyModal>().await?;
    ///
    ///     // Here we can do something quick.
    ///
    ///     // Now we await the modal, allowing the user to submit the modal and getting the data
    ///     let data = modal.await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// [`WaitModal`]: WaitModal
    pub async fn create_modal<M>(&self) -> Result<WaitModal<M>, twilight_http::Error>
    where
        M: Modal<D>
    {
        let modal_id = self.interaction.id.to_string();
        self.interaction_client.create_response(
            self.interaction.id,
            &self.interaction.token,
            &M::create(self, modal_id.clone())
        ).await?;

        let waiter = self.wait_interaction(move |interaction| {
            let Some(InteractionData::ModalSubmit(data)) = &interaction.data else {
                return false;
            };

            data.custom_id == modal_id
        });

        Ok(WaitModal::new(waiter, &self.interaction_client, M::parse))
    }

    /// Returns a waiter used to wait for a specific interaction which satisfies the provided
    /// closure.
    pub fn wait_interaction<F>(&self, fun: F) -> InteractionWaiter
    where
        F: Fn(&Interaction) -> bool + Send + 'static
    {
        let (waker, waiter) = new_pair(fun);
        let mut lock = self.waiters.lock();
        lock.push(waker);
        waiter
    }
}
