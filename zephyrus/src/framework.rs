use crate::{
    argument::CommandArgument,
    builder::{FrameworkBuilder, WrappedClient},
    command::{Command, CommandMap},
    context::{AutocompleteContext, Focused, SlashContext},
    group::{GroupParent, GroupParentMap, ParentType},
    hook::{AfterHook, BeforeHook},
    twilight_exports::{
        ApplicationMarker, Client,
        Command as TwilightCommand, CommandDataOption, CommandOption, CommandOptionType,
        CommandOptionValue, GuildMarker, Id, Interaction, InteractionData, InteractionType, InteractionClient, InteractionResponse,
        InteractionResponseType,
    },
    wait::WaiterWaker
};
use tracing::debug;
use parking_lot::Mutex;
use crate::command::ExecutionResult;
use crate::parse::ParseError;

macro_rules! extract {
    ($expr:expr => $variant:ident) => {
        match $expr {
            InteractionData::$variant(inner) => inner,
            _ => unreachable!()
        }
    };
}

macro_rules! focused {
    ($($tt:tt)*) => {
        match $($tt)* {
            CommandOptionValue::Focused(input, kind) => Focused {
                input: input.clone(),
                kind: *kind
            },
            _ => return None
        }
    };
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
    pub async fn process(&self, interaction: Interaction) {
        match interaction.kind {
            InteractionType::ApplicationCommand => self.try_execute(interaction).await,
            InteractionType::ApplicationCommandAutocomplete => self.try_autocomplete(interaction).await,
            InteractionType::MessageComponent | InteractionType::ModalSubmit => {
                let mut lock = self.waiters.lock();
                if let Some(position) = lock.iter().position(|waker| waker.check(&interaction)) {
                    lock.remove(position).wake(interaction);
                }
            }
            _ => ()
        }
    }

    async fn try_execute(&self, mut interaction: Interaction) {
        if let Some(command) = self.get_command(&mut interaction) {
            self.execute(command, interaction).await;
        }
    }

    async fn try_autocomplete(&self, mut interaction: Interaction) {
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
            }
        }
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

            let command = self.get_command(interaction)?;
            let position = command
                .arguments
                .iter()
                .position(|arg| arg.name == focused.name)?;
            return Some((command.name, command.arguments.get(position)?, focused!(&focused.value)));
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
    async fn execute(&self, cmd: &Command<D, T, E>, interaction: Interaction) {
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
            let ExecutionResult::Finished(result) = cmd.execute(&context).await else {
                return;
            };
            if let Some(after) = &self.after {
                (after.0)(&context, cmd.name, result).await;
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
            let mut options = Vec::new();

            for i in &cmd.arguments {
                options.push(i.as_option());
            }
            let interaction_client = self.interaction_client();
            let mut command = interaction_client
                .create_guild_command(guild_id)
                .chat_input(cmd.name, cmd.description)?
                .command_options(&options)?;

            if let Some(permissions) = &cmd.required_permissions {
                command = command.default_member_permissions(*permissions);
            }

            commands.push(command.await?.model().await?);
        }

        for group in self.groups.values() {
            let options = self.create_group(group);
            let interaction_client = self.interaction_client();
            let mut command = interaction_client
                .create_guild_command(guild_id)
                .chat_input(group.name, group.description)?
                .command_options(&options)?;

            if let Some(permissions) = &group.required_permissions {
                command = command.default_member_permissions(*permissions);
            }

            commands.push(command.await?.model().await?);
        }

        Ok(commands)
    }

    /// Registers the commands provided to the framework globally.
    pub async fn register_global_commands(
        &self,
    ) -> Result<Vec<TwilightCommand>, Box<dyn std::error::Error + Send + Sync>> {
        let mut commands = Vec::new();

        for cmd in self.commands.values() {
            let mut options = Vec::new();

            for i in &cmd.arguments {
                options.push(i.as_option());
            }
            let interaction_client = self.interaction_client();
            let mut command = interaction_client
                .create_global_command()
                .chat_input(cmd.name, cmd.description)?
                .command_options(&options)?;

            if let Some(permissions) = &cmd.required_permissions {
                command = command.default_member_permissions(*permissions);
            }

            commands.push(command.await?.model().await?);
        }

        for group in self.groups.values() {
            let options = self.create_group(group);
            let interaction_client = self.interaction_client();
            let mut command = interaction_client
                .create_global_command()
                .chat_input(group.name, group.description)?
                .command_options(&options)?;

            if let Some(permissions) = &group.required_permissions {
                command = command.default_member_permissions(*permissions);
            }

            commands.push(command.await?.model().await?);
        }

        Ok(commands)
    }

    fn arg_options(&self, arguments: &Vec<CommandArgument<D>>) -> Vec<CommandOption> {
        let mut options = Vec::with_capacity(arguments.len());

        for arg in arguments {
            options.push(arg.as_option());
        }

        options
    }

    fn create_group(&self, parent: &GroupParent<D, T, E>) -> Vec<CommandOption> {
        debug!("Registering group {}", parent.name);

        if let ParentType::Group(map) = &parent.kind {
            let mut subgroups = Vec::new();
            for group in map.values() {
                debug!("Registering subgroup [{}] of [{}]", group.name, parent.name);

                let mut subcommands = Vec::new();
                for sub in group.subcommands.values() {
                    subcommands.push(self.create_subcommand(sub))
                }

                subgroups.push(CommandOption {
                    kind: CommandOptionType::SubCommandGroup,
                    name: group.name.to_string(),
                    description: group.description.to_string(),
                    options: Some(subcommands),
                    autocomplete: None,
                    choices: None,
                    required: None,
                    channel_types: None,
                    description_localizations: None,
                    max_length: None,
                    max_value: None,
                    min_length: None,
                    min_value: None,
                    name_localizations: None,
                });
            }
            subgroups
        } else if let ParentType::Simple(map) = &parent.kind {
            let mut subcommands = Vec::new();
            for sub in map.values() {
                subcommands.push(self.create_subcommand(sub));
            }

            subcommands
        } else {
            unreachable!()
        }
    }

    /// Creates a subcommand at the given scope.
    fn create_subcommand(&self, cmd: &Command<D, T, E>) -> CommandOption {
        debug!("Registering [{}] subcommand", cmd.name);

        CommandOption {
            kind: CommandOptionType::SubCommand,
            name: cmd.name.to_string(),
            description: cmd.description.to_string(),
            options: Some(self.arg_options(&cmd.arguments)),
            autocomplete: None,
            choices: None,
            required: None,
            channel_types: None,
            description_localizations: None,
            max_length: None,
            max_value: None,
            min_length: None,
            min_value: None,
            name_localizations: None,
        }
    }
}
