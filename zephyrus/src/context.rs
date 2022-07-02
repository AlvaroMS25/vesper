use crate::{
    builder::WrappedClient,
    command::CommandResult,
    message::Message,
    twilight_exports::*,
    waiter::{WaiterReceiver, WaiterSender},
};
use parking_lot::Mutex;
use crate::iter::DataIterator;
use crate::parse::{Parse, ParseError};

/// Context given to all functions used to autocomplete arguments.
pub struct AutocompleteContext<'a, D> {
    /// The http client used by the framework.
    pub http_client: &'a WrappedClient,
    /// The data shared across the framework.
    pub data: &'a D,
    /// The user input, if exists.
    pub user_input: Option<String>,
    /// The interaction itself.
    pub interaction: &'a mut ApplicationCommandAutocomplete,
}

impl<'a, D> AutocompleteContext<'a, D> {
    pub(crate) fn new(
        http_client: &'a WrappedClient,
        data: &'a D,
        user_input: Option<String>,
        interaction: &'a mut ApplicationCommandAutocomplete,
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
    /// Waiters used to wait for component interactions.
    waiters: &'a Mutex<Vec<WaiterSender>>,
    /// The interaction itself.
    pub interaction: ApplicationCommand,
}

impl<'a, D> Clone for SlashContext<'a, D> {
    fn clone(&self) -> Self {
        SlashContext {
            http_client: &self.http_client,
            application_id: self.application_id,
            interaction_client: self.http_client.inner().interaction(self.application_id),
            data: &self.data,
            waiters: &self.waiters,
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
        waiters: &'a Mutex<Vec<WaiterSender>>,
        interaction: ApplicationCommand,
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

    /// Responds to the interaction with an empty message to allow to respond later.
    ///
    /// When this method is used [update_response](Self::update_response) has to be used to edit the response.
    pub async fn acknowledge(&self) -> CommandResult {
        self.interaction_client
            .create_response(
                self.interaction.id,
                &self.interaction.token,
                &InteractionResponse {
                    kind: InteractionResponseType::DeferredChannelMessageWithSource,
                    data: None,
                },
            )
            .exec()
            .await?;

        Ok(())
    }

    /// Updates the sent interaction, this method is a shortcut to twilight's
    /// [update_interaction_original](InteractionClient::update_response)
    /// but http is automatically provided.
    pub async fn update_response<F>(
        &'a self,
        fun: F,
    ) -> Result<Message<'a, D>, Box<dyn std::error::Error + Send + Sync>>
    where
        F: FnOnce(UpdateResponse<'a>) -> UpdateResponse<'a>,
    {
        let update = fun(self
            .interaction_client
            .update_response(&self.interaction.token));
        Ok(update
            .exec()
            .await?
            .model()
            .await
            .map(|msg| Message::new(&self, msg))?)
    }

    /// Waits for a component interaction which satisfies the given predicate.
    pub fn wait_component<F>(&self, fun: F) -> WaiterReceiver
    where
        F: Fn(&MessageComponentInteraction) -> bool + Send + 'static,
    {
        let (sender, receiver) = WaiterSender::new(fun);
        {
            let mut lock = self.waiters.lock();
            lock.push(sender);
        }
        receiver
    }
}

impl<D: Send + Sync> SlashContext<'_, D> {
    #[doc(hidden)]
    pub async fn named_parse<T>(
        &self,
        name: &str,
        iterator: &mut DataIterator<'_>
    ) -> Result<T, ParseError>
    where
        T: Parse<D>
    {
        let value = iterator.get(|s| s.name == name);
        if value.is_none() && <T as Parse<D>>::is_required() {
            Err(ParseError::StructureMismatch(format!("{} not found", name)))
        } else {
            <T as Parse<D>>::parse(&self.http_client, &self.data, value.map(|it| &it.value)).await
        }
    }
}
