use crate::{
    argument::CommandArgument,
    builder::{FrameworkBuilder, WrappedClient},
    command::{Command, CommandMap, ExecutionState, OutputLocation},
    context::{AutocompleteContext, Focused, SlashContext},
    group::GroupParentMap,
    hook::{AfterHook, BeforeHook},
    twilight_exports::{
        ApplicationMarker, Client,
        Command as TwilightCommand, CommandDataOption, CommandOptionType,
        CommandOptionValue, GuildMarker, Id, Interaction, InteractionData, InteractionType, InteractionClient, InteractionResponse,
        InteractionResponseType,
    },
    wait::WaiterWaker
};
use tracing::debug;
use parking_lot::Mutex;
use crate::command::ExecutionResult;
use crate::parse::ParseError;
#[cfg(feature = "bulk")]
use crate::if_some;

macro_rules! extract {
    ($expr:expr => $variant:ident) => {
        match $expr {
            InteractionData::$variant(inner) => inner,
            _ => return None
        }
    };
}

/// The result of a `.process` call, containing the state of the interaction handling.
#[non_exhaustive]
pub enum ProcessResult<T, E> {
    /// The specified command was not found, either to execute its handler or to try to autocomplete
    /// an argument.
    CommandNotFound,
    /// The interaction was a modal submit interaction.
    ModalSubmit,
    /// The interaction was a message component interaction.
    MessageComponent,
    /// The specified command argument was autocompleted successufully.
    Autocompleted,
    /// The specified command was executed.
    CommandExecuted(ExecutionResult<T, E>),
    /// The interaction type is not supported. This should unly happen with `Ping` interactions.
    UnknownInteraction
}

/// The default error used by the framework.
pub type DefaultError = Box<dyn std::error::Error + Send + Sync>;

/// A generic return type for commands provided by the framework.
pub type DefaultCommandResult = Result<(), DefaultError>;

/// The framework used to dispatch slash commands.
pub struct Framework<D, T = (), E = DefaultError> {
    /// The http client used by the framework.
    pub http_client: WrappedClient,
    /// The application id of the client.
    pub application_id: Id<ApplicationMarker>,
    /// Data shared across all command and hook invocations.
    pub data: D,
    /// A map of simple commands.
    pub commands: CommandMap<D, T, E>,
    /// A map of command groups including all children.
    pub groups: GroupParentMap<D, T, E>,
    /// A hook executed before the command.
    pub before: Option<BeforeHook<D>>,
    /// A hook executed after command's execution.
    pub after: Option<AfterHook<D, T, E>>,
    pub waiters: Mutex<Vec<WaiterWaker>>
}

impl<D, T, E> Framework<D, T, E>
where
    E: From<ParseError>
{
    pub(crate) fn from_builder(builder: FrameworkBuilder<D, T, E>) -> Self {
        Self {
            http_client: builder.http_client,
            application_id: builder.application_id,
            data: builder.data,
            commands: builder.commands,
            groups: builder.groups,
            before: builder.before,
            after: builder.after,
            waiters: Mutex::new(Vec::new())
        }
    }

    /// Creates a new framework builder, this is a shortcut to FrameworkBuilder.
    /// [new](crate::builder::FrameworkBuilder::new)
    pub fn builder(
        http_client: impl Into<WrappedClient>,
        application_id: Id<ApplicationMarker>,
        data: D,
    ) -> FrameworkBuilder<D, T, E> {
        FrameworkBuilder::new(http_client, application_id, data)
    }

    /// Gets the http client used by the framework.
    pub fn http_client(&self) -> &Client {
        self.http_client.inner()
    }

    /// Gets the [interaction client](InteractionClient) using this framework's
    /// [http client](Client) and [application id](ApplicationMarker)
    pub fn interaction_client(&self) -> InteractionClient {
        self.http_client().interaction(self.application_id)
    }

    /// Processes the given interaction, dispatching commands or waking waiters if necessary.
    pub async fn process(&self, mut interaction: Interaction) -> ProcessResult<T, E> {
        match interaction.kind {
            InteractionType::ApplicationCommand => {
                let Some(command) = self.get_command(&mut interaction) else {
                    self.wake_waiters(interaction);
                    return ProcessResult::CommandNotFound;
                };
                self.execute(command, interaction).await.into()
            },
            InteractionType::ApplicationCommandAutocomplete => self.try_autocomplete(interaction).await,
            InteractionType::MessageComponent  => {
                self.wake_waiters(interaction);
                ProcessResult::MessageComponent
            },
            InteractionType::ModalSubmit => {
                self.wake_waiters(interaction);
                ProcessResult::ModalSubmit
            },
            _ => ProcessResult::UnknownInteraction
        }
    }

    fn wake_waiters(&self, interaction: Interaction) {
        let mut lock = self.waiters.lock();
        if let Some(position) = lock.iter().position(|waker| waker.check(&interaction)) {
            lock.remove(position).wake(interaction);
        }
    }

    async fn try_autocomplete(&self, mut interaction: Interaction) -> ProcessResult<T, E> {
        if let Some((name, argument, value)) = self.get_autocomplete_argument(&interaction) {
            if let Some(fun) = &argument.autocomplete {
                let context = AutocompleteContext::new(
                    &self.http_client,
                    &self.data,
                    value,
                    &mut interaction,
                );
                debug!("Command [{}] executing argument {} autocomplete function", name, argument.name);
                let data = (fun.0)(context).await;

                let _ = self
                    .interaction_client()
                    .create_response(
                        interaction.id,
                        &interaction.token,
                        &InteractionResponse {
                            kind: InteractionResponseType::ApplicationCommandAutocompleteResult,
                            data,
                        },
                    )
                    .await;

                return ProcessResult::Autocompleted;
            }
        }

        ProcessResult::CommandNotFound
    }

    fn get_autocomplete_argument(
        &self,
        interaction: &Interaction,
    ) -> Option<(&str, &CommandArgument<D>, Focused)> {
        let data = extract!(interaction.data.as_ref().unwrap() => ApplicationCommand);
        if !data.options.is_empty() {
            let outer = data.options.get(0)?;
            let focused = match &outer.value {
                CommandOptionValue::SubCommandGroup(sc_group) => {
                    let next = sc_group.get(0)?;
                    if let CommandOptionValue::SubCommand(options) = &next.value {
                        self.get_focus(options)
                    } else {
                        None
                    }
                }
                CommandOptionValue::SubCommand(sc) => self.get_focus(sc),
                _ => self.get_focus(&data.options)
            }?;
            let CommandOptionValue::Focused(ref input, kind) = focused.value else {
                return None;
            };

            let command = self.get_command(interaction)?;
            let position = command
                .arguments
                .iter()
                .position(|arg| arg.name == focused.name)?;
            return Some((
                command.name,
                command.arguments.get(position)?,
                Focused {
                    input: input.clone(),
                    kind
                }
            ));
        }

        None
    }

    fn get_focus<'a>(&self, data: &'a Vec<CommandDataOption>) -> Option<&'a CommandDataOption> {
        for item in data {
            if let CommandOptionValue::Focused(..) = &item.value {
                return Some(item);
            }
        }
        None
    }

    /// Gets the command matching the given
    /// [ApplicationCommand](ApplicationCommand),
    /// returning `None` if no command matches the given interaction.
    fn get_command(&self, interaction: &Interaction) -> Option<&Command<D, T, E>> {
        let data = interaction.data.as_ref()?;
        let interaction_data = extract!(data => ApplicationCommand);
        if let Some(next) = self.get_next(&interaction_data.options) {
            let group = self.groups.get(&*interaction_data.name)?;
            match next.value.kind() {
                CommandOptionType::SubCommand => {
                    let subcommands = group.kind.as_simple()?;
                    subcommands.get(&*next.name)
                }
                CommandOptionType::SubCommandGroup => {
                    let CommandOptionValue::SubCommandGroup(options) = &next.value else {
                        unreachable!();
                    };
                    let subcommand = self.get_next(options)?;
                    let subgroups = group.kind.as_group()?;
                    let group = subgroups.get(&*next.name)?;
                    group.subcommands.get(&*subcommand.name)
                }
                _ => None,
            }
        } else {
            self.commands.get(&*interaction_data.name)
        }
    }

    /// Gets the next [option](CommandDataOption)
    /// only if it corresponds to a subcommand or a subcommand group.
    fn get_next<'a>(&self, interaction: &'a Vec<CommandDataOption>) -> Option<&'a CommandDataOption> {
        if !interaction.is_empty()
            && (interaction[0].value.kind() == CommandOptionType::SubCommand
                || interaction[0].value.kind() == CommandOptionType::SubCommandGroup)
        {
            interaction.get(0)
        } else {
            None
        }
    }

    /// Executes the given [command](crate::command::Command) and the hooks.
    async fn execute(&self, cmd: &Command<D, T, E>, interaction: Interaction) -> ExecutionResult<T, E> {
        let context = SlashContext::new(
            &self.http_client,
            self.application_id,
            &self.data,
            &self.waiters,
            interaction,
        );

        let execute = if let Some(before) = &self.before {
            (before.0)(&context, cmd.name).await
        } else {
            true
        };

        if execute {
            let mut result = cmd.execute(&context).await;

            match (&self.after, result.state) {
                // The after hook should not execute if any check returned false or a check errored.
                (Some(after), 
                ExecutionState::CommandFinished 
                | ExecutionState::CommandErrored) => {
                    // Set the output as taken, if it was already taken, we'll restore it to the previous state.
                    let output = std::mem::replace(&mut result.output, OutputLocation::TakenByAfterHook);

                    let output = if let OutputLocation::Present(return_value) = output {
                        // If the output is not taken beforehand by the error handler, leave it as taken
                        // by the after hook one.
                        Some(return_value)
                    } else {
                        // If it was taken, return it to it's previous state.
                        result.output = output;
                        None
                    };

                    (after.0)(&context, cmd.name, output).await;
                },
                _ => ()
            }

            result
        } else {
            ExecutionResult {
                state: ExecutionState::BeforeHookFailed,
                output: OutputLocation::NotExecuted
            }
        }
    }

    /// Registers the commands provided to the framework in the specified guild.
    pub async fn register_guild_commands(
        &self,
        guild_id: Id<GuildMarker>,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for cmd in self.commands.values() {
            debug!("Registering command [{}]", cmd.name);

            commands.push(cmd.create(&self.interaction_client(), Some(guild_id)).await?);
        }

        for group in self.groups.values() {
            commands.push(group.create(&self.interaction_client(), Some(guild_id)).await?);
        }

        Ok(commands)
    }

    /// Registers the commands provided to the framework globally.
    pub async fn register_global_commands(
        &self,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for cmd in self.commands.values() {
            commands.push(cmd.create(&self.interaction_client(), None).await?);
        }

        for group in self.groups.values() {
            commands.push(group.create(&self.interaction_client(), None).await?);
        }

        Ok(commands)
    }

    /// Creates a vector of Twilight [`Command`](twilight_model::application::command::Command) objects, to be used against Discord's bulk endpoint.
    #[cfg(feature = "bulk")]
    pub fn twilight_commands(
        &self,
    ) -> Vec<TwilightCommand> {
        use twilight_model::application::command::CommandType;
        use twilight_util::builder::command::CommandBuilder;

        let mut commands = Vec::new();

        for cmd in self.commands.values() {
            // only chat input commands can have a description
            // for other types of commands, the description is ignored, provided as an empty string
            let description = match cmd.kind {
                CommandType::ChatInput => cmd.description,
                _ => ""
            };

            let mut command = CommandBuilder::new(cmd.name, description, cmd.kind);

            // only chat input commands can have options and descriptions
            if cmd.kind == CommandType::ChatInput {
                for i in &cmd.arguments {
                    command = command.option(i.as_option());
                }
                if_some!(&cmd.localized_descriptions, |d| command = command.name_localizations(d));
            }

            if_some!(cmd.required_permissions, |p| command = command.default_member_permissions(p));
            if_some!(&cmd.localized_names, |n| command = command.name_localizations(n));
            

            commands.push(command.build());
        }

        for group in self.groups.values() {
            let options = group.get_options();
            // groups are only supported by chat input
            let mut command = CommandBuilder::new(group.name, group.description, CommandType::ChatInput);

            for i in options {
                command = command.option(i);
            }

            if_some!(group.required_permissions, |p| command = command.default_member_permissions(p));

            commands.push(command.build());
        }

        commands
    }
}
